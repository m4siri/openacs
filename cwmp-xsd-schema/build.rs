use quote::ToTokens;
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;
use std::process::{Command, Output, Stdio};
use xsd_parser::config::Resolver;
use xsd_parser::models::meta::{
    AttributeMeta, AttributesMeta, Base, ComplexMeta, CustomMeta, ElementMeta, ElementMetaVariant,
    ElementMode, ElementsMeta, GroupMeta, MetaTypeVariant, SimpleMeta,
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
        meta::{MetaType, MetaTypes},
        schema::{Schemas, xs::FormChoiceType},
    },
};

fn main() -> Result<(), Error> {
    let mut config = Config::default();
    config.parser.schemas = vec![
        Schema::File("xsd/xsi.xsd".into()),
        Schema::File("xsd/cwmp-1-0.xsd".into()),
        Schema::File("xsd/cwmp-1-1.xsd".into()),
        Schema::File("xsd/cwmp-1-4.xsd".into()), // this imports 1-2. 1-3 & 1-4 are not completely new versions
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

    config.interpreter.types = vec![(
        IdentTriple::from((
            IdentType::Attribute,
            NamespaceIdent::namespace(b"http://www.w3.org/2001/XMLSchema-instance"),
            "type",
        )),
        MetaType::from(CustomMeta::new("XsiType").include_from("::cwmp_xsd_types::XsiType")),
    )];

    let config = config.with_render_steps([
        RenderStep::Types,
        RenderStep::Defaults,
        RenderStep::NamespaceConstants,
        RenderStep::QuickXmlDeserialize {
            boxed_deserializer: false,
        },
    ]);

    let schemas = exec_parser(config.parser)?;
    let meta_types = exec_interpreter(config.interpreter, &schemas)?;
    let meta_types = typed_envelope(&schemas, meta_types)?;
    let meta_types = exec_optimizer(config.optimizer, meta_types)?;
    let data_types = exec_generator(config.generator, &schemas, &meta_types)?;
    let module = exec_render(config.renderer, &data_types)?;

    let code = module.to_token_stream().to_string();

    let code = rustfmt_pretty_print(code).unwrap();

    // lol
    let code = code.to_string().replace(
        "pub type ArrayType = ::std::string::String;",
        "// pub type ArrayType = ::std::string::String;",
    );

    let mut file = File::create("src/schema.rs")?;
    file.write_all(code.as_bytes())?;
    Ok(())
}

pub fn typed_envelope(schemas: &Schemas, types: MetaTypes) -> Result<MetaTypes, Error> {
    let types = create_inner_content(schemas, types, "Header", create_header_content)?;
    let types = create_inner_content(schemas, types, "Body", create_body_content)?;
    let types = create_inner_content(schemas, types, "Array", create_array_type_content)?;
    let types = create_inner_content(schemas, types, "detail", create_soap_fault_detail_content)?;

    let types = fix_parameter_value_struct(schemas, types)?;

    Ok(types)
}

pub fn fix_parameter_value_struct(
    schemas: &Schemas,
    mut types: MetaTypes,
) -> Result<MetaTypes, Error> {
    let ident = {
        let type_ident = IdentTriple::from((
            IdentType::Attribute,
            NamespaceIdent::namespace(b"http://www.w3.org/2001/XMLSchema-instance"),
            "type",
        ))
        .resolve(schemas)
        .unwrap();

        let value_ident = Ident::new(Name::named("ValueTypeContent"));
        let ty = MetaType::new(MetaTypeVariant::SimpleType(SimpleMeta::new(Ident::STRING)));
        types.items.insert(value_ident.clone(), ty);

        let ident = Ident::new(Name::named("ValueType"));
        let mut attributes = AttributesMeta::default();
        attributes.deref_mut().push(AttributeMeta::new(
            type_ident.clone(),
            type_ident,
            FormChoiceType::Qualified,
        ));
        let ty = MetaType::new(MetaTypeVariant::ComplexType(ComplexMeta {
            base: Base::None,
            content: Some(value_ident),
            min_occurs: 1,
            max_occurs: MaxOccurs::Bounded(1),
            is_dynamic: false,
            is_mixed: false,
            attributes: attributes,
        }));
        types.items.insert(ident.clone(), ty);
        ident
    };

    let namespaces = vec![
        b"urn:dslforum-org:cwmp-1-0",
        b"urn:dslforum-org:cwmp-1-1",
        b"urn:dslforum-org:cwmp-1-2",
    ];

    let arg_structs = namespaces
        .iter()
        .filter_map(|ns| {
            let ty = IdentTriple::from((
                IdentType::Type,
                Some(NamespaceIdent::namespace(*ns)),
                "ParameterValueStruct",
            ))
            .resolve(schemas)
            .ok()
            .and_then(|ident| types.items.get(&ident))?;

            let MetaTypeVariant::ComplexType(inner) = &ty.variant else {
                return None;
            };

            inner.content.clone()
        })
        .collect::<Vec<_>>();

    for arg_struct in arg_structs.into_iter() {
        let ty = types.items.get_mut(&arg_struct).unwrap();
        let MetaTypeVariant::Sequence(inner) = &mut ty.variant else {
            panic!();
        };

        for el in inner.elements.iter_mut() {
            if el.ident.name.as_str() == "Value" {
                el.variant = ElementMetaVariant::Type {
                    type_: ident.clone(),
                    mode: ElementMode::Element,
                };
            }
        }
    }
    Ok(types)
}

pub fn create_soap_fault_detail_content(schemas: &Schemas, types: &MetaTypes) -> ElementsMeta {
    let cwmp_fault = vec!["Fault"];
    create_choice_elements(cwmp_fault, schemas, types, IdentType::Element)
}

pub fn create_body_content(schemas: &Schemas, types: &MetaTypes) -> ElementsMeta {
    let mut elements: Vec<ElementMeta> = Vec::new();
    let mut rpc_elements = create_rpc(schemas, &types);
    let mut soap_faults = create_soap_fault(schemas, &types);

    elements.append(&mut rpc_elements);
    elements.append(&mut soap_faults);

    ElementsMeta(elements)
}

pub fn create_soap_fault(schemas: &Schemas, _: &MetaTypes) -> ElementsMeta {
    let ident = IdentTriple::from((
        IdentType::Element,
        Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        "Fault",
    ))
    .resolve(schemas)
    .unwrap();
    let element = ElementMeta {
        ident: ident.clone(),
        display_name: Some("SoapFault".to_string()),
        form: FormChoiceType::Unqualified,
        nillable: false,
        min_occurs: 1,
        max_occurs: MaxOccurs::Bounded(1),
        documentation: vec![],
        variant: ElementMetaVariant::Type {
            type_: ident,
            mode: ElementMode::Element,
        },
    };
    ElementsMeta(vec![element])
}

pub fn create_rpc(schemas: &Schemas, types: &MetaTypes) -> ElementsMeta {
    let rpcs = vec![
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
    ];
    create_choice_elements(rpcs, schemas, types, IdentType::ElementType)
}

pub fn create_header_content(schemas: &Schemas, types: &MetaTypes) -> ElementsMeta {
    let headers = vec![
        "ID",
        "HoldRequests",
        "SessionTimeout",
        "SupportedCWMPVersions",
        "UseCWMPVersion",
    ];

    create_choice_elements(headers, schemas, types, IdentType::Element)
}

pub fn create_array_type_content(schemas: &Schemas, types: &MetaTypes) -> ElementsMeta {
    let array_types = vec![
        "MethodList",
        "ParameterValueStruct",
        "SetParameterAttributesStruct",
    ];
    let mut elements = create_choice_elements(array_types, schemas, types, IdentType::Type);

    elements.push(ElementMeta {
        ident: Ident {
            ns: None,
            name: Name::named("string"),
            type_: IdentType::Element,
        },
        display_name: Some("String".to_string()),
        form: FormChoiceType::Unqualified,
        nillable: false,
        min_occurs: 1,
        max_occurs: MaxOccurs::Bounded(1),
        documentation: vec![],
        variant: ElementMetaVariant::Type {
            type_: Ident::STRING,
            mode: ElementMode::Element,
        },
    });

    elements
}

pub fn create_inner_content<F>(
    schemas: &Schemas,
    mut types: MetaTypes,
    element_name: &str,
    choice_elements: F,
) -> Result<MetaTypes, Error>
where
    F: Fn(&Schemas, &MetaTypes) -> ElementsMeta,
{
    let ns = match element_name {
        "Array" => Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/encoding/",
        )),
        "Header" | "Body" | "detail" => Some(NamespaceIdent::namespace(
            b"http://schemas.xmlsoap.org/soap/envelope/",
        )),
        _ => None,
    };

    let ident = {
        let ident = IdentTriple::from((IdentType::Type, ns, element_name))
            .resolve(schemas)
            .unwrap();

        let ty = types.items.get_mut(&ident).unwrap();

        let MetaTypeVariant::ComplexType(inner) = &mut ty.variant else {
            panic!()
        };

        if element_name == "Header" || element_name == "Array" {
            inner.min_occurs = 1;
            inner.max_occurs = MaxOccurs::Unbounded;
        }

        inner.content.clone().unwrap()
    };

    let elements = choice_elements(schemas, &types);
    let ty = types.items.get_mut(&ident).unwrap();

    // a rust enum to represent headers or request/responses across different CWMP version.
    ty.variant = MetaTypeVariant::Choice(GroupMeta {
        is_mixed: false,
        elements,
    });

    Ok(types)
}

pub fn create_choice_elements(
    choice: Vec<&str>,
    schemas: &Schemas,
    types: &MetaTypes,
    ident_type: IdentType,
) -> ElementsMeta {
    let namespaces = vec![
        (b"urn:dslforum-org:cwmp-1-0", "_10"),
        (b"urn:dslforum-org:cwmp-1-1", "_11"),
        (b"urn:dslforum-org:cwmp-1-2", "_12"),
    ];

    let rpcs = choice
        .into_iter()
        .map(|rpc| {
            namespaces
                .iter()
                .filter_map(|ns| {
                    let ident =
                        IdentTriple::from((ident_type, Some(NamespaceIdent::namespace(ns.0)), rpc))
                            .resolve(schemas)
                            .ok()?;

                    types.items.get(&ident).map(|_| (ident, ns.1))
                })
                .collect::<Vec<(Ident, &str)>>()
        })
        .flatten()
        .map(|(ident, suffix)| {
            let elem_ident = ident.clone();
            let display_name = format!("{}{}", ident.name.clone(), suffix);

            let element = ElementMeta {
                ident,
                display_name: Some(display_name),
                form: FormChoiceType::Unqualified,
                nillable: false,
                min_occurs: 1,
                max_occurs: MaxOccurs::Bounded(1),
                documentation: vec![],
                variant: ElementMetaVariant::Type {
                    type_: elem_ident,
                    mode: ElementMode::Element,
                },
            };
            element
        })
        .collect::<Vec<_>>();

    ElementsMeta(rpcs)
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
