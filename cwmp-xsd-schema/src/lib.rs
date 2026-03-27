mod schema;
pub use schema::*;

#[cfg(test)]
mod test {

    use crate::soapenc::ArrayTypeContent;
    use crate::soapenv::{BodyTypeContent, Envelope, HeaderTypeContent};
    use std::io::Cursor;
    use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

    use crate::schema::cwmp_13::SessionTimeout;
    use crate::schema::tns::IdElementType;

    fn get_envelope_with_only_header(ns: &str) -> String {
        format!(
            r#"
    <soap:Envelope
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:cwmp="{}">
        <soap:Header>
                <cwmp:ID soap:mustUnderstand="1">1234</cwmp:ID>
                <cwmp:SessionTimeout soap:mustUnderstand="0">40</cwmp:SessionTimeout>
        </soap:Header>
        <soap:Body>
        </soap:Body>
    </soap:Envelope>
            "#,
            ns
        )
    }
    fn get_envelope_with_body(ns: &str) -> String {
        format!(
            r#"
<soap:Envelope
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:cwmp="{}">
  <soap:Header/>
  <soap:Body>
    <cwmp:GetRPCMethodsResponse>
      <MethodList soap-enc:arrayType="xsd:string[2]"
                  xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/">
        <string>GetRPCMethods</string>
        <string>SetParameterValues</string>
      </MethodList>
    </cwmp:GetRPCMethodsResponse>
  </soap:Body>
</soap:Envelope>"#,
            ns
        )
    }

    fn deserialize_envelope(xml: String) -> Envelope {
        let cursor = Cursor::new(xml);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = Envelope::deserialize(&mut reader).unwrap();
        env
    }

    #[test]
    fn header_deserialization_cwmp_12() {
        let request = get_envelope_with_only_header("urn:dslforum-org:cwmp-1-2");
        let env = deserialize_envelope(request);
        let headers = env.header.unwrap().content;
        match &headers[0] {
            HeaderTypeContent::Id12(IdElementType {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, true);
                assert_eq!(content, "1234");
            }
            _ => unreachable!(),
        };
        match &headers[1] {
            HeaderTypeContent::SessionTimeout12(SessionTimeout {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, Some(false));
                assert_eq!(*content, 40);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn header_deserialization_cwmp_11() {
        let request = get_envelope_with_only_header("urn:dslforum-org:cwmp-1-1");
        let env = deserialize_envelope(request);
        let headers = env.header.unwrap().content;
        match &headers[0] {
            HeaderTypeContent::Id11(IdElementType {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, true);
                assert_eq!(content, "1234");
            }
            _ => unreachable!(),
        };
        assert!(headers.len() == 1);
    }

    #[test]
    fn header_deserialization_cwmp_10() {
        let request = get_envelope_with_only_header("urn:dslforum-org:cwmp-1-0");
        let env = deserialize_envelope(request);
        let headers = env.header.unwrap().content;
        match &headers[0] {
            HeaderTypeContent::Id10(IdElementType {
                must_understand,
                content,
            }) => {
                assert_eq!(*must_understand, true);
                assert_eq!(content, "1234");
            }
            _ => unreachable!(),
        };
        assert!(headers.len() == 1);
    }

    #[test]
    fn body_deserialization_cwmp_10() {
        let request = get_envelope_with_body("urn:dslforum-org:cwmp-1-0");
        let env = deserialize_envelope(request);
        let content = &env.body.content;

        let BodyTypeContent::GetRpcMethodsResponse10(inner) = content else {
            panic!()
        };

        assert!(inner.method_list.array_type == Some("xsd:string[2]".to_string()));
        assert!(inner.method_list.content.len() == 2);
        let ArrayTypeContent::String(elem) = &inner.method_list.content[0] else {
            panic!()
        };
        assert!(*elem == "GetRPCMethods".to_string());
        let ArrayTypeContent::String(elem) = &inner.method_list.content[1] else {
            panic!()
        };
        assert!(*elem == "SetParameterValues".to_string());
    }

    #[test]
    fn body_deserialization_cwmp_11() {
        let request = get_envelope_with_body("urn:dslforum-org:cwmp-1-1");
        let env = deserialize_envelope(request);
        let content = &env.body.content;

        let BodyTypeContent::GetRpcMethodsResponse11(inner) = content else {
            panic!()
        };

        assert!(inner.method_list.array_type == Some("xsd:string[2]".to_string()));
        assert!(inner.method_list.content.len() == 2);
        let ArrayTypeContent::String(elem) = &inner.method_list.content[0] else {
            panic!()
        };
        assert!(*elem == "GetRPCMethods".to_string());
        let ArrayTypeContent::String(elem) = &inner.method_list.content[1] else {
            panic!()
        };
        assert!(*elem == "SetParameterValues".to_string());
    }
    #[test]
    fn body_deserialization_cwmp_12() {
        let request = get_envelope_with_body("urn:dslforum-org:cwmp-1-2");
        let env = deserialize_envelope(request);
        let content = &env.body.content;

        let BodyTypeContent::GetRpcMethodsResponse12(inner) = content else {
            panic!()
        };

        assert!(inner.method_list.array_type == Some("xsd:string[2]".to_string()));
        assert!(inner.method_list.content.len() == 2);
        let ArrayTypeContent::String(elem) = &inner.method_list.content[0] else {
            panic!()
        };
        assert!(*elem == "GetRPCMethods".to_string());
        let ArrayTypeContent::String(elem) = &inner.method_list.content[1] else {
            panic!()
        };
        assert!(*elem == "SetParameterValues".to_string());
    }

    fn parameter_list_deserialization() {
        let request = r#"
<soap:Envelope
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:cwmp="{}">
  <soap:Header/>
  <soap:Body>
    <cwmp:GetRPCMethodsResponse>
      <MethodList soap-enc:arrayType="xsd:string[2]"
                  xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/">
        <string>GetRPCMethods</string>
        <string>SetParameterValues</string>
      </MethodList>
    </cwmp:GetRPCMethodsResponse>
  </soap:Body>
</soap:Envelope>
"#;
    }

    #[test]
    fn soap_fault_parse_cwmp10() {
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
        let env = Envelope::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }

    #[test]
    fn soap_set_parameter_values_parse_cwmp10() {
        let request = r#"
<soap:Envelope
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:cwmp="urn:dslforum-org:cwmp-1-0"
    xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <soap:Header>
        <cwmp:ID soap:mustUnderstand="0">XYZ</cwmp:ID>
    </soap:Header>
    <soap:Body>
        <cwmp:SetParameterValues>
            <ParameterList soapenc:arrayType="cwmp:ParameterValueStruct[1]">
                <ParameterValueStruct>
                    <Name>PQR</Name>
                    <Value xsi:type="xsd:boolean">true</Value>
                </ParameterValueStruct>
            </ParameterList>
            <ParameterKey>ABC</ParameterKey>
        </cwmp:SetParameterValues>
    </soap:Body>
</soap:Envelope>
"#;

        let cursor = Cursor::new(request);
        let mut reader = IoReader::new(cursor).with_error_info();
        let env = Envelope::deserialize(&mut reader).unwrap();
        dbg!(&env);
    }
}
