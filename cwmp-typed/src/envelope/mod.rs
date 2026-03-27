mod body;
mod fault;
mod header;

use body::EnvelopeBody;
use header::EnvelopeHeaders;

use crate::cwmp::CwmpVersion;

pub struct Element(pub xsd_parser_types::xml::AnyElement);

const NS_SOAPENV: &'static str = "http://schemas.xmlsoap.org/soap/envelope/";
const NS_SOAPENC: &'static str = "http://schemas.xmlsoap.org/soap/encoding/";
const NS_XSI: &'static str = "http://www.w3.org/2001/XMLSchema-instance";
const NS_XSD: &'static str = "http://www.w3.org/2001/XMLSchema";

#[derive(Debug)]
pub struct Envelope {
    pub(crate) cwmp_version: CwmpVersion,
    pub(crate) headers: EnvelopeHeaders,
    pub(crate) body: EnvelopeBody,
}

#[cfg(test)]
mod test {
    use cwmp_xsd_schema::soapenc::{ArrayType, ArrayTypeContent};
    use cwmp_xsd_schema::soapenv::{
        BodyType, BodyTypeContent, EnvelopeType, HeaderType, HeaderTypeContent,
    };
    use cwmp_xsd_schema::tns::{GetRpcMethodsResponseElementType, IdElementType};
    use cwmp_xsd_schema::{
        ParameterValueStructType, SetParameterValuesResponse,
        SetParameterValuesResponseStatusElementType, ValueType,
    };
    use cwmp_xsd_types::XsiType;
    use xsd_parser_types::xml::{AnyAttributes, AnyElement};

    use xsd_parser_types::quick_xml::{
        DeserializeSync, IoReader, SerializeSync, Writer, XmlReader,
    };
}
