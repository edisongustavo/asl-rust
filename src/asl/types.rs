use crate::asl::instrinsic_functions::{IntrinsicFunction, IntrinsicFunctionExecutionError};
use serde::Deserialize;
use serde_json::Value;
use serde_json_path::{JsonPath, ParseError as JsonPathParseError};
use serde_with::DeserializeFromStr;
use std::str::FromStr;
use thiserror::Error;

// TODO: Implement Timestamp
pub type Timestamp = String;

// TODO: Implement JSONPath
// pub type MyJsonPath = JsonPath;
#[derive(DeserializeFromStr, Debug, PartialEq, Eq)]
pub enum DynamicValue {
    ValueJsonPath(JsonPath),
    Context(JsonPath),
    Value(Value),
    IntrinsicFunction(IntrinsicFunction),
    Identity,
}
#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ValueJsonPath {
    Foo(Value),
}

pub trait Evaluate {
    fn evaluate(&self, input: &Value) -> Option<Value>;
}
impl Evaluate for JsonPath {
    fn evaluate(&self, input: &Value) -> Option<Value> {
        let results = self.query(input);
        results.first().map(|v| v.clone())
    }
}

#[derive(Error, Debug)]
pub enum DynamicValueEvaluateError {
    #[error("error is IntrinsicFunctionExecutionError")] //TODO: specific better error
    IntrinsicFunctionExecutionError(IntrinsicFunctionExecutionError),
}

impl DynamicValue {
    pub fn evaluate(
        &self,
        input: &ExecutionInput,
    ) -> Result<Option<Value>, DynamicValueEvaluateError> {
        match self {
            DynamicValue::ValueJsonPath(path) => Ok(path.evaluate(input.value)),
            // DynamicValue::Context(path) => path.evaluate(input.context.into()),
            DynamicValue::Context(path) => todo!(),
            DynamicValue::Value(val) => Ok(Some(val.clone())), //TODO: remove clone()
            DynamicValue::IntrinsicFunction(function) => {
                function
                    .evaluate(input.value)
                    .map_err(|e| DynamicValueEvaluateError::IntrinsicFunctionExecutionError(e))
            },
            DynamicValue::Identity => Ok(Some(input.value.clone())),  //TODO: remove clone()
        }
    }
}

#[derive(Deserialize, Debug)]
pub enum ContextJsonPath {
    //TODO: do I need this?
}

impl FromStr for DynamicValue {
    type Err = JsonPathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = if s == "$" {
            DynamicValue::Identity
        } else if s.starts_with("$.") {
            let path = JsonPath::parse(s)?;
            DynamicValue::ValueJsonPath(path)
        } else if s.starts_with("$$") {
            let path = JsonPath::parse(&s[1..])?;
            DynamicValue::Context(path)
        } else {
            let intrinsic_function = IntrinsicFunction::try_from(s);
            if intrinsic_function.is_ok() {
                DynamicValue::IntrinsicFunction(intrinsic_function.unwrap())
            } else {
                DynamicValue::Value(Value::String(String::from(s)))
            }
        };
        Ok(ret)
    }
}

// TODO: Implement Context
pub trait StateMachineContext {}

// TODO: Implement ReferenceJsonPath
/// This type only allows a single result when evaluated.
pub type ReferenceJsonPath = String;

pub type InvertedJsonPath = String;

// TODO: Model a Payload according to https://states-language.net/spec.html#payload-template
pub type Payload = Value;

pub type Parameters = Payload;
pub type ResultSelector = Payload;

// TODO: Maybe move this inside the `choice.rs` module
pub struct ExecutionInput<'a> {
    pub value: &'a Value,
    pub context: &'a Box<dyn StateMachineContext>,
}
