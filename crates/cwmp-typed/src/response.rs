use quick_xml::events::BytesText;
use std::borrow::Cow;
use xsd_parser_types::xml::{AnyAttributes, AnyElement, Value};

#[derive(Debug)]
pub enum Response {
    SetParameterValuesResponse(SetParameterValuesResponse),
    NoContent,
}

#[derive(Debug)]
pub struct SetParameterValuesResponse {
    pub status: bool,
}
