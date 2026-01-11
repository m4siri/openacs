use quote::ToTokens;
use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use xsd_parser::config::Resolver;
use xsd_parser::models::meta::{
    AttributesMeta, Base, BuildInMeta, Constrains, ElementMeta, ElementMetaVariant, ElementMode,
    ElementsMeta, GroupMeta, MetaType, MetaTypeVariant, ReferenceMeta, SimpleMeta, UnionMeta,
    UnionMetaType, UnionMetaTypes,
};
use xsd_parser::models::schema::MaxOccurs;
use xsd_parser::pipeline::renderer::NamespaceSerialization;
use xsd_parser::{
    Config, Error, Ident,
    config::{
        Generate, GeneratorFlags, IdentTriple, InterpreterFlags, NamespaceIdent, OptimizerFlags,
        ParserFlags, RenderStep, RendererFlags, Schema,
    },
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::{
        IdentType, Name,
        meta::{MetaTypes, ModuleMeta, SchemaMeta},
        schema::{NamespaceId, SchemaId, Schemas, xs::FormChoiceType},
    },
};
use xsd_parser_types::misc::Namespace;

fn main() -> Result<(), Error> {
    let mut config = Config::default();
    config.parser.schemas = vec![
        Schema::File("xsd/cwmp-1-0.xsd".into()),
        Schema::File("xsd/cwmp-1-1.xsd".into()),
        Schema::File("xsd/cwmp-1-4.xsd".into()), // this imports 1-2. 1-3 & 1-4 are not completely versions
                                                 // but rather just an extension to 1-2
    ];

    config.parser.resolver = vec![Resolver::Web, Resolver::File];

    config.parser.flags = ParserFlags::all();
    config.interpreter.flags = InterpreterFlags::all();
    config.optimizer.flags = OptimizerFlags::all();
    config.generator.flags = GeneratorFlags::all();
    config.generator.generate = Generate::All;
    config.renderer.xsd_parser_types = "xsd_parser_types".into();
    config.renderer.flags = RendererFlags::all();

    let config = config.with_render_steps([
        RenderStep::Types,
        RenderStep::Defaults,
        RenderStep::NamespaceConstants,
        RenderStep::QuickXmlDeserialize {
            boxed_deserializer: false,
        },
        RenderStep::QuickXmlSerialize {
            namespaces: NamespaceSerialization::Global,
            default_namespace: None,
        },
    ]);

    let schemas = exec_parser(config.parser)?;
    let meta_types = exec_interpreter(config.interpreter, &schemas)?;
    let meta_types = typed_envelope_header(&schemas, meta_types)?;
    let meta_types = typed_envelope_body(&schemas, meta_types)?;
    let meta_types = exec_optimizer(config.optimizer, meta_types)?;
    let data_types = exec_generator(config.generator, &schemas, &meta_types)?;
    let module = exec_render(config.renderer, &data_types)?;

    let code = module.to_token_stream().to_string();

    let code = rustfmt_pretty_print(code).unwrap();

    // there's an extra of each for some reason,
    // which i haven't cared to figure out yet, commenting out works.
    let code = code
        .to_string()
        .replace(
            "pub type ArrayType = ::std::string::String;",
            "// pub type ArrayType = ::std::string::String;",
        )
        .replacen(
            "pub type Id = IdElementType;
pub type IdElementType = IdElementType;
",
            "",
            2,
        );

    let mut file = File::create("src/schema.rs")?;
    file.write_all(code.as_bytes())?;
    Ok(())
}

// TODO: refactor
fn typed_envelope_header(schemas: &Schemas, mut types: MetaTypes) -> Result<MetaTypes, Error> {
    let header_container = IdentTriple::from((
        IdentType::Type,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        "Header",
    ))
    .resolve(schemas)
    .unwrap();

    // The header ID exists on all versions, they're structurally same
    // so we are just creating one of each types at a global namespace, rather
    // than inside modules of their each. I assume this should be done for parsing
    // the body into an enum as well.
    let headers = [
        ("ID", "urn:dslforum-org:cwmp-1-0"),
        ("SessionTimeout", "urn:dslforum-org:cwmp-1-2"),
        ("SupportedCWMPVersions", "urn:dslforum-org:cwmp-1-2"),
        ("UseCWMPVersion", "urn:dslforum-org:cwmp-1-2"),
    ]
    .into_iter()
    .map(|(name, ns)| {
        let ident_ty = Ident {
            ns: None,
            name: Name::Named(Cow::Borrowed(&name)),
            type_: IdentType::ElementType,
        };

        let ident = Ident {
            ns: None,
            name: Name::Named(Cow::Borrowed(&name)),
            type_: IdentType::Element,
        };

        let ident_ty_meta = {
            let ident = IdentTriple::from((
                IdentType::ElementType,
                Some(NamespaceIdent::namespace(ns.as_bytes())),
                name,
            ))
            .resolve(schemas)
            .unwrap();
            types.items.get(&ident).unwrap().clone()
        };

        types.items.insert(ident_ty.clone(), ident_ty_meta);

        let ident_meta = {
            let ident = IdentTriple::from((
                IdentType::Element,
                Some(NamespaceIdent::namespace(ns.as_bytes())),
                name,
            ))
            .resolve(schemas)
            .unwrap();
            types.items.get(&ident).unwrap().clone()
        };

        types.items.insert(ident.clone(), ident_meta.clone());

        (ident_ty, ident)
    })
    .map(|(ident_ty, ident)| {
        ElementMeta::new(
            ident.clone(),
            ident_ty.clone(),
            ElementMode::Element,
            FormChoiceType::Unqualified,
        )
    })
    .collect::<Vec<_>>();

    let inner_ident = {
        let header_ty = types.items.get_mut(&header_container).unwrap();
        let MetaTypeVariant::ComplexType(complex_ty) = &mut header_ty.variant else {
            panic!("Header is not a ComplexType");
        };
        complex_ty.max_occurs = MaxOccurs::Unbounded;
        complex_ty.min_occurs = 0;
        complex_ty.content.clone().unwrap()
    };

    let inner_ty = types.items.get_mut(&inner_ident).unwrap();
    inner_ty.variant = MetaTypeVariant::Choice(GroupMeta {
        is_mixed: false,
        elements: ElementsMeta(headers),
    });

    Ok(types)
}
fn get_envelope_fault(schemas: &Schemas) -> Ident {
    IdentTriple::from((
        IdentType::Type,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        "Fault",
    ))
    .resolve(schemas)
    .unwrap()
}

fn typed_body_rpc(schemas: &Schemas, types: &mut MetaTypes) -> Result<Vec<ElementMeta>, Error> {
    let elements: Vec<_> = [
        "GetRPCMethods",
        "SetParameterValues",
        "GetParameterValues",
        "GetParameterNames",
        "SetParameterAttributes",
        "GetParameterAttributes",
        "AddObject",
        "DeleteObject",
        "Reboot",
        "Download",
        "ScheduleDownload",
        "Upload",
        "FactoryReset",
        "GetAllQueuedTransfers",
        "CancelTransfer",
        "ScheduleInform",
        "ChangeDUState",
        "GetRPCMethodsResponse",
        "SetParameterValuesResponse",
        "GetParameterValuesResponse",
        "GetParameterNamesResponse",
        "SetParameterAttributesResponse",
        "GetParameterAttributesResponse",
        "AddObjectResponse",
        "DeleteObjectResponse",
        "RebootResponse",
        "DownloadResponse",
        "ScheduleDownloadResponse",
        "UploadResponse",
        "FactoryResetResponse",
        "GetAllQueuedTransfersResponse",
        "CancelTransferResponse",
        "ScheduleInformResponse",
        "ChangeDUStateResponse",
    ]
    .into_iter()
    .map(|rpc_method| {
        let ident_main = Ident {
            ns: None,
            name: Name::Named(Cow::Borrowed(rpc_method)),
            type_: IdentType::Element,
        };

        let ident = IdentTriple::from((
            IdentType::Element,
            Some(NamespaceIdent::namespace(b"urn:dslforum-org:cwmp-1-2")),
            rpc_method,
        ))
        .resolve(schemas)
        .unwrap();
        {
            let meta = types.items.get(&ident).unwrap();
            types.items.insert(ident_main.clone(), meta.clone());
        }
        let _ = [
            "urn:dslforum-org:cwmp-1-0",
            "urn:dslforum-org:cwmp-1-1",
            "urn:dslforum-org:cwmp-1-2",
        ]
        .into_iter()
        .map(|ns| {
            let ident = IdentTriple::from((
                IdentType::Element,
                Some(NamespaceIdent::namespace(ns.as_bytes())),
                rpc_method,
            ))
            .resolve(schemas)
            .unwrap();
            types.items.remove(&ident);
        })
        .collect::<Vec<_>>();

        ElementMeta::new(
            ident_main.clone(),
            ident_main.clone(),
            ElementMode::Element,
            FormChoiceType::Unqualified,
        )
    })
    .collect();
    Ok(elements)
}

// experimenting chaning the inner structure as well
// to make things easier
fn typed_get_rpc_methods(schemas: &Schemas, mut types: MetaTypes) -> Result<MetaTypes, Error> {
    let array_type_ident = IdentTriple::from((
        IdentType::Attribute,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/encoding/",
        )),
        "arrayType",
    ))
    .resolve(schemas)
    .unwrap();
    let ml_inner = MetaType::new(MetaTypeVariant::Sequence(GroupMeta {
        is_mixed: false,
        elements: ElementsMeta(vec![ElementMeta {
            ident: Ident {
                type_: IdentType::Element,
                ns: None,
                name: Name::named("string"),
            },
            variant: ElementMetaVariant::Type {
                type_: array_type_ident.clone(),
                mode: ElementMode::Element,
            },
            form: FormChoiceType::Unqualified,
            nillable: false,
            min_occurs: 1,
            max_occurs: MaxOccurs::Unbounded,
            display_name: None,
            documentation: Vec::new(),
        }]),
    }));
    let ml_ident = Ident::new("MethodListContent".into());
    types.items.insert(ml_ident.clone(), ml_inner);
    let ty = Ident {
        type_: IdentType::Type,
        ns: Some(NamespaceId(6)),
        name: Name::named("MethodList".into()),
    };
    let meta = types.items.get_mut(&ty).unwrap();
    let MetaTypeVariant::ComplexType(inner) = &mut meta.variant else {
        panic!()
    };

    inner.base = Base::Restriction(ml_ident.clone());

    Ok(types)
}

fn typed_envelope_fault(schemas: &Schemas, mut types: MetaTypes) -> Result<MetaTypes, Error> {
    let mut types = typed_get_rpc_methods(schemas, types).unwrap();

    let efault_ident = get_envelope_fault(schemas);

    let fault_elem_meta = ElementMeta::new(
        efault_ident.clone(),
        efault_ident.clone(),
        ElementMode::Element,
        FormChoiceType::Unqualified,
    );
    let mut rpc = typed_body_rpc(schemas, &mut types).unwrap();
    rpc.push(fault_elem_meta);

    let elements = ElementsMeta(rpc);

    let body_ident = IdentTriple::from((
        IdentType::Type,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        "Body",
    ))
    .resolve(schemas)
    .unwrap();

    let body_content_ident = {
        let body_ty = types.items.get_mut(&body_ident).unwrap();
        let MetaTypeVariant::ComplexType(body_inner) = &mut body_ty.variant else {
            panic!()
        };
        body_inner.max_occurs = MaxOccurs::Unbounded;

        body_inner.content.clone().unwrap()
    };

    let content = types.items.get_mut(&body_content_ident).unwrap();
    content.variant = MetaTypeVariant::Choice(GroupMeta {
        is_mixed: false,
        elements,
    });

    Ok(types)
}

fn create_no_namespace_faults(schemas: &Schemas, mut types: MetaTypes) -> Result<MetaTypes, Error> {
    // creating a dupe of 1-2 but without a ns, 1-2 specifically
    // cause im guessing its backwards compatible and contains new feature if exists.
    let fault_ident = IdentTriple::from((
        IdentType::ElementType,
        Some(NamespaceIdent::namespace(b"urn:dslforum-org:cwmp-1-2")),
        "Fault",
    ))
    .resolve(schemas)
    .unwrap();

    let fault_meta = types.items.get(&fault_ident).unwrap().clone();
    let core_ident = Ident {
        ns: None,
        name: Name::named("Fault"),
        type_: IdentType::ElementType,
    };

    types.items.insert(core_ident.clone(), fault_meta);

    // swap references
    let _: Vec<_> = [
        "urn:dslforum-org:cwmp-1-0",
        "urn:dslforum-org:cwmp-1-1",
        "urn:dslforum-org:cwmp-1-2",
    ]
    .into_iter()
    .map(|ns| {
        let ident = IdentTriple::from((
            IdentType::Element,
            Some(NamespaceIdent::namespace(ns.as_bytes())),
            "Fault",
        ))
        .resolve(schemas)
        .unwrap();
        {
            let meta = types.items.get_mut(&ident).unwrap();
            let MetaTypeVariant::Reference(inner) = &mut meta.variant else {
                panic!()
            };

            inner.type_ = core_ident.clone();
        }
        let ident = IdentTriple::from((
            IdentType::ElementType,
            Some(NamespaceIdent::namespace(ns.as_bytes())),
            "Fault",
        ))
        .resolve(schemas)
        .unwrap();

        types.items.remove(&ident);
        ns
    })
    .collect();

    Ok(types)
}

fn typed_cwmp_fault(schemas: &Schemas, types: MetaTypes) -> Result<MetaTypes, Error> {
    let mut types = create_no_namespace_faults(schemas, types).unwrap();
    let fault_ident = IdentTriple::from((IdentType::ElementType, None, "Fault"))
        .resolve(schemas)
        .unwrap();

    let detail_ident = IdentTriple::from((
        IdentType::Type,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        "detail",
    ))
    .resolve(schemas)
    .unwrap();

    let detail_ident = {
        let detail_meta = types.items.get_mut(&detail_ident).unwrap();
        let MetaTypeVariant::ComplexType(content) = &mut detail_meta.variant else {
            panic!()
        };
        content.content.clone().unwrap()
    };
    let detail_meta = types.items.get_mut(&detail_ident).unwrap();
    detail_meta.variant = MetaTypeVariant::Sequence(GroupMeta {
        is_mixed: false,
        elements: ElementsMeta(vec![ElementMeta::new(
            fault_ident.clone(),
            fault_ident.clone(),
            ElementMode::Element,
            FormChoiceType::Unqualified,
        )]),
    });

    Ok(types)
}

fn typed_envelope_body(schemas: &Schemas, types: MetaTypes) -> Result<MetaTypes, Error> {
    let types = typed_envelope_fault(schemas, types).unwrap();
    let types = typed_cwmp_fault(schemas, types).unwrap();

    Ok(types)
}

pub fn rustfmt_pretty_print(code: String) -> Result<String, Error> {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let mut stdin = child.stdin.take().unwrap();

    write!(stdin, "{code}")?;
    stdin.flush()?;
    drop(stdin);

    let Output {
        status,
        stdout,
        stderr,
    } = child.wait_with_output()?;

    let stdout = String::from_utf8_lossy(&stdout);
    let stderr = String::from_utf8_lossy(&stderr);

    if !status.success() {
        let code = status.code();
        match code {
            Some(code) => {
                if code != 0 {
                    panic!("The `rustfmt` command failed with return code {code}!\n{stderr}");
                }
            }
            None => {
                panic!("The `rustfmt` command failed!\n{stderr}")
            }
        }
    }

    Ok(stdout.into())
}
