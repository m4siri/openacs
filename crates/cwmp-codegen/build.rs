use std::fs::File;
use std::io::Write;
use std::process::{Command, Output, Stdio};
use quote::ToTokens;

use xsd_parser::pipeline::renderer::NamespaceSerialization;
use xsd_parser::{
    config::{GeneratorFlags, InterpreterFlags, RendererFlags, OptimizerFlags, ParserFlags, RenderStep, Schema,Generate, IdentTriple},
    generate, Config, Error,   
    exec_generator, exec_interpreter, exec_optimizer, exec_parser, exec_render,
    models::{meta::MetaTypes, schema::Schemas, IdentType},
};

use xsd_parser::models::meta::MetaTypeVariant;
use xsd_parser::models::meta::ElementMetaVariant;
use xsd_parser::models::meta::ElementMode;
use xsd_parser::config::Resolver;


fn main() -> Result<(), Error> {
    let mut config = Config::default();
    config.parser.schemas = vec![
        Schema::File("xsd/cwmp-1-0.xsd".into()),
        Schema::File("xsd/cwmp-1-1.xsd".into()),
        Schema::File("xsd/cwmp-1-2.xsd".into()),
        Schema::File("xsd/cwmp-1-3.xsd".into()),
        Schema::File("xsd/cwmp-1-4.xsd".into()),
    ];

    config.parser.resolver = vec![
        Resolver::Web,
        Resolver::File
    ];

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
    let meta_types = define_custom_names(&schemas, meta_types)?;
    let meta_types = exec_optimizer(config.optimizer, meta_types)?;
    let data_types = exec_generator(config.generator, &schemas, &meta_types)?;
    let module = exec_render(config.renderer, &data_types)?;


    // let code = generate(config)?;
    // let code = code.to_string();
    let code = module.to_token_stream().to_string();


    // Use a small helper to pretty-print the code (it uses `RUSTFMT`).
    let code = rustfmt_pretty_print(code).unwrap();

    // there's an extra arrayType in the schema that's a string type
    let code = code.to_string().replace("pub type ArrayType = ::std::string::String;", "// pub type ArrayType = ::std::string::String;");

    let mut file = File::create("src/schema.rs")?;
    file.write_all(code.as_bytes())?;

    Ok(())
}

// A small helper to call `rustfmt` when generating file(s).
// This may be useful to compare different versions of generated files.
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

fn define_custom_names(schemas: &Schemas, mut types: MetaTypes) -> Result<MetaTypes, Error> {

    // let target_ident = IdentTriple::from((IdentType::Type, "GetParameterValuesResponse"));
    // let target_ident = IdentTriple::from((IdentType::Type, "GetParameterValuesResponse"));
    let ident = IdentTriple::from((IdentType::Type, "GetParameterValuesResponse"));
    let ident = ident.resolve(schemas)?;

    // dbg!("HERERER");
    // dbg!(&ident);

    // dbg!(&ident);
    //  let ty = types.items.get_mut(&ident).unwrap();
    // let MetaTypeVariant::ComplexType(ty) = &ty.variant else { panic!(); }; // I assume your type is a complex type
    // let content = ty.content.clone().unwrap();
    
    // // Get the content type
    // let ty = types.items.get_mut(&ident).unwrap();
    // let MetaTypeVariant::Sequence(ty) = &mut ty.variant else { panic!(); }; // I assume the content of your type is a sequence
    // let ti = target_ident.resolve(schemas)?;
    // // Loop through the elements
    // for el in &mut *ty.elements {
    //     if el.ident.name.as_str() == "my-element" {
    //         let schema = ti.clone();
    //         el.variant = ElementMetaVariant::Type { // For simplicity I assign a new `ElementMetaVariant` here
    //             type_: schema,                // A better approach would be to check if it is a `::Type`
    //             mode: ElementMode::Element,         // and then only assign `type_`.
    //         }
    //     }
    // }

    Ok(types)

}
