use crate::{ParameterList, ParameterValueStruct};
use cwmp_schema::soapenv::{BodyType, BodyTypeContent};
use quick_xml::events::BytesText;
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyAttributes, AnyElement, Value};

#[derive(Debug)]
pub enum Request {
    SetParameterValues(crate::request::SetParameterValues),
}

#[derive(Debug)]
pub struct SetParameterValues {
    pub(crate) parameter_list: ParameterList,
    pub(crate) parameter_key: String,
}

impl From<Request> for AnyElement {
    fn from(input: Request) -> Self {
        match input {
            Request::SetParameterValues(inner) => {
                <SetParameterValues as Into<AnyElement>>::into(inner)
            }
        }
    }
}

impl From<SetParameterValues> for AnyElement {
    fn from(input: SetParameterValues) -> Self {
        let parameter_list = {
            let mut root = AnyElement::new()
                .name(Cow::Borrowed(b"ParameterList".as_ref()))
                .attribute(
                    Cow::Borrowed(b"soapenc:arrayType".as_ref()),
                    Cow::Owned(
                        format!(
                            "cwmp:ParameterValueStruct[{}]",
                            &input.parameter_list.0.len()
                        )
                        .into_bytes(),
                    ),
                );
            for param in input.parameter_list.0.into_iter() {
                root = root.child(Value::Element(param.into()));
            }
            root
        };
        let parameter_key = AnyElement::new()
            .name(Cow::Borrowed(b"ParameterKey".as_ref()))
            .child(Value::Text(
                BytesText::new(&input.parameter_key).into_owned(),
            ));

        AnyElement::new()
            .name(Cow::Borrowed(b"cwmp:SetParameterValues".as_ref()))
            .child(Value::Element(parameter_list))
            .child(Value::Element(parameter_key))
    }
}
