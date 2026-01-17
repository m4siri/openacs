use crate::request::Request as CwmpRequest;
use crate::response::Response as CwmpResponse;
use quick_xml::events::BytesText;
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyElement, Value};

const NS_SOAPENV: &'static str = "http://schemas.xmlsoap.org/soap/envelope/";
const NS_SOAPENC: &'static str = "http://schemas.xmlsoap.org/soap/encoding/";
const NS_XSI: &'static str = "http://www.w3.org/2001/XMLSchema-instance";
const NS_XSD: &'static str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug)]
pub struct Envelope {
    pub(crate) headers: EnvelopeHeaders,
    pub(crate) body: EnvelopeBody,
}

#[derive(Debug)]
pub struct EnvelopeHeaders(Vec<EnvelopeHeader>);

#[derive(Debug)]
pub enum EnvelopeHeader {
    Id(cwmp_schema::Id),
    // SessionTimeout(cwmp_schema::SessionTimeout),
}

#[derive(Debug)]
pub enum EnvelopeBody {
    Request(CwmpRequest),
    Response(CwmpResponse),
}

impl TryFrom<EnvelopeBody> for AnyElement {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: EnvelopeBody) -> Result<Self, Self::Error> {
        let body: AnyElement = match input {
            EnvelopeBody::Request(inner) => inner.into(),
            EnvelopeBody::Response(_) => unimplemented!(), // TODO: change to a known error type after we have one
                                                           // currently this library is expected to be used as acs
        };
        Ok(AnyElement::new()
            .name(Cow::Borrowed(b"soap:Body".as_ref()))
            .child(Value::Element(body)))
    }
}

impl From<EnvelopeHeaders> for AnyElement {
    fn from(input: EnvelopeHeaders) -> Self {
        let bs = |v: bool| if v { "1" } else { "0" };

        let headers = input
            .0
            .into_iter()
            .map(|header| match header {
                EnvelopeHeader::Id(inner) => AnyElement::new()
                    .name(Cow::Borrowed(b"cwmp:ID".as_ref()))
                    .attribute(
                        Cow::Borrowed(b"soap:mustUnderstand".as_ref()),
                        Cow::Borrowed(bs(inner.must_understand).as_ref()),
                    )
                    .child(Value::Text(BytesText::new(&inner.content).into_owned())),
            })
            .collect::<Vec<AnyElement>>();

        let mut soap_header = AnyElement::new().name(Cow::Borrowed(b"soap:Header".as_ref()));

        for header in headers.into_iter() {
            soap_header = soap_header.child(Value::Element(header));
        }

        soap_header
    }
}

impl TryFrom<Envelope> for AnyElement {
    type Error = Box<dyn std::error::Error>;

    fn try_from(input: Envelope) -> Result<Self, Self::Error> {
        Ok(AnyElement::new()
            .name(Cow::Borrowed(b"soap:Envelope".as_ref()))
            .attribute(
                Cow::Borrowed(b"xmlns:soap".as_ref()),
                Cow::Borrowed(NS_SOAPENV.as_bytes()),
            )
            .attribute(
                Cow::Borrowed(b"xmlns:soapenc".as_ref()),
                Cow::Borrowed(NS_SOAPENC.as_bytes()),
            )
            .attribute(
                Cow::Borrowed(b"xmlns:xsd".as_ref()),
                Cow::Borrowed(NS_XSD.as_bytes()),
            )
            .attribute(
                Cow::Borrowed(b"xmlns:xsi".as_ref()),
                Cow::Borrowed(NS_XSI.as_bytes()),
            )
            .child(Value::Element(<EnvelopeHeaders as Into<AnyElement>>::into(
                input.headers,
            )))
            .child(Value::Element(
                <EnvelopeBody as TryInto<AnyElement>>::try_into(input.body)?,
            )))
    }
}

impl From<cwmp_schema::soapenv::HeaderType> for EnvelopeHeaders {
    fn from(input: cwmp_schema::soapenv::HeaderType) -> Self {
        use cwmp_schema::soapenv::HeaderTypeContent as CwmpHeader;
        let headers = input
            .content
            .into_iter()
            .filter_map(|header| match header {
                CwmpHeader::Id(inner) => Some(EnvelopeHeader::Id(inner)),
                _ => None,
            })
            .collect::<Vec<EnvelopeHeader>>();

        EnvelopeHeaders(headers)
    }
}

impl From<cwmp_schema::soapenv::BodyType> for EnvelopeBody {
    fn from(input: cwmp_schema::soapenv::BodyType) -> Self {
        use crate::response::SetParameterValuesResponse;
        use cwmp_schema::soapenv::BodyTypeContent;

        let response = input
            .content
            .into_iter()
            .filter_map(|content| match content {
                BodyTypeContent::SetParameterValuesResponse(inner) => {
                    use cwmp_schema::cwmp_12::SetParameterValuesResponseStatusElementType as Status;
                    let status = matches!(inner.status, Status::_1);
                    Some(CwmpResponse::SetParameterValuesResponse(
                        SetParameterValuesResponse { status },
                    ))
                }
                _ => None,
            })
            .next()
            // any element that we do not recognize is ignored by the deserializer
            // however, it does not insert a identifier to indicate presence of no recognizable body
            // elements are recognized based on their schema definitions in cwmp_10, cwmp_11 & cwmp12.
            // we're taking the first recognized body element just in case there is presence of many.
            // if thats the case then we should probably invalidate the request but im really
            // not sure since we can just ignore unknown elements.
            .unwrap_or(CwmpResponse::NoContent);

        EnvelopeBody::Response(response)
    }
}

impl From<cwmp_schema::soapenv::Envelope> for Envelope {
    fn from(input: cwmp_schema::soapenv::Envelope) -> Self {
        let body: crate::envelope::EnvelopeBody = input.body.into();
        // TODO: change cwmp-schema/build.rs to change Option<T> to T for header
        // i mean all request must have a ID header don't they ?
        let headers: crate::envelope::EnvelopeHeaders = input.header.unwrap().into();

        Envelope { headers, body }
    }
}
#[cfg(test)]
mod test {

    use crate::envelope::*;
    use crate::request::*;
    use crate::*;
    use quick_xml::Writer;
    use xsd_parser_types::quick_xml::SerializeSync;

    fn normalize_xml(s: &str) -> String {
        s.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("")
    }

    #[test]
    fn envelope_set_parameter_values_serialization() {
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
        let _ = <Envelope as TryInto<AnyElement>>::try_into(envelope)
            .unwrap()
            .serialize("soap:Envelope", &mut writer);

        let xml = String::from_utf8(writer.into_inner()).unwrap();
        let expected = r#"
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <soap:Header>
        <cwmp:ID soap:mustUnderstand="0">XYZ</cwmp:ID>
    </soap:Header>
    <soap:Body>
        <cwmp:SetParameterValues>
            <ParameterList soapenc:arrayType="cwmp:ParameterValueStruct[1]">
                <ParameterValueStruct>
                    <Name>PQR</Name>
                    <Value xsi:type="xsd:string">MNO</Value>
                </ParameterValueStruct>
            </ParameterList>
            <ParameterKey>ABC</ParameterKey>
        </cwmp:SetParameterValues>
    </soap:Body>
</soap:Envelope>"#;

        assert_eq!(normalize_xml(&xml), normalize_xml(expected));
    }

    #[test]
    fn schema_envelope_type_to_envelope() {
        use crate::envelope::Envelope;
        use crate::response::Response as CwmpResponse;
        use cwmp_schema::soapenv::EnvelopeType;
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
<soap-env:Envelope
        xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/"
        xmlns:soap-env="http://schemas.xmlsoap.org/soap/envelope/"
        xmlns:xsd="http://www.w3.org/2001/XMLSchema"
        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
        xmlns:cwmp="urn:dslforum-org:cwmp-1-2">
    <soap-env:Header>
        <cwmp:ID soap-env:mustUnderstand="1">0</cwmp:ID>
    </soap-env:Header>
    <soap-env:Body>
        <cwmp:SetParameterValuesResponse>
            <Status>1</Status>
        </cwmp:SetParameterValuesResponse>
    </soap-env:Body>
</soap-env:Envelope>
"#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let inner_env = EnvelopeType::deserialize(&mut reader).unwrap();

        let env: Envelope = inner_env.into();

        let EnvelopeHeader::Id(header) = &env.headers.0[0] else {
            panic!()
        };
        assert!(header.must_understand);
        assert!(header.content == "0".to_string());

        let EnvelopeBody::Response(CwmpResponse::SetParameterValuesResponse(response)) = &env.body
        else {
            panic!()
        };
        assert!(response.status);
    }
}
