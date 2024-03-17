use crate::asl::types::DynamicValue;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum FailStateErrorField {
    Error(String),
    ErrorPath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum FailStateCauseField {
    Cause(String),
    CausePath(DynamicValue),
}
