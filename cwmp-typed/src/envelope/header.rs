use super::Element;
use crate::cwmp;
use crate::cwmp::CwmpVersion;
use crate::error::Error;
use quick_xml::events::BytesText;
use std::borrow::Cow;
use std::convert::TryFrom;
use xsd_parser_types::xml::{AnyElement, Value};

use std::ops::{Deref, DerefMut};

const SOAP_MUST_UNDERSTAND: &'static str = "soap:mustUnderstand";

impl Deref for Element {
    type Target = xsd_parser_types::xml::AnyElement;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
pub struct EnvelopeHeaders(Vec<EnvelopeHeader>);

#[derive(Debug)]
pub enum EnvelopeHeader {
    Id(cwmp::header::Id),
    HoldRequests(cwmp::header::HoldRequests),
    SessionTimeout(cwmp::header::SessionTimeout),
    SupportedCwmpVersions(cwmp::header::SupportedCwmpVersions),
    UseCwmpVersion(cwmp::header::UseCwmpVersion),
}

#[inline(always)]
fn soap_bool(b: bool) -> &'static str {
    if b { "1" } else { "0" }
}

impl TryFrom<(&CwmpVersion, cwmp::header::Id)> for Element {
    type Error = Error;

    fn try_from((_, input): (&CwmpVersion, cwmp::header::Id)) -> Result<Element, Self::Error> {
        let element = AnyElement::new()
            .name(Cow::Borrowed("cwmp:ID".as_bytes()))
            .attribute(
                Cow::Borrowed(SOAP_MUST_UNDERSTAND.as_bytes()),
                Cow::Borrowed(soap_bool(input.must_understand).as_ref()),
            )
            .child(Value::Text(BytesText::new(&input.content).into_owned()));
        Ok(Element(element))
    }
}

impl TryFrom<(&CwmpVersion, cwmp::header::HoldRequests)> for Element {
    type Error = Error;

    fn try_from(
        (_, input): (&CwmpVersion, cwmp::header::HoldRequests),
    ) -> Result<Element, Self::Error> {
        let element = AnyElement::new()
            .name(Cow::Borrowed("cwmp:ID".as_bytes()))
            .attribute(
                Cow::Borrowed(SOAP_MUST_UNDERSTAND.as_bytes()),
                Cow::Borrowed(soap_bool(input.must_understand).as_ref()),
            )
            .child(Value::Text(
                BytesText::new(soap_bool(input.content)).into_owned(),
            ));
        Ok(Element(element))
    }
}

impl TryFrom<(&CwmpVersion, cwmp::header::SessionTimeout)> for Element {
    type Error = Error;

    fn try_from(
        (version, input): (&CwmpVersion, cwmp::header::SessionTimeout),
    ) -> Result<Element, Self::Error> {
        match version {
            CwmpVersion::_12 => {
                let mut element =
                    AnyElement::new().name(Cow::Borrowed("cwmp:SessionTimeout".as_bytes()));
                if let Some(must_understand) = input.must_understand {
                    element = element.attribute(
                        Cow::Borrowed(SOAP_MUST_UNDERSTAND.as_bytes()),
                        Cow::Borrowed(soap_bool(must_understand).as_ref()),
                    );
                }
                let element = element.child(Value::Text(
                    BytesText::new(&input.content.to_string()).into_owned(),
                ));
                Ok(Element(element))
            }
            _ => Err(Error::UnsupportedHeaderVersion(
                "SessionTimeout".to_string(),
            )),
        }
    }
}

impl TryFrom<(&CwmpVersion, cwmp::header::SupportedCwmpVersions)> for Element {
    type Error = Error;

    fn try_from(
        (version, input): (&CwmpVersion, cwmp::header::SupportedCwmpVersions),
    ) -> Result<Element, Self::Error> {
        match version {
            CwmpVersion::_12 => {
                let mut element =
                    AnyElement::new().name(Cow::Borrowed("cwmp:SupportedCwmpVersions".as_bytes()));
                if let Some(must_understand) = input.must_understand {
                    element = element.attribute(
                        Cow::Borrowed(SOAP_MUST_UNDERSTAND.as_bytes()),
                        Cow::Borrowed(soap_bool(must_understand).as_ref()),
                    );
                }
                let element = element.child(Value::Text(
                    BytesText::new(&input.content.to_string()).into_owned(),
                ));
                Ok(Element(element))
            }
            _ => Err(Error::UnsupportedHeaderVersion(
                "SupportedCwmpVersions".to_string(),
            )),
        }
    }
}

impl TryFrom<(&CwmpVersion, cwmp::header::UseCwmpVersion)> for Element {
    type Error = Error;

    fn try_from(
        (version, input): (&CwmpVersion, cwmp::header::UseCwmpVersion),
    ) -> Result<Element, Self::Error> {
        match version {
            CwmpVersion::_12 => {
                let element = AnyElement::new()
                    .name(Cow::Borrowed("cwmp:UseCwmpVersion".as_bytes()))
                    .attribute(
                        Cow::Borrowed(SOAP_MUST_UNDERSTAND.as_bytes()),
                        Cow::Borrowed(soap_bool(input.must_understand).as_ref()),
                    )
                    .child(Value::Text(
                        BytesText::new(&input.content.to_string()).into_owned(),
                    ));
                Ok(Element(element))
            }

            _ => Err(Error::UnsupportedHeaderVersion(
                "UseCwmpVersion".to_string(),
            )),
        }
    }
}
#[cfg(test)]
mod test {
    use crate::cwmp::CwmpVersion;
    use crate::cwmp::header::{
        HoldRequests, Id, SessionTimeout, SupportedCwmpVersions, UseCwmpVersion,
    };
    use crate::envelope::Element;
    use xsd_parser_types::quick_xml::{SerializeSync, Writer};

    fn normalize_xml(s: &str) -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn header_id_serialization() {
        let serialize = |header: Id| -> String {
            let element: Element = (&CwmpVersion::_10, header).try_into().unwrap();
            let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
            let _ = element.0.serialize("cwmp:ID", &mut writer);
            String::from_utf8(writer.into_inner()).unwrap()
        };
        {
            let header = Id {
                must_understand: true,
                content: "WwuvPbeTOPUPh0VI6G8zgkV8u".to_string(),
            };
            let xml = serialize(header);
            let expected =
                r#"<cwmp:ID soap:mustUnderstand="1">WwuvPbeTOPUPh0VI6G8zgkV8u</cwmp:ID>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = Id {
                must_understand: false,
                content: "czXtQjoYDAMUAsQvRxvRhJE0x".to_string(),
            };
            let xml = serialize(header);
            let expected =
                r#"<cwmp:ID soap:mustUnderstand="0">czXtQjoYDAMUAsQvRxvRhJE0x</cwmp:ID>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        }
    }

    #[test]
    fn header_session_timeout_serialization() {
        let serialize = |header: SessionTimeout| -> String {
            let element: Element = (&CwmpVersion::_12, header).try_into().unwrap();
            let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
            let _ = element.0.serialize("cwmp:SessionTimeout", &mut writer);
            String::from_utf8(writer.into_inner()).unwrap()
        };
        {
            let header = SessionTimeout {
                must_understand: Some(true),
                content: 60,
            };
            let xml = serialize(header);
            let expected =
                r#"<cwmp:SessionTimeout soap:mustUnderstand="1">60</cwmp:SessionTimeout>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = SessionTimeout {
                must_understand: None,
                content: 50,
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:SessionTimeout>50</cwmp:SessionTimeout>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        }
    }
    #[test]
    fn header_hold_requests_serialization() {
        let serialize = |header: HoldRequests| -> String {
            let element: Element = (&CwmpVersion::_12, header).try_into().unwrap();
            let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
            let _ = element.0.serialize("cwmp:HoldRequests", &mut writer);
            String::from_utf8(writer.into_inner()).unwrap()
        };
        {
            let header = HoldRequests {
                must_understand: true,
                content: true,
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:HoldRequests soap:mustUnderstand="1">1</cwmp:HoldRequests>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = HoldRequests {
                must_understand: false,
                content: false,
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:HoldRequests soap:mustUnderstand="0">0</cwmp:HoldRequests>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
    }

    #[test]
    fn header_supported_cwmp_versions_serialization() {
        let serialize = |header: SupportedCwmpVersions| -> String {
            let element: Element = (&CwmpVersion::_12, header).try_into().unwrap();
            let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
            let _ = element
                .0
                .serialize("cwmp:SupportedCwmpVersions", &mut writer);
            String::from_utf8(writer.into_inner()).unwrap()
        };
        {
            let header = SupportedCwmpVersions {
                must_understand: Some(true),
                content: "1.0,1.1,1.2".to_string(),
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:SupportedCwmpVersions soap:mustUnderstand="1">1.0,1.1,1.2</cwmp:SupportedCwmpVersions>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = SupportedCwmpVersions {
                must_understand: None,
                content: "1.2".to_string(),
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:SupportedCwmpVersions>1.2</cwmp:SupportedCwmpVersions>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = SupportedCwmpVersions {
                must_understand: Some(false),
                content: "1.0".to_string(),
            };
            let xml = serialize(header);
            let expected = r#"<cwmp:SupportedCwmpVersions soap:mustUnderstand="0">1.0</cwmp:SupportedCwmpVersions>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        }
    }

    #[test]
    fn header_use_cwmp_version_serialization() {
        let serialize = |header: UseCwmpVersion| -> String {
            let element: Element = (&CwmpVersion::_12, header).try_into().unwrap();
            let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
            let _ = element.0.serialize("cwmp:UseCwmpVersion", &mut writer);
            String::from_utf8(writer.into_inner()).unwrap()
        };
        {
            let header = UseCwmpVersion {
                must_understand: true,
                content: "1.2".to_string(),
            };
            let xml = serialize(header);
            let expected =
                r#"<cwmp:UseCwmpVersion soap:mustUnderstand="1">1.2</cwmp:UseCwmpVersion>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        };
        {
            let header = UseCwmpVersion {
                must_understand: false,
                content: "1.1".to_string(),
            };
            let xml = serialize(header);
            let expected =
                r#"<cwmp:UseCwmpVersion soap:mustUnderstand="0">1.1</cwmp:UseCwmpVersion>"#;
            assert!(normalize_xml(&xml) == normalize_xml(&expected));
        }
    }
}
