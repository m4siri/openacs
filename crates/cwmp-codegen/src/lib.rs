mod schema;
pub use schema::*;

#[cfg(test)]
mod test {

    #[test]
    fn test_schema_parse() {
        use crate::soapenv::EnvelopeType;
        use std::io::Cursor;
        use std::io::{BufReader, BufWriter};
        use xsd_parser_types::quick_xml::{
            DeserializeSync, IoReader, SerializeSync, Writer, XmlReader,
        };

        let request = r#"
            <SOAP-ENV:Envelope xmlns:SOAP-ENV='http://schemas.xmlsoap.org/soap/envelope/'
            xmlns:SOAP-ENC="http://schemas.xmlsoap.org/soap/encoding/" xmlns:xsi=
            "http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd='http://www.w3.org/2001/XMLSchema' xmlns:cwmp="urn:ds1forum-org:cwmp-1-0">
            <SOAP-ENV:Header>
            <cwmp:ID SOAP-ENV:mustUnderstand="1">112</cwmp:ID>
            </SOAP-ENV:Header>
            <SOAP-ENV:Body>
            <cwmp:SetParameterValues>
            <ParameterList SOAP-ENC:arrayType="cwmp:ParameterValueStruct[1]">
            <ParameterValueStruct>
            <Name>Device.WiFi.AccessPoint.10001.Enable</Name>
            <Value xsi:type="xsd:boolean">1</Value>
            </ParameterValueStruct>
            </ParameterList>
            <Parameterkey>bulk_set_1</Parameterkey>
            </cwmp:SetParameterValues>
            </SOAP-ENV:Body>
            </SOAP-ENV:Envelope>
            "#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let doc = EnvelopeType::deserialize(&mut reader).unwrap();
        assert!(doc.header.is_some());
        let header = doc.header.unwrap();
        assert!(&header.any[0].qname().into_inner() == b"cwmp:ID");
    }
}
