use serde::Deserialize;
use crate::asl::types::MyJsonPath;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum FailStateErrorField {
    Error(String),
    ErrorPath(MyJsonPath)
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum FailStateCauseField {
    Cause(String),
    CausePath(MyJsonPath)
}
