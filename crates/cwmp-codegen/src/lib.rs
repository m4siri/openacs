mod schema;
pub use schema::*;

#[cfg(test)]
mod test {

    #[test]
    fn test_header_parse() {
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
    #[test]
    fn test_body_parse() {
        use crate::soapenv::EnvelopeType;
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
<soap:Envelope
xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
xmlns:cwmp="urn:dslforum-org:cwmp-1-0">
    <soap:Header>
    <cwmp:ID soap:mustUnderstand="1">1234</cwmp:ID>
    </soap:Header>
    <soap:Body>
        <soap:Fault>
            <faultcode>Client</faultcode>
            <faultstring>CWMP fault</faultstring>
            <detail>
                <cwmp:Fault>
                    <FaultCode>9003</FaultCode>
                    <FaultString>Invalid arguments</FaultString>
                    <SetParameterValuesFault>
                        <ParameterName>
                        Device.Time.NTPServer1
                        </ParameterName>
                        <FaultCode>9007</FaultCode>
                        <FaultString>Invalid IP Address</FaultString>
                    </SetParameterValuesFault>
                    <SetParameterValuesFault>
                        <ParameterName>
                        Device.Time.LocalTimeZoneName
                        </ParameterName>
                        <FaultCode>9007</FaultCode>
                        <FaultString>String too long</FaultString>
                    </SetParameterValuesFault>
                </cwmp:Fault>
            </detail>
        </soap:Fault>
    </soap:Body>
</soap:Envelope>
"#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = EnvelopeType::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }

    #[test]
    fn test_body_rpc_parse() {
        use crate::soapenv::EnvelopeType;
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
<soap-env:Envelope xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/"
xmlns:soap-env="http://schemas.xmlsoap.org/soap/envelope/"
xmlns:xsd="http://www.w3.org/2001/XMLSchema"
xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xmlns:cwmp="urn:dslforum-org:cwmp-1-2">
<soap-env:Header>
<cwmp:ID soap-env:mustUnderstand="1">0</cwmp:ID>
</soap-env:Header>
<soap-env:Body>
<cwmp:GetParameterNames>
<ParameterPath>Object.</ParameterPath>
<NextLevel>0</NextLevel>
</cwmp:GetParameterNames>
</soap-env:Body>
</soap-env:Envelope>
        "#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = EnvelopeType::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }

    #[test]
    fn test_methodlist_parse() {
        use crate::soapenv::EnvelopeType;
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:cwmp="urn:dslforum-org:cwmp-1-2">
  <soap:Header/>
  <soap:Body>
    <cwmp:GetRPCMethodsResponse>
      <MethodList soap-enc:arrayType="xsd:string[7]"
                  xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/">
        <string>GetRPCMethods</string>
        <string>SetParameterValues</string>
        <string>GetParameterValues</string>
        <string>GetParameterNames</string>
        <string>SetParameterAttributes</string>
        <string>GetParameterAttributes</string>
        <string>AddObject</string>
        <string>DeleteObject</string>
        <string>Download</string>
        <string>Upload</string>
        <string>Reboot</string>
        <string>FactoryReset</string>
      </MethodList>
    </cwmp:GetRPCMethodsResponse>
  </soap:Body>
</soap:Envelope>

        "#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = EnvelopeType::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }
    #[test]
    fn test_set_parameter_values_type() {
        use crate::soapenv::EnvelopeType;
        use std::io::Cursor;
        use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

        let request = r#"
            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
               xmlns:cwmp="urn:dslforum-org:cwmp-1-0">
  <soap:Header/>
  <soap:Body>
    <cwmp:SetParameterValues>
      <ParameterList soap-enc:arrayType="cwmp:ParameterValueStruct[2]"
                     xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/">
        <ParameterValueStruct>
          <Name>InternetGatewayDevice.ManagementServer.URL</Name>
          <Value xsi:type="xsd:string">https://acs.example.com:7547</Value>
        </ParameterValueStruct>
        <ParameterValueStruct>
          <Name>InternetGatewayDevice.ManagementServer.PeriodicInformEnable</Name>
          <Value xsi:type="xsd:boolean">true</Value>
        </ParameterValueStruct>
      </ParameterList>
      <ParameterKey>BOOTSTRAP_2026-01-11T14:11+05:45</ParameterKey>
    </cwmp:SetParameterValues>
  </soap:Body>
</soap:Envelope>

        "#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = EnvelopeType::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }
}
