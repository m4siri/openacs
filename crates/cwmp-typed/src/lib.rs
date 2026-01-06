use cwmp_schema::soapenv::EnvelopeType;
use cwmp_schema::tns::GetParameterValuesResponse as SchemaGetParameterValuesResponse;
use xsd_parser_types::quick_xml::{
    DeserializeSync, SerializeSync, SliceReader, WithDeserializer, Writer, XmlReader,
};
use xsd_parser_types::xml::AnyElement;

pub fn parse_message(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = SliceReader::new(message.trim());

    {
        let mut doc = EnvelopeType::deserialize(&mut reader)?;

        dbg!(&doc);

        for rpc in doc.body.any.into_iter().take(1) {
            let method_name = rpc.qname().local_name();
            match method_name.as_ref() {
                b"GetParameterValuesResponse" => {
                    let _ = parse_get_parameter_values_response(rpc);
                }
                t @ _ => todo!(),
            };
        }
    }

    Ok(())
}

fn parse_get_parameter_values_response(
    element: AnyElement,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut writer = Writer::new(&mut buffer);
    element.serialize("GetParameterValuesResponse", &mut writer)?;

    let xml = String::from_utf8(buffer)?;
    let mut reader = SliceReader::new(xml.as_str());
    let schema_response = SchemaGetParameterValuesResponse::deserialize(&mut reader)?;

    let response: GetParameterValuesResponse = schema_response.try_into()?;

    Ok(())
}

impl TryInto<GetParameterValuesResponse> for SchemaGetParameterValuesResponse {
    type Error = Box<dyn std::error::Error>;

    fn try_into(self) -> Result<GetParameterValuesResponse, Self::Error> {
        let mut parameter_list: Vec<ParameterValue> = Vec::new();
        let content = self.parameter_list.content;
        if let Some(content) = content {
            let elements = content.any;
            for element in elements.into_iter() {
                // https://github.com/Bergmann89/xsd-parser/blob/master/xsd-parser-types/src/xml/value.rs#L13
                // Value has 4 different types, I'm not sure if type is ever a Element in this case.
                // so will be handling only Text()
            }
        }
        Ok(GetParameterValuesResponse { parameter_list })
    }
}

#[derive(Debug)]
pub struct GetParameterValuesResponse {
    pub parameter_list: Vec<ParameterValue>,
}

#[derive(Debug)]
struct ParameterValue {
    name: String,
    value: String,
}

#[cfg(test)]
mod tests {

    use crate::*;

    #[test]
    fn test_message_parse() {
        let msg = r#"
            <?xml version="1.0" encoding="UTF-8"?>
<soap:Envelope 
    xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
    xmlns:cwmp="urn:dslforum-org:cwmp-1-0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xmlns:xsd="http://www.w3.org/2001/XMLSchema">
    <soap:Header>
        <cwmp:ID soap:mustUnderstand="1">1234567890</cwmp:ID>
    </soap:Header>
    <soap:Body>
        <cwmp:GetParameterValuesResponse>
            <ParameterList soap:arrayType="cwmp:ParameterValueStruct[1]">
                <ParameterValueStruct>
                    <Name>InternetGatewayDevice.DeviceInfo.Manufacturer</Name>
                    <Value xsi:type="xsd:string">ExampleManufacturer</Value>
                </ParameterValueStruct>
            </ParameterList>
        </cwmp:GetParameterValuesResponse>
    </soap:Body>
</soap:Envelope>
        "#;
        let msg = parse_message(&msg);
    }
}
