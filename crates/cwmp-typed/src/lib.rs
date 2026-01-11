mod envelope;
mod request;
use quick_xml::events::BytesText;
use std::borrow::Cow;
use std::sync::Arc;
use xsd_parser_types::xml::{AnyElement, Value};

pub struct ParameterValueStruct {
    pub name: String,
    pub value: ParameterValueStructValue,
}

pub struct ParameterValueStructValue {
    pub type_: String,
    pub value: String,
}

pub struct ParameterList(pub Vec<ParameterValueStruct>);

impl From<ParameterValueStruct> for AnyElement {
    fn from(input: ParameterValueStruct) -> Self {
        let mut root =
            AnyElement::new().name(Cow::Owned("ParameterValueStruct".to_string().into_bytes()));

        root = root.child(Value::Element(
            AnyElement::new()
                .name(Cow::Owned("Name".to_string().into_bytes()))
                .child(Value::Text(BytesText::new(&input.name).into_owned())),
        ));

        root = root.child(Value::Element(
            AnyElement::new()
                .name(Cow::Owned("Value".to_string().into_bytes()))
                .attribute(
                    Cow::Owned("xsi:type".to_string().into_bytes()),
                    Cow::Owned(format!("xsd:{}", &input.value.type_).into_bytes()),
                )
                .child(Value::Text(BytesText::new(&input.value.value).into_owned())),
        ));

        root
    }
}
