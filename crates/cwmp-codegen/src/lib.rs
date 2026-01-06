mod schema;
pub use schema::*;

#[cfg(test)]
mod test {

    #[test]
    fn test_schema_parse() {
        use crate::IdElementType;
        use crate::SessionTimeoutElementType;
        use crate::soapenv::{EnvelopeType, HeaderTypeContent};
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
<soap:Envelope
xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
xmlns:cwmp="urn:dslforum-org:cwmp-1-2">
    <soap:Header>
            <cwmp:ID soap:mustUnderstand="1">1234</cwmp:ID>
            <cwmp:SessionTimeout soap:mustUnderstand="0">40</cwmp:SessionTimeout>
    </soap:Header>
    <soap:Body>
        <cwmp:Action>
        <argument>value</argument>
        </cwmp:Action>
    </soap:Body>
</soap:Envelope>
"#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = EnvelopeType::deserialize(&mut reader).unwrap();
        let header = env.header.unwrap();

        match &header.content[0] {
            HeaderTypeContent::Id(IdElementType {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, true);
                assert_eq!(content, "1234");
            }
            _ => panic!("Expected HeaderTypeContent::Id variant"),
        };
        match &header.content[1] {
            HeaderTypeContent::SessionTimeout(SessionTimeoutElementType {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, Some(false));
                assert_eq!(*content, 40);
            }
            _ => panic!("Expected HeaderTypeContent::SessionTimeout variant"),
        }
    }
}
