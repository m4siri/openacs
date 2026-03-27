#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Header '{0}' is incompatible with the cwmp version.")]
    UnsupportedHeaderVersion(String),
    #[error("Rpc '{0}' is incompatible with the cwmp version.")]
    UnsupportedRpcVersion(String),
    #[error("{0}")]
    InvalidParameterName(String),
    #[error("Expected attribute {0}")]
    MissingAttribute(String),
    #[error("{0}")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Invalid value: {0}")]
    InvalidValue(String),
    #[error("Rpc is incompatible with the cwmp version.")]
    RpcVersionMismatch,
    #[error("Rpc does not exist.")]
    UnknownRpc,
}
