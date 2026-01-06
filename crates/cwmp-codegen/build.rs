use quote::ToTokens;
use std::borrow::Cow;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use xsd_parser::config::Resolver;
use xsd_parser::models::meta::{
    ElementMeta, ElementMode, ElementsMeta, GroupMeta, MetaTypeVariant,
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
        meta::MetaTypes,
        schema::{Schemas, xs::FormChoiceType},
    },
};

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
