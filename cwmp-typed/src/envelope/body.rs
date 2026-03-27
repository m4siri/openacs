use crate::cwmp;

#[derive(Debug)]
pub enum EnvelopeBody {
    Rpc(cwmp::rpc::Rpc),
    Fault(super::fault::Fault),
}
