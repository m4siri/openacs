use crate::request::Request as CwmpRequest;
pub use cwmp_schema::{Id, SessionTimeout};
use quick_xml::events::BytesText;
use std::borrow::Cow;
use std::convert::TryFrom;
use xsd_parser_types::xml::{AnyAttributes, AnyElement, Value};

const NS_SOAPENV: &'static str = "http://schemas.xmlsoap.org/soap/envelope/";
const NS_SOAPENC: &'static str = "http://schemas.xmlsoap.org/soap/encoding/";
const NS_XSI: &'static str = "http://www.w3.org/2001/XMLSchema";
const NS_XSD: &'static str = "http://www.w3.org/2001/XMLSchema-instance";

pub struct Envelope {
    pub(crate) headers: EnvelopeHeaders,
    pub(crate) body: EnvelopeBody,
}

pub struct EnvelopeHeaders(Vec<EnvelopeHeader>);

pub enum EnvelopeHeader {
    Id(cwmp_schema::Id),
    // SessionTimeout(cwmp_schema::SessionTimeout),
}

pub enum EnvelopeBody {
    Request(CwmpRequest),
}

impl From<EnvelopeBody> for AnyElement {
    fn from(input: EnvelopeBody) -> Self {
        let body = match input {
            EnvelopeBody::Request(inner) => <CwmpRequest as Into<AnyElement>>::into(inner),
        };
        AnyElement::new()
            .name(Cow::Borrowed(b"soap:Body".as_ref()))
            .child(Value::Element(body))
    }
}

impl From<EnvelopeHeaders> for AnyElement {
    fn from(input: EnvelopeHeaders) -> Self {
        let must_understand = |v: bool| match v {
            true => b"1",
            false => b"0",
        };

        let headers = input
            .0
            .into_iter()
            .map(|header| match header {
                EnvelopeHeader::Id(inner) => AnyElement::new()
                    .name(Cow::Borrowed(b"cwmp:ID".as_ref()))
                    .attribute(
                        Cow::Borrowed(b"soap:mustUnderstand".as_ref()),
                        Cow::Owned(inner.must_understand.to_string().into_bytes()),
                    )
                    .child(Value::Text(BytesText::new(&inner.content).into_owned())),
            })
            .collect::<Vec<AnyElement>>();

        let mut soap_header =
            AnyElement::new().name(Cow::Owned("soap:Header".to_string().into_bytes()));

        for header in headers.into_iter() {
            soap_header = soap_header.child(Value::Element(header));
        }

        soap_header
    }
}

impl From<Envelope> for AnyElement {
    fn from(input: Envelope) -> Self {
        AnyElement::new()
            .name(Cow::Owned("soapenv:Envelope".to_string().into_bytes()))
            .attribute(
                Cow::Owned("xmlns:soapenv".to_string().into_bytes()),
                Cow::Borrowed(NS_SOAPENV.as_bytes()),
            )
            .attribute(
                Cow::Owned("xmlns:soapenc".to_string().into_bytes()),
                Cow::Borrowed(NS_SOAPENC.as_bytes()),
            )
            .attribute(
                Cow::Owned("xmlns:xsi".to_string().into_bytes()),
                Cow::Borrowed(NS_XSI.as_bytes()),
            )
            .attribute(
                Cow::Owned("xmlns:xsd".to_string().into_bytes()),
                Cow::Borrowed(NS_XSD.as_bytes()),
            )
            .child(Value::Element(<EnvelopeBody as Into<AnyElement>>::into(
                input.body,
            )))
            .child(Value::Element(<EnvelopeHeaders as Into<AnyElement>>::into(
                input.headers,
            )))
    }
}

#[cfg(test)]
mod test {

    use crate::envelope::*;
    use crate::request::*;
    use crate::*;
    use quick_xml::Writer;
    use xsd_parser_types::quick_xml::SerializeSync;

    #[test]
    fn envelope_header_serialization() {
        use cwmp_schema::Id;

        let headers = vec![EnvelopeHeader::Id(Id {
            must_understand: false,
            content: "XYZ".to_string(),
        })];
        let header = EnvelopeHeaders(headers);

        let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
        let _ = <EnvelopeHeaders as Into<AnyElement>>::into(header)
            .serialize("soap:Header", &mut writer);

        let xml = String::from_utf8(writer.into_inner()).unwrap();

        println!("{}", xml);
    }

    #[test]
    fn envelope_serialization() {
        use cwmp_schema::Id;

        let headers = vec![EnvelopeHeader::Id(Id {
            must_understand: false,
            content: "XYZ".to_string(),
        })];
        let headers = EnvelopeHeaders(headers);

        let body = EnvelopeBody::Request(Request::SetParameterValues(SetParameterValues {
            parameter_list: ParameterList(vec![ParameterValueStruct {
                name: "PQR".to_string(),
                value: ParameterValueStructValue {
                    type_: "string".to_string(),
                    value: "MNO".to_string(),
                },
            }]),
            parameter_key: "ABC".to_string(),
        }));

        let envelope = Envelope { headers, body };

        let mut writer = Writer::new_with_indent(Vec::new(), b'\t', 1);
        let _ =
            <Envelope as Into<AnyElement>>::into(envelope).serialize("soap:Envelope", &mut writer);

        let xml = String::from_utf8(writer.into_inner()).unwrap();

        println!("{}", xml);
    }
}
