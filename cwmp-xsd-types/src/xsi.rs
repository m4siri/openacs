use std::borrow::Cow;
use xsd_parser_types::misc::{Namespace, NamespacePrefix};
use xsd_parser_types::quick_xml::{
    BytesStart, DeserializeBytes, DeserializeHelper, Error, ErrorKind, QName, ResolveResult,
};

#[derive(Debug)]
pub struct XsiType {
    pub prefix: String,
    pub namespace: String,
    pub type_: String,
}

impl DeserializeBytes for XsiType {
    // TODO - could handle this better, not sure if the value will always be
    // prefixed with a namespace such as `xsi:type="xsd:string"`.
    fn deserialize_bytes(helper: &mut DeserializeHelper, bytes: &[u8]) -> Result<Self, Error> {
        let value = QName(bytes);
        let Some(prefix) = value.prefix() else {
            return Err(Error {
                kind: ErrorKind::UnknownOrInvalidValue(
                    "Expected type to have a namespace prefix.".into(),
                ),
                elements: None,
                position: None,
            });
        };

        let (resolve_result, local_name) = helper.resolve(value, true);
        match resolve_result {
            ResolveResult::Bound(ns) => Ok(XsiType {
                prefix: String::from_utf8_lossy(prefix.into_inner()).into_owned(),
                namespace: String::from_utf8_lossy(ns.into_inner()).into_owned(),
                type_: String::from_utf8_lossy(local_name.into_inner()).into_owned(),
            }),
            ResolveResult::Unbound => Err(Error {
                kind: ErrorKind::UnknownOrInvalidValue(
                    "Expected type to have a namespace prefix.".into(),
                ),
                elements: None,
                position: None,
            }),
            ResolveResult::Unknown(_) => Err(Error {
                kind: ErrorKind::UnknownOrInvalidValue(
                    "Specified prefix was not found in scope.".into(),
                ),
                elements: None,
                position: None,
            }),
        }
    }
}
