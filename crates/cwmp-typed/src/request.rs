use crate::{ParameterList, ParameterValueStruct};
use cwmp_schema::soapenv::{BodyType, BodyTypeContent};
use quick_xml::events::BytesText;
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyAttributes, AnyElement, Value};

#[derive(Debug)]
pub enum Request {
    SetParameterValues(crate::request::SetParameterValues),
    GetParameterValues(crate::request::GetParameterValues),
}

#[derive(Debug)]
pub struct GetParameterValues {
    pub(crate) parameter_names: ParameterNames,
}

#[derive(Debug)]
pub struct ParameterNames(Vec<ParameterName>);

#[derive(Debug)]
pub struct SetParameterValues {
    pub(crate) parameter_list: ParameterList,
    pub(crate) parameter_key: String,
}

#[derive(Debug)]
pub enum ParameterName {
    Partial(String),
    Full(String),
    WildCard(String), // refer: 3.6.2, A.2.4
}

impl From<Request> for AnyElement {
    fn from(input: Request) -> Self {
        let request: AnyElement = match input {
            Request::SetParameterValues(inner) => inner.into(),
            Request::GetParameterValues(inner) => inner.into(),
        };
        request
    }
}

impl From<GetParameterValues> for AnyElement {
    fn from(input: GetParameterValues) -> Self {
        AnyElement::new()
            .name(Cow::Borrowed(b"cwmp:GetParameterValues".as_ref()))
            .child(Value::Element(input.parameter_names.into()))
    }
}

impl From<ParameterNames> for AnyElement {
    fn from(input: ParameterNames) -> Self {
        let mut parameter_names = AnyElement::new()
            .name(Cow::Borrowed(b"ParameterNames".as_ref()))
            .attribute(
                Cow::Borrowed(b"soapenc:arrayType".as_ref()),
                Cow::Owned(format!("xsd:string[{}]", &input.0.len()).into_bytes()),
            );

        for parameter_name in input.0.into_iter() {
            let child = AnyElement::new()
                .name(Cow::Borrowed(b"string".as_ref()))
                .child(Value::Text(
                    BytesText::new(&parameter_name.inner()).into_owned(),
                ));

            parameter_names = parameter_names.child(Value::Element(child));
        }

        parameter_names
    }
}

impl From<SetParameterValues> for AnyElement {
    fn from(input: SetParameterValues) -> Self {
        let parameter_key = AnyElement::new()
            .name(Cow::Borrowed(b"ParameterKey".as_ref()))
            .child(Value::Text(
                BytesText::new(&input.parameter_key).into_owned(),
            ));

        AnyElement::new()
            .name(Cow::Borrowed(b"cwmp:SetParameterValues".as_ref()))
            .child(Value::Element(input.parameter_list.into()))
            .child(Value::Element(parameter_key))
    }
}

impl ParameterName {
    fn inner(self) -> String {
        match self {
            Self::Partial(inner) => inner,
            Self::Full(inner) => inner,
            Self::WildCard(inner) => inner,
        }
    }
}

impl GetParameterValues {
    pub fn new() -> Self {
        Self {
            parameter_names: ParameterNames(Vec::new()),
        }
    }

    pub fn with_parameter(
        mut self,
        parameter_name: impl Into<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        self.parameter_names
            .0
            .push(parameter_name.into().try_into()?);
        Ok(self)
    }
}

impl TryFrom<String> for ParameterName {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: String) -> Result<Self, Self::Error> {
        if input.ends_with('*') || input.ends_with("*.") {
            todo!("Wildcard '*' cannot be the last part of a path");
        }

        let partial = input.ends_with('.');
        let wildcard = input.contains('*');

        let parameter_name = match (partial, wildcard) {
            (true, true) => Self::Partial(input), // ends with '.' and contains '*'
            (true, false) => Self::Partial(input), // ends with '.' only
            (false, true) => Self::WildCard(input), // contains '*' but doesn't end with '.'
            (false, false) => Self::Full(input),  // neither
        };

        Ok(parameter_name)
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use quick_xml::Writer;
    use xsd_parser_types::quick_xml::SerializeSync;
    use xsd_parser_types::xml::{AnyElement, Value};

    fn normalize_xml(s: &str) -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn parse_get_parameter_values() {
        let request = request::GetParameterValues::new()
            .with_parameter("Device.DeviceInfo.".to_string())
            .unwrap()
            .with_parameter(
                "Device.WANDevice.*.WANConnectionDevice.1.WANIPConnection.1.ExternalIPAddress",
            )
            .unwrap()
            .with_parameter(
                "Device.WANDevice.1.WANConnectionDevice.1.WANIPConnection.1.ExternalIPAddress",
            )
            .unwrap();

        let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
        let _ = AnyElement::new()
            .child(Value::Element(request.into()))
            .serialize("soap:Body", &mut writer);
        let xml = String::from_utf8(writer.into_inner()).unwrap();
        let expected = r#"
    <soap:Body>
        <cwmp:GetParameterValues>
            <ParameterNames soapenc:arrayType="xsd:string[3]">
                <string>Device.DeviceInfo.</string>
                <string>Device.WANDevice.*.WANConnectionDevice.1.WANIPConnection.1.ExternalIPAddress</string>
                <string>Device.WANDevice.1.WANConnectionDevice.1.WANIPConnection.1.ExternalIPAddress</string>
            </ParameterNames>
        </cwmp:GetParameterValues>
    </soap:Body>
        "#;

        assert!(normalize_xml(&xml) == normalize_xml(&expected));
    }
}
