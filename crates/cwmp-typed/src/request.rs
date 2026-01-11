use crate::{ParameterList, ParameterValueStruct};
use cwmp_schema::soapenv::{BodyType, BodyTypeContent};
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyAttributes, AnyElement, Value};

pub enum Request {
    SetParameterValues(crate::request::SetParameterValues),
}

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
        AnyElement::new()
            .name(Cow::Borrowed(b"SetParameterValues".as_ref()))
            .child(Value::Element(parameter_list))
    }
}
