use crate::asl::types::{DynamicValue, ExecutionInput};
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Fail {
    #[serde(flatten)]
    pub error: Option<FailStateErrorField>,
    #[serde(flatten)]
    pub cause: Option<FailStateCauseField>,

    // Common fields
    pub comment: Option<String>,
}
