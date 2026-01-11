mod envelope;
mod request;
mod response;

use quick_xml::events::BytesText;
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyElement, Value};

#[derive(Debug)]
pub struct ParameterValueStruct {
    pub name: String,
    pub value: ParameterValueStructValue,
}

#[derive(Debug)]
pub struct ParameterValueStructValue {
    pub type_: String,
    pub value: String,
}
#[derive(Debug)]
pub struct ParameterList(pub Vec<ParameterValueStruct>);

impl From<ParameterValueStruct> for AnyElement {
    fn from(input: ParameterValueStruct) -> Self {
        let mut root = AnyElement::new().name(Cow::Borrowed("ParameterValueStruct".as_ref()));

        root = root.child(Value::Element(
            AnyElement::new()
                .name(Cow::Borrowed("Name".as_ref()))
                .child(Value::Text(BytesText::new(&input.name).into_owned())),
        ));

        root = root.child(Value::Element(
            AnyElement::new()
                .name(Cow::Borrowed("Value".as_ref()))
                .attribute(
                    Cow::Borrowed("xsi:type".as_ref()),
                    Cow::Owned(format!("xsd:{}", &input.value.type_).into_bytes()),
                )
                .child(Value::Text(BytesText::new(&input.value.value).into_owned())),
        ));

        root
    }
}
