use super::CwmpVersion;
use super::types::{
    GetParameterNames, ParameterList, ParameterName, ParameterNames, ParameterValueStruct,
    SetParameterAttributesStruct,
};
use crate::error::Error;
use std::borrow::Cow;
use xsd_parser_types::xml::AnyElement;

#[derive(Debug)]
pub struct Rpc(CwmpVersion, RpcMethod);

#[derive(Debug)]
pub enum RpcMethod {
    GetRpcMethods,
    SetParameterValues(SetParameterValues),
    GetParameterValues(GetParameterValues),
    GetParameterNames(GetParameterNames),
    SetParameterAttributes(SetParameterAttributes),
    // SetParameterValuesResponse(SetParameterValuesResponse),
}

#[derive(Debug)]
pub struct GetParameterValues {
    pub(crate) parameter_names: ParameterNames,
}

#[derive(Debug)]
pub struct SetParameterValues {
    pub(crate) parameter_list: ParameterList<ParameterValueStruct>,
    pub(crate) parameter_key: String,
}

#[derive(Debug)]
pub struct SetParameterAttributes {
    pub(crate) parameter_list: ParameterList<SetParameterAttributesStruct>,
}

impl From<(CwmpVersion, RpcMethod)> for Rpc {
    fn from((version, method): (CwmpVersion, RpcMethod)) -> Self {
        Rpc(version, method)
    }
}

impl TryFrom<cwmp_xsd_schema::soapenv::BodyTypeContent> for Rpc {
    type Error = Error;
    fn try_from(input: cwmp_xsd_schema::soapenv::BodyTypeContent) -> Result<Self, Self::Error> {
        use cwmp_xsd_schema::soapenv::BodyTypeContent as RpcBody;

        match input {
            RpcBody::GetRpcMethods10(_) => {
                Ok(Rpc::from((CwmpVersion::_10, RpcMethod::GetRpcMethods)))
            }
            RpcBody::GetRpcMethods11(_) => {
                Ok(Rpc::from((CwmpVersion::_11, RpcMethod::GetRpcMethods)))
            }
            RpcBody::GetRpcMethods12(_) => {
                Ok(Rpc::from((CwmpVersion::_12, RpcMethod::GetRpcMethods)))
            }
            RpcBody::SetParameterValues10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::SetParameterValues(
                    (inner.parameter_key, inner.parameter_list).try_into()?,
                ),
            ))),
            RpcBody::SetParameterValues11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::SetParameterValues(
                    (inner.parameter_key, inner.parameter_list).try_into()?,
                ),
            ))),
            RpcBody::SetParameterValues12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::SetParameterValues(
                    (inner.parameter_key, inner.parameter_list).try_into()?,
                ),
            ))),
            RpcBody::GetParameterNames10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterNames((inner.parameter_path, inner.next_level).try_into()?),
            ))),
            RpcBody::GetParameterNames11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterNames((inner.parameter_path, inner.next_level).try_into()?),
            ))),
            RpcBody::GetParameterNames12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterNames(
                    (Some(inner.parameter_path), inner.next_level).try_into()?,
                ),
            ))),
            RpcBody::GetParameterValues10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterValues(inner.parameter_names.try_into()?),
            ))),
            RpcBody::GetParameterValues11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterValues(inner.parameter_names.try_into()?),
            ))),
            RpcBody::GetParameterValues12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterValues(inner.parameter_names.try_into()?),
            ))),
            RpcBody::SetParameterAttributes10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::SetParameterAttributes(inner.parameter_list.try_into()?),
            ))),
            RpcBody::SetParameterAttributes11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::SetParameterAttributes(inner.parameter_list.try_into()?),
            ))),
            RpcBody::SetParameterAttributes12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::SetParameterAttributes(inner.parameter_list.try_into()?),
            ))),
            _ => unimplemented!(),
        }
    }
}

impl TryFrom<cwmp_xsd_schema::soapenc::ArrayType> for SetParameterAttributes {
    type Error = Error;
    fn try_from(input: cwmp_xsd_schema::soapenc::ArrayType) -> Result<Self, Self::Error> {
        Ok(Self {
            parameter_list: input.try_into()?,
        })
    }
}

impl TryFrom<cwmp_xsd_schema::soapenc::ArrayType> for GetParameterValues {
    type Error = Error;
    fn try_from(input: cwmp_xsd_schema::soapenc::ArrayType) -> Result<Self, Self::Error> {
        Ok(Self {
            parameter_names: input.try_into()?,
        })
    }
}

impl TryFrom<(String, cwmp_xsd_schema::soapenc::ArrayType)> for SetParameterValues {
    type Error = Error;
    fn try_from(
        (parameter_key, parameter_list): (String, cwmp_xsd_schema::soapenc::ArrayType),
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            parameter_key,
            parameter_list: parameter_list.try_into()?,
        })
    }
}

#[cfg(test)]
mod test {

    use crate::cwmp::CwmpVersion;
    use crate::cwmp::rpc::Rpc;
    use cwmp_xsd_schema::soapenv::EnvelopeType;
    use std::io::Cursor;
    use xsd_parser_types::quick_xml::{DeserializeSync, IoReader, XmlReader};

    #[test]
    fn deserialize_set_parameter_values() {
        let soap = r#"
            <soap:Envelope
                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/"
                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xmlns:cwmp="urn:dslforum-org:cwmp-1-2">
                <soap:Header>
                    <cwmp:ID soap:mustUnderstand="1">uuid:6d6f6e-6579-2d62-616e-6b2d31323334</cwmp:ID>
                </soap:Header>
                <soap:Body>
                    <cwmp:SetParameterValues>
                        <ParameterList soapenc:arrayType="cwmp:ParameterValueStruct[3]">
                            <ParameterValueStruct>
                                <Name>InternetGatewayDevice.LANDevice.1.WLANConfiguration.1.SSID</Name>
                                <Value xsi:type="xsd:string">MyHomeWiFi_5G</Value>
                            </ParameterValueStruct>
                            <ParameterValueStruct>
                                <Name>InternetGatewayDevice.LANDevice.1.WLANConfiguration.1.KeyPassphrase</Name>
                                <Value xsi:type="xsd:string">SuperSecurePass123!</Value>
                            </ParameterValueStruct>
                            <ParameterValueStruct>
                                <Name>InternetGatewayDevice.LANDevice.1.WLANConfiguration.1.Enable</Name>
                                <Value xsi:type="xsd:boolean">1</Value>
                            </ParameterValueStruct>
                        </ParameterList>
                        <ParameterKey>config-20260203-1745</ParameterKey>
                    </cwmp:SetParameterValues>
                </soap:Body>
            </soap:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_get_parameter_values() {
        let soap = r#"
            <soap:Envelope
                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/"
                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xmlns:cwmp="urn:dslforum-org:cwmp-1-0">
              <soap:Header>
                <cwmp:ID soap:mustUnderstand="1">123456789</cwmp:ID>
              </soap:Header>
              <soap:Body>
                <cwmp:GetParameterValues>
                  <ParameterNames soapenc:arrayType="xsd:string[4]">
                    <string>Device.DeviceInfo.Manufacturer</string>
                    <string>Device.</string>
                    <string>Device.*.AccessPoint.</string>
                    <string>Device.WiFi.SSID.*.Stats</string>
                  </ParameterNames>
                </cwmp:GetParameterValues>
              </soap:Body>
            </soap:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_set_parameter_attributes() {
        let soap = r#"
            <soap:Envelope
                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                xmlns:soap-enc="http://schemas.xmlsoap.org/soap/encoding/"
                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xmlns:cwmp="urn:dslforum-org:cwmp-1-0">

                <soap:Header>
                    <cwmp:ID soap:mustUnderstand="1">12345</cwmp:ID>
                </soap:Header>

                <soap:Body>
                    <cwmp:SetParameterAttributes>
                        <ParameterList soap-enc:arrayType="cwmp:SetParameterAttributesStruct[2]">

                            <SetParameterAttributesStruct>
                                <Name>InternetGatewayDevice.WANDevice.1.WANConnectionDevice.1.WANIPConnection.1.ExternalIPAddress</Name>
                                <NotificationChange>true</NotificationChange>
                                <Notification>2</Notification>
                                <AccessListChange>true</AccessListChange>
                                <AccessList soap-enc:arrayType="xsd:string[1]">
                                    <string>Subscriber</string>
                                </AccessList>
                            </SetParameterAttributesStruct>

                            <SetParameterAttributesStruct>
                                <Name>InternetGatewayDevice.ManagementServer.PeriodicInformInterval</Name>
                                <NotificationChange>true</NotificationChange>
                                <Notification>1</Notification>
                                <AccessListChange>false</AccessListChange>
                                <AccessList soap-enc:arrayType="xsd:string[0]"/>
                            </SetParameterAttributesStruct>

                        </ParameterList>
                    </cwmp:SetParameterAttributes>
                </soap:Body>
            </soap:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }
}
