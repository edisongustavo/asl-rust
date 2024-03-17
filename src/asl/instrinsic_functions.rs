use serde::Deserialize;
use serde_json::value::Value;
use std::convert::TryFrom;
use thiserror::Error;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum IntrinsicFunction {}

impl IntrinsicFunction {
    pub fn evaluate(&self, input: &Value) -> Result<Option<Value>, IntrinsicFunctionExecutionError> {
        todo!()
    }
}

#[derive(Error, Debug)]
pub enum IntrinsicFunctionParseError {
    #[error("The string '{0}' does not contain a known intrinsic function")]
    Unknown(String),

    #[error("The intrinsic function '{name}' expected {expected} arguments, but only {actual} arguments were provided")]
    InsufficientArguments { name: String, expected: usize, actual: usize },
}

#[derive(Error, Debug)]
pub enum IntrinsicFunctionExecutionError {
    #[error("Wrong argument types specified to function")]
    WrongArguments, // TODO: Improve error
}


impl TryFrom<&str> for IntrinsicFunction {
    type Error = IntrinsicFunctionParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Err(IntrinsicFunctionParseError::Unknown(value.to_owned()))
    }
}
