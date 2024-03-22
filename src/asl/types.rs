use crate::asl::instrinsic_functions::{IntrinsicFunction, IntrinsicFunctionExecutionError};
use chrono::{DateTime, ParseError, Utc};
use itertools::Itertools;
use serde::Deserialize;
use serde_json::Value;
use serde_json_path::{JsonPath, ParseError as JsonPathParseError};
use serde_with::DeserializeFromStr;
use std::rc::Rc;
use std::str::FromStr;
use thiserror::Error;

#[derive(DeserializeFromStr, Clone, Debug, PartialEq, Eq)]
pub struct TimestampType {
    pub datetime: DateTime<Utc>,
}

pub(crate) type DateTimeParseError = ParseError;

impl TimestampType {
    /// Calculates how many seconds until we reach the timestamp from now.
    pub(crate) fn seconds_to_now(&self) -> f64 {
        let now = Utc::now();
        let diff = self.datetime - now;
        let seconds = diff.num_milliseconds() as f64 / 1000.;
        seconds.max(0.)
    }
}

impl FromStr for TimestampType {
    type Err = DateTimeParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let val = DateTime::parse_from_rfc3339(s)?.to_utc();
        let ret = TimestampType { datetime: val };
        Ok(ret)
    }
}

#[derive(DeserializeFromStr, Debug, PartialEq, Eq)]
pub enum DynamicValue {
    ValueJsonPath(MyJsonPath),
    Context(MyJsonPath),
    Value(Value),
    IntrinsicFunction(IntrinsicFunction),
}

#[derive(DeserializeFromStr, Debug, PartialEq, Eq)]
pub struct MyJsonPath {
    path: JsonPath,
}

impl MyJsonPath {
    pub fn evaluate(&self, value: &Value) -> Option<Value> {
        let results = self.path.query(value);
        // TODO: this should probably be ".at_most_one()" for ReferencePath
        if results.len() > 1 {
            let values = results.into_iter().cloned().collect_vec();
            Some(Value::Array(values))
        } else {
            results.first().cloned()
        }
    }
}

impl FromStr for MyJsonPath {
    type Err = JsonPathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Fix the usage of paths with hyphen
        let path = s.parse::<JsonPath>();
        let ret = MyJsonPath { path: path? };
        Ok(ret)
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
            DynamicValue::ValueJsonPath(path) => Ok(path.evaluate(&input.value)),
            // DynamicValue::Context(path) => path.evaluate(input.context.into()),
            DynamicValue::Context(path) => Ok(path.evaluate(&input.context.as_value())),
            DynamicValue::Value(val) => Ok(Some(val.clone())), //TODO: remove clone()
            DynamicValue::IntrinsicFunction(function) => function
                .evaluate(&input.value)
                .map_err(|e| DynamicValueEvaluateError::IntrinsicFunctionExecutionError(e)),
            // DynamicValue::Identity => Ok(Some(input.value.clone())), //TODO: remove clone()
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
        if s.starts_with("$$") {
            let path = MyJsonPath::from_str(&s[1..])?;
            return Ok(DynamicValue::Context(path));
        }
        if s.starts_with('$') {
            let path = MyJsonPath::from_str(s)?;
            return Ok(DynamicValue::ValueJsonPath(path));
        }
        if let Ok(func) = IntrinsicFunction::try_from(s) {
            return Ok(DynamicValue::IntrinsicFunction(func));
        }
        Ok(DynamicValue::Value(Value::String(String::from(s))))
    }
}

pub trait StateMachineContext {
    fn as_value(&self) -> &Value;
}

// TODO: Implement ReferenceJsonPath
/// This type only allows a single result when evaluated.
pub type ReferenceJsonPath = String;

pub type InvertedJsonPath = String;

// TODO: Model a Payload according to https://states-language.net/spec.html#payload-template
pub type Payload = Value;

pub type Parameters = Payload;
pub type ResultSelector = Payload;

pub struct ExecutionInput {
    pub value: Value,
    pub context: Rc<dyn StateMachineContext>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asl::types::StateMachineContext;
    use anyhow::Result;
    use rstest::{fixture, rstest};

    #[rstest]
    #[case("value", DynamicValue::Value(Value::String(String::from("value"))))]
    #[case("$", DynamicValue::ValueJsonPath("$".parse::<MyJsonPath>().unwrap()))]
    #[case("$.path", DynamicValue::ValueJsonPath("$.path".parse::<MyJsonPath>().unwrap()))]
    // TODO: Make paths with hyphens work
    // #[case("$.path-with-hyphens", DynamicValue::ValueJsonPath(r#"$["path-with-hyphens"]"#.parse::<MyJsonPath>().unwrap()))]
    #[case("$$", DynamicValue::Context("$".parse::<MyJsonPath>().unwrap()))]
    #[case("$$.path", DynamicValue::Context("$.path".parse::<MyJsonPath>().unwrap()))]
    fn parse_dynamic_value(#[case] input: &str, #[case] expected: DynamicValue) -> Result<()> {
        let actual = DynamicValue::from_str(input)?;
        assert_eq!(actual, expected);
        Ok(())
    }

    #[rstest]
    #[case("$.an [ in valid ]- json path")]
    #[case("$$.an [ in valid ]- json path")]
    // #[case("$", DynamicValue::Identity)]
    fn parse_dynamic_value_error(#[case] input: &str) -> Result<()> {
        let actual = DynamicValue::from_str(input);
        assert!(actual.is_err(), "Expected an error, but got: {:?}", actual);
        Ok(())
    }

    impl StateMachineContext for Value {
        fn as_value(&self) -> &Value {
            self
        }
    }

    #[fixture]
    fn execution_input() -> ExecutionInput {
        let value = Value::from_str(
            r#"
                {
                    "path": "input object: path",
                    "path-with-hyphens": "input object: path-with-hyphens",
                    "array_of_numbers": [0, 1, 2, 3],
                    "object": [
                        {"bar": "input object: bar1"},
                        {"length": 1, "bar": "input object: bar2"},
                        {"length": 1, "bar": "input object: bar3"}
                    ]
                }
            "#,
        )
        .unwrap();
        let context = Value::from_str(
            r#"
                {
                    "path": "context object: path",
                    "path-with-hyphens": "context object: path-with-hyphens",
                    "array_of_numbers": [0, 1, 2, 3]
                }
              "#,
        )
        .unwrap();
        ExecutionInput {
            value,
            context: Rc::new(context),
        }
    }

    #[rstest]
    #[case("value", Some(r#""value""#.to_string()))]
    #[case("$", Some(execution_input().value.to_string()))]
    #[case("$.path", Some(r#""input object: path""#.to_string()))]
    #[case("$.object[?(@.length)].bar", Some(r#"["input object: bar2", "input object: bar3"]"#.to_string()))]
    // TODO: handle hyphens
    // #[case("$.path-with-hyphens", Some(r#""input object: path-with-hyphens""#.to_string()))]
    #[case("$$", Some(execution_input().context.as_value().to_string()))]
    #[case("$$.path", Some(r#""context object: path""#.to_string()))]
    // TODO: handle hyphens
    // #[case("$$.path-with-hyphens", Some(r#""context object: path-with-hyphens""#.to_string()))]
    #[case("$.array_of_numbers", Some("[0, 1, 2, 3]".to_string()))]
    #[case("$.array_of_numbers[-1]", Some("3".to_string()))]
    fn evaluate(
        #[case] path_string: &str,
        #[case] expected: Option<String>,
        execution_input: ExecutionInput,
    ) -> Result<()> {
        let path = DynamicValue::from_str(path_string)?;
        let actual = path.evaluate(&execution_input)?;
        let expected_value = match expected {
            None => None,
            Some(s) => Some(s.parse::<Value>()?),
        };
        assert_eq!(actual, expected_value);
        Ok(())
    }
}

/// A context that returns an empty value everytime. Mostly used if no context is needed.
pub struct EmptyContext {}

impl StateMachineContext for EmptyContext {
    fn as_value(&self) -> &Value {
        &Value::Null
    }
}
