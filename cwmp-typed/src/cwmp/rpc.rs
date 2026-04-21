use super::CwmpVersion;
use super::types::{
    AddObject, AddObjectResponse, DeleteObject, DeleteObjectResponse, Download, DownloadResponse,
    GetParameterAttributesResponse, GetParameterNames, GetParameterNamesResponse,
    GetParameterValuesResponse, GetRPCMethodsResponse, ParameterList, ParameterNames,
    ParameterValueStruct, Reboot, SetParameterAttributesStruct, SetParameterValuesResponse,
};
use crate::error::Error;
use std::borrow::Cow;
use xsd_parser_types::xml::AnyElement;

#[derive(Debug)]
pub struct Rpc(CwmpVersion, RpcMethod);

#[derive(Debug)]
pub enum RpcMethod {
    GetRPCMethods,
    GetParameterNames(GetParameterNames),
    SetParameterValues(SetParameterValues),
    GetParameterValues(GetParameterValues),
    SetParameterAttributes(SetParameterAttributes),
    GetParameterAttributes(ParameterNames),
    AddObject(AddObject),
    DeleteObject(DeleteObject),
    Reboot(Reboot), // SetParameterValuesResponse(SetParameterValuesResponse),
    FactoryReset,
    Download(Download),
    GetRPCMethodsResponse(GetRPCMethodsResponse),
    GetParameterNamesResponse(GetParameterNamesResponse),
    SetParameterValuesResponse(SetParameterValuesResponse),
    GetParameterValuesResponse(GetParameterValuesResponse),
    SetParameterAttributesResponse,
    GetParameterAttributesResponse(GetParameterAttributesResponse),
    AddObjectResponse(AddObjectResponse),
    DeleteObjectResponse(DeleteObjectResponse),
    RebootResponse,
    FactoryResetResponse,
    DownloadResponse(DownloadResponse),
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
                Ok(Rpc::from((CwmpVersion::_10, RpcMethod::GetRPCMethods)))
            }
            RpcBody::GetRpcMethods11(_) => {
                Ok(Rpc::from((CwmpVersion::_11, RpcMethod::GetRPCMethods)))
            }
            RpcBody::GetRpcMethods12(_) => {
                Ok(Rpc::from((CwmpVersion::_12, RpcMethod::GetRPCMethods)))
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
            RpcBody::GetParameterAttributes10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterAttributes(inner.parameter_names.try_into()?),
            ))),
            RpcBody::GetParameterAttributes11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterAttributes(inner.parameter_names.try_into()?),
            ))),
            RpcBody::GetParameterAttributes12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterAttributes(inner.parameter_names.try_into()?),
            ))),
            RpcBody::AddObject10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::AddObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::AddObject11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::AddObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::AddObject12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::AddObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::DeleteObject10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::DeleteObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::DeleteObject11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::DeleteObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::DeleteObject12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::DeleteObject((inner.object_name, inner.parameter_key).into()),
            ))),
            RpcBody::Reboot10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::Reboot(inner.command_key.into()),
            ))),
            RpcBody::Reboot11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::Reboot(inner.command_key.into()),
            ))),
            RpcBody::Reboot12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::Reboot(inner.command_key.into()),
            ))),
            RpcBody::FactoryReset10(_) => {
                Ok(Rpc::from((CwmpVersion::_10, RpcMethod::FactoryReset)))
            }
            RpcBody::FactoryReset11(_) => {
                Ok(Rpc::from((CwmpVersion::_11, RpcMethod::FactoryReset)))
            }
            RpcBody::FactoryReset12(_) => {
                Ok(Rpc::from((CwmpVersion::_12, RpcMethod::FactoryReset)))
            }
            RpcBody::Download10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::Download(inner.into()),
            ))),
            RpcBody::Download11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::Download(inner.into()),
            ))),
            RpcBody::Download12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::Download(inner.into()),
            ))),
            RpcBody::GetRpcMethodsResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetRPCMethodsResponse(inner.method_list.try_into()?),
            ))),
            RpcBody::GetRpcMethodsResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetRPCMethodsResponse(inner.method_list.try_into()?),
            ))),
            RpcBody::GetRpcMethodsResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetRPCMethodsResponse(inner.method_list.try_into()?),
            ))),
            RpcBody::GetParameterNamesResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterNamesResponse(inner.parameter_list.try_into()?),
            ))),
            RpcBody::GetParameterNamesResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterNamesResponse(inner.parameter_list.try_into()?),
            ))),
            RpcBody::GetParameterNamesResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterNamesResponse(inner.parameter_list.try_into()?),
            ))),
            RpcBody::SetParameterValuesResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::SetParameterValuesResponse(SetParameterValuesResponse::new(
                    inner.status,
                )),
            ))),
            RpcBody::SetParameterValuesResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::SetParameterValuesResponse(SetParameterValuesResponse::new(
                    inner.status,
                )),
            ))),
            RpcBody::SetParameterValuesResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::SetParameterValuesResponse(SetParameterValuesResponse::new(
                    inner.status,
                )),
            ))),
            RpcBody::GetParameterValuesResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterValuesResponse(GetParameterValuesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::GetParameterValuesResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterValuesResponse(GetParameterValuesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::GetParameterValuesResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterValuesResponse(GetParameterValuesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::SetParameterAttributesResponse10(_) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::SetParameterAttributesResponse,
            ))),
            RpcBody::SetParameterAttributesResponse11(_) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::SetParameterAttributesResponse,
            ))),
            RpcBody::SetParameterAttributesResponse12(_) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::SetParameterAttributesResponse,
            ))),
            RpcBody::GetParameterAttributesResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::GetParameterAttributesResponse(GetParameterAttributesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::GetParameterAttributesResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::GetParameterAttributesResponse(GetParameterAttributesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::GetParameterAttributesResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::GetParameterAttributesResponse(GetParameterAttributesResponse::new(
                    inner.parameter_list,
                )?),
            ))),
            RpcBody::AddObjectResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::AddObjectResponse(AddObjectResponse::new(
                    inner.instance_number,
                    inner.status,
                )),
            ))),
            RpcBody::AddObjectResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::AddObjectResponse(AddObjectResponse::new(
                    inner.instance_number,
                    inner.status,
                )),
            ))),
            RpcBody::AddObjectResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::AddObjectResponse(AddObjectResponse::new(
                    inner.instance_number,
                    inner.status,
                )),
            ))),
            RpcBody::DeleteObjectResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::DeleteObjectResponse(DeleteObjectResponse::new(inner.status)),
            ))),
            RpcBody::DeleteObjectResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::DeleteObjectResponse(DeleteObjectResponse::new(inner.status)),
            ))),
            RpcBody::DeleteObjectResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::DeleteObjectResponse(DeleteObjectResponse::new(inner.status)),
            ))),
            RpcBody::RebootResponse10(_) => {
                Ok(Rpc::from((CwmpVersion::_10, RpcMethod::RebootResponse)))
            }
            RpcBody::RebootResponse11(_) => {
                Ok(Rpc::from((CwmpVersion::_11, RpcMethod::RebootResponse)))
            }
            RpcBody::RebootResponse12(_) => {
                Ok(Rpc::from((CwmpVersion::_12, RpcMethod::RebootResponse)))
            }
            RpcBody::FactoryResetResponse10(_) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::FactoryResetResponse,
            ))),
            RpcBody::FactoryResetResponse11(_) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::FactoryResetResponse,
            ))),
            RpcBody::FactoryResetResponse12(_) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::FactoryResetResponse,
            ))),
            RpcBody::DownloadResponse10(inner) => Ok(Rpc::from((
                CwmpVersion::_10,
                RpcMethod::DownloadResponse(DownloadResponse::new(
                    inner.status,
                    inner.start_time,
                    inner.complete_time,
                )),
            ))),
            RpcBody::DownloadResponse11(inner) => Ok(Rpc::from((
                CwmpVersion::_11,
                RpcMethod::DownloadResponse(DownloadResponse::new(
                    inner.status,
                    inner.start_time,
                    inner.complete_time,
                )),
            ))),
            RpcBody::DownloadResponse12(inner) => Ok(Rpc::from((
                CwmpVersion::_12,
                RpcMethod::DownloadResponse(DownloadResponse::new(
                    inner.status,
                    inner.start_time,
                    inner.complete_time,
                )),
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

    use super::*;
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
              <soap:Body> <cwmp:GetParameterValues> <ParameterNames soapenc:arrayType="xsd:string[4]">
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
        let RpcMethod::SetParameterAttributes(attr) = rpc.1 else {
            panic!();
        };
        let items = attr.parameter_list.0;
        dbg!(&items);
    }

    #[test]
    fn deserialize_get_rpc_methods_response() {
        let soap = r#"
            <soap:Envelope
                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/"
                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xmlns:cwmp="urn:dslforum-org:cwmp-1-2">

                <soap:Header>
                </soap:Header>

                <soap:Body>
                    <cwmp:GetRPCMethodsResponse>
                        <MethodList soapenc:arrayType="xsd:string[15]">
                            <string>GetRPCMethods</string>
                            <string>SetParameterValues</string>
                            <string>GetParameterNames</string>
                            <string>GetParameterValues</string>
                            <string>GetParameterAttributes</string>
                            <string>SetParameterAttributes</string>
                            <string>AddObject</string>
                            <string>DeleteObject</string>
                            <string>Reboot</string>
                            <string>Download</string>
                            <string>ScheduleInform</string>
                            <string>FaultLog</string>
                            <string>Upload</string>
                            <string>TransferComplete</string>
                            <string>Kick</string>
                        </MethodList>
                    </cwmp:GetRPCMethodsResponse>
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
    fn deserialize_get_parameter_names_response() {
        let soap = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <soapenv:Envelope
              xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
              xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
              xmlns:xsd="http://www.w3.org/2001/XMLSchema"
              xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
              xmlns:cwmp="urn:dslforum-org:cwmp-1-2">

              <soapenv:Header>
                <cwmp:ID soapenv:mustUnderstand="1">1234567890</cwmp:ID>
              </soapenv:Header>

              <soapenv:Body>
                <cwmp:GetParameterNamesResponse>
                  <ParameterList soap:arrayType="cwmp:ParameterInfoStruct[10]">

                    <!-- Root device info object -->
                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.</Name>
                      <Writable>0</Writable>
                    </ParameterInfoStruct>

                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.Manufacturer</Name>
                      <Writable>1</Writable>
                    </ParameterInfoStruct>

                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.ManufacturerOUI</Name>
                      <Writable>0</Writable>
                    </ParameterInfoStruct>

                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.ModelName</Name>
                      <Writable>0</Writable>
                    </ParameterInfoStruct>

                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.SerialNumber</Name>
                      <Writable>0</Writable>
                    </ParameterInfoStruct>

                    <ParameterInfoStruct>
                      <Name>Device.DeviceInfo.SoftwareVersion</Name>
                      <Writable>0</Writable>
                    </ParameterInfoStruct>
                              </ParameterList>
                </cwmp:GetParameterNamesResponse>
              </soapenv:Body>
            </soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_get_parameter_values_response() {
        let soap = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <soapenv:Envelope
              xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
              xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
              xmlns:xsd="http://www.w3.org/2001/XMLSchema"
              xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
              xmlns:cwmp="urn:dslforum-org:cwmp-1-0">

              <soapenv:Header>
                <cwmp:ID soapenv:mustUnderstand="1">1234567891</cwmp:ID>
              </soapenv:Header>

              <soapenv:Body>
                <cwmp:GetParameterValuesResponse>
                  <ParameterList soap:arrayType="cwmp:ParameterValueStruct[12]">

                    <!-- Device Info -->
                    <ParameterValueStruct>
                      <Name>Device.DeviceInfo.Manufacturer</Name>
                      <Value xsi:type="xsd:string">Technicolor</Value>
                    </ParameterValueStruct>

                    <ParameterValueStruct>
                      <Name>Device.DeviceInfo.ManufacturerOUI</Name>
                      <Value xsi:type="xsd:string">00A0BC</Value>
                    </ParameterValueStruct>

                    <ParameterValueStruct>
                      <Name>Device.DeviceInfo.ModelName</Name>
                      <Value xsi:type="xsd:string">TC4400</Value>
                    </ParameterValueStruct>

                    <!-- DNS -->
                    <ParameterValueStruct>
                      <Name>Device.DNS.Client.Server.1.DNSServer</Name>
                      <Value xsi:type="xsd:string">8.8.8.8</Value>
                    </ParameterValueStruct>

                    <!-- Management Server -->
                    <ParameterValueStruct>
                      <Name>Device.ManagementServer.PeriodicInformInterval</Name>
                      <Value xsi:type="xsd:unsignedInt">86400</Value>
                    </ParameterValueStruct>

                  </ParameterList>
                </cwmp:GetParameterValuesResponse>
              </soapenv:Body>

            </soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_get_parameter_attributes_response() {
        let soap = r#"
            <?xml version="1.0" encoding="UTF-8"?>
            <soapenv:Envelope
              xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
              xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
              xmlns:xsd="http://www.w3.org/2001/XMLSchema"
              xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
              xmlns:cwmp="urn:dslforum-org:cwmp-1-2">

              <soapenv:Header>
                <cwmp:ID soapenv:mustUnderstand="1">1234567893</cwmp:ID>
              </soapenv:Header>

              <soapenv:Body>
                <cwmp:GetParameterAttributesResponse>
                  <ParameterList soap:arrayType="cwmp:ParameterAttributeStruct[3]">

                    <!-- Active notification on WAN IP changes -->
                    <ParameterAttributeStruct>
                      <Name>Device.IP.Interface.1.IPv4Address.1.IPAddress</Name>
                      <Notification>2</Notification>
                      <AccessList soap:arrayType="xsd:string[0]" />
                    </ParameterAttributeStruct>

                    <!-- Passive notification on software version -->
                    <ParameterAttributeStruct>
                      <Name>Device.DeviceInfo.SoftwareVersion</Name>
                      <Notification>1</Notification>
                      <AccessList soap:arrayType="xsd:string[0]" />
                    </ParameterAttributeStruct>

                    <!-- Notification off on uptime, with Subscriber access -->
                    <ParameterAttributeStruct>
                      <Name>Device.DeviceInfo.UpTime</Name>
                      <Notification>0</Notification>
                      <AccessList soap:arrayType="xsd:string[1]">
                        <string>Subscriber</string>
                      </AccessList>
                    </ParameterAttributeStruct>

                  </ParameterList>
                </cwmp:GetParameterAttributesResponse>
              </soapenv:Body>

            </soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_add_object_response() {
        let soap = r#"
<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope
  xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
  xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
  xmlns:xsd="http://www.w3.org/2001/XMLSchema"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns:cwmp="urn:dslforum-org:cwmp-1-1">

  <soapenv:Header>
    <cwmp:ID soapenv:mustUnderstand="1">1234567894</cwmp:ID>
  </soapenv:Header>

  <soapenv:Body>
    <cwmp:AddObjectResponse>
      <InstanceNumber>2</InstanceNumber>
      <Status>0</Status>
    </cwmp:AddObjectResponse>
  </soapenv:Body>

</soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_delete_object_response() {
        let soap = r#"
<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope
  xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
  xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
  xmlns:xsd="http://www.w3.org/2001/XMLSchema"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns:cwmp="urn:dslforum-org:cwmp-1-2">

  <soapenv:Header>
    <cwmp:ID soapenv:mustUnderstand="1">1234567895</cwmp:ID>
  </soapenv:Header>

  <soapenv:Body>
    <cwmp:DeleteObjectResponse>
      <Status>0</Status>
    </cwmp:DeleteObjectResponse>
  </soapenv:Body>

</soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }

    #[test]
    fn deserialize_download_response() {
        let soap = r#"
<?xml version="1.0" encoding="UTF-8"?>
<soapenv:Envelope
  xmlns:soapenv="http://schemas.xmlsoap.org/soap/envelope/"
  xmlns:soap="http://schemas.xmlsoap.org/soap/encoding/"
  xmlns:xsd="http://www.w3.org/2001/XMLSchema"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xmlns:cwmp="urn:dslforum-org:cwmp-1-0">
  <soapenv:Header>
    <cwmp:ID soapenv:mustUnderstand="1">1234567896</cwmp:ID>
  </soapenv:Header>
  <soapenv:Body>
    <cwmp:DownloadResponse>
      <Status>1</Status>
      <StartTime>2024-01-01T10:00:00Z</StartTime>
      <CompleteTime>0001-01-01T00:00:00Z</CompleteTime>
    </cwmp:DownloadResponse>
  </soapenv:Body>
</soapenv:Envelope>
"#;

        let cursor = Cursor::new(soap);
        let mut reader = IoReader::new(cursor).with_error_info();
        let envelope = EnvelopeType::deserialize(&mut reader).unwrap();

        let rpc: Rpc = envelope.body.content.try_into().unwrap();
        dbg!(&rpc);
    }
}
