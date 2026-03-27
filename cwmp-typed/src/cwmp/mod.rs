pub mod rpc;
pub mod types;

#[derive(Debug)]
pub enum CwmpVersion {
    _10,
    _11,
    _12,
}

pub mod header {
    pub type Id = cwmp_xsd_schema::tns::IdElementType;
    pub type HoldRequests = cwmp_xsd_schema::tns::HoldRequestsElementType;
    pub type SessionTimeout = cwmp_xsd_schema::cwmp_13::SessionTimeoutElementType;

    // would be nice if we implement a custom deserializer for this
    // so we can parse the recieved string version values into a Vec<CwmpVersion>
    pub type SupportedCwmpVersions = cwmp_xsd_schema::cwmp_14::SupportedCwmpVersionsElementType;
    pub type UseCwmpVersion = cwmp_xsd_schema::cwmp_14::UseCwmpVersionElementType;
}
