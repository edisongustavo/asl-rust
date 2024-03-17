use crate::asl::types::{DynamicValue, DynamicValueEvaluateError, ExecutionInput, Timestamp};
use serde::Deserialize;
use serde_json::{Number, Value};
use thiserror::Error;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum Operation {
    StringEquals(String),
    StringEqualsPath(DynamicValue),

    StringLessThan(String),
    StringLessThanPath(DynamicValue),

    StringGreaterThan(String),
    StringGreaterThanPath(DynamicValue),

    StringLessThanEquals(String),
    StringLessThanEqualsPath(DynamicValue),

    StringGreaterThanEquals(String),
    StringGreaterThanEqualsPath(DynamicValue),

    /// Note: The value MUST be a String which MAY contain one or more "*" characters.
    /// The expression yields true if the data value selected by the Variable Path matches the value,
    /// where "*" in the value matches zero or more characters.
    /// Thus, foo*.log matches foo23.log, *.log matches zebra.log, and foo*.* matches foobar.zebra.
    /// No characters other than "*" have any special meaning during matching.
    ///
    /// If the character "*" needs to appear as part of the value without serving as a wildcard, it MUST be escaped with a backslash.
    ///
    /// If the character "\ needs to appear as part of the value without serving as an escape character, it MUST be escaped with a backslash.
    ///
    /// The literal string \* represents *.
    /// The literal string \\ represents \.
    ///
    /// In JSON, all backslashes contained in a string literal value must be escaped with another backslash, therefore, the above will equate to:
    ///
    /// The escaped string \\* represents *.
    /// The escaped string \\\\ represents \.
    ///
    /// If an open escape backslash \ is found in the StringMatches string, the interpreter will throw a runtime error.
    StringMatches(String),

    NumericEquals(Number),
    NumericEqualsPath(DynamicValue),

    NumericLessThan(Number),
    NumericLessThanPath(DynamicValue),

    NumericGreaterThan(Number),
    NumericGreaterThanPath(DynamicValue),

    NumericLessThanEquals(Number),
    NumericLessThanEqualsPath(DynamicValue),

    NumericGreaterThanEquals(Number),
    NumericGreaterThanEqualsPath(DynamicValue),

    BooleanEquals(bool),

    TimestampEquals(Timestamp),
    TimestampEqualsPath(DynamicValue),

    TimestampLessThan(Timestamp),
    TimestampLessThanPath(DynamicValue),

    TimestampGreaterThan(Timestamp),
    TimestampGreaterThanPath(DynamicValue),

    TimestampLessThanEquals(Timestamp),
    TimestampLessThanEqualsPath(DynamicValue),

    TimestampGreaterThanEquals(Timestamp),
    TimestampGreaterThanEqualsPath(DynamicValue),

    IsNull,
    IsPresent,
    IsNumeric,
    IsString,
    IsBoolean,
    IsTimestamp,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ComposedExpression {
    Not(Box<ChoiceExpression>),
    And(Vec<Box<ChoiceExpression>>),
    Or(Vec<Box<ChoiceExpression>>),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChoiceExpression {
    #[serde(rename_all = "PascalCase")]
    BooleanExpression {
        // The spec says that this must be a Path, but that would remove handling other things, such as Context or Intrinsic functions.
        // I believe this is makes it richer though.
        variable: DynamicValue,

        #[serde(flatten)]
        operation: Operation,
    },
    ComposedExpression(ComposedExpression),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ChoiceRule {
    #[serde(flatten)]
    pub expression: ChoiceExpression,
    pub next: String,
}

#[derive(Error, Debug)]
pub enum ChoiceEvaluationError {
    #[error("Wrong type for value '{val:?}', expected a '{expected_type}'")]
    WrongType { val: Value, expected_type: String },
    #[error("Can't parse the string into timestamp: {0}")]
    ParseTimestampError(String),
    #[error("Evaluation error")]
    EvaluateError(DynamicValueEvaluateError),
    #[error("Value not found from input")]
    ValueNotFound,
}

fn match_string(
    value: Option<Value>,
    expected: &str,
    f: impl Fn(&str, &str) -> bool,
) -> Result<bool, ChoiceEvaluationError> {
    let value = value.ok_or_else(|| ChoiceEvaluationError::ValueNotFound)?;
    let Value::String(val) = value else {
        return Err(ChoiceEvaluationError::WrongType {
            val: value.clone(),
            expected_type: "number".to_owned(),
        });
    };
    let ret = f(&val, expected);
    Ok(ret)
}

fn match_number(
    value: Option<Value>,
    expected: f64,
    f: impl Fn(f64, f64) -> bool,
) -> Result<bool, ChoiceEvaluationError> {
    let value = value.ok_or(ChoiceEvaluationError::ValueNotFound)?;
    let Value::Number(val_number) = &value else {
        return Err(ChoiceEvaluationError::WrongType {
            val: value.clone(),
            expected_type: "number".to_owned(),
        });
    };
    let val_f64 = val_number
        .as_f64()
        .ok_or(ChoiceEvaluationError::WrongType {
            val: value.clone(),
            expected_type: "f64".to_owned(),
        })?;
    let ret = f(val_f64, expected);
    Ok(ret)
}

impl Operation {
    pub fn evaluate(&self, input: Option<Value>) -> Result<bool, ChoiceEvaluationError> {
        // TODO: is there a cleaner way to do this?
        match self {
            Operation::StringEquals(expected) => match_string(input, expected, |a, b| a == b),
            // Operation::StringEqualsPath(expected) => match_string(value, expected, |a, b| a == b),
            // Operation::StringLessThan(expected) => match_string(value, expected, |a, b| a < b),
            // Operation::StringLessThanPath(expected) => match_string(&value.from_json_path(expected), |a, b| a < b),
            // Operation::StringGreaterThan(expected) => match_string(value, expected, |a, b| a > b),
            // Operation::StringGreaterThanPath(expected) => match_string(&value.from_json_path(expected), |a, b| a > b),
            // Operation::StringLessThanEquals(expected) => match_string(value, expected, |a, b| a <= b),
            // Operation::StringLessThanEqualsPath(expected) => match_string(&value.from_json_path(expected), |a, b| a <= b),
            // Operation::StringGreaterThanEquals(expected) => match_string(value, expected, |a, b| a >= b),
            // Operation::StringGreaterThanEqualsPath(expected) => match_string(&value.from_json_path(expected), |a, b| a >= b),
            // Operation::StringMatches(expected) => match_string(value, expected, |a, b| a == b), //TODO: implement
            Operation::NumericEquals(expected) => {
                match_number(input, expected.as_f64().unwrap(), |a, b| a == b)
            }
            // Operation::NumericEqualsPath(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericLessThan(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericLessThanPath(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericGreaterThan(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericGreaterThanPath(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericLessThanEquals(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericLessThanEqualsPath(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericGreaterThanEquals(expected) => match_number(value, expected, |a, b| a == b),
            // Operation::NumericGreaterThanEqualsPath(expected) => match_number(value, expected, |a, b| a == b),
            // => {
            //     let Value::Number(_) = value else {
            //         return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "number".to_owned() });
            //     };
            // }
            // Operation::TimestampEquals(_) |
            // Operation::TimestampEqualsPath(_) |
            // Operation::TimestampLessThan(_) |
            // Operation::TimestampLessThanPath(_) |
            // Operation::TimestampGreaterThan(_) |
            // Operation::TimestampGreaterThanPath(_) |
            // Operation::TimestampLessThanEquals(_) |
            // Operation::TimestampLessThanEqualsPath(_) |
            // Operation::TimestampGreaterThanEquals(_) |
            // Operation::TimestampGreaterThanEqualsPath(_) => {
            //     let Value::String(val) = value else {
            //         return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "bool".to_owned() });
            //     };
            //     // Timestamp::try_from(val)
            //     //     .map_err(|err| ChoiceEvaluationError::ParseTimestampError(val.to_owned()))?
            // }
            //
            // Operation::IsNull => {}
            Operation::IsPresent => Ok(input.is_some()),
            Operation::IsNumeric => match_number(input, 0f64, |_, _| true),
            Operation::IsString => match_string(input, "", |_, _| true),
            // Operation::IsBoolean => {}
            // Operation::IsTimestamp => {}
            _ => Ok(false),
        }
    }
}

impl ChoiceRule {
    pub fn evaluate(&self, input: &ExecutionInput) -> Result<bool, ChoiceEvaluationError> {
        self.expression.evaluate(input)
    }
}

impl ChoiceExpression {
    pub fn evaluate(&self, input: &ExecutionInput) -> Result<bool, ChoiceEvaluationError> {
        match self {
            ChoiceExpression::BooleanExpression {
                variable,
                operation,
            } => {
                let transformed_value = variable
                    .evaluate(input)
                    .map_err(ChoiceEvaluationError::EvaluateError)?;
                operation.evaluate(transformed_value)
            }
            ChoiceExpression::ComposedExpression(expression) => expression.evaluate(input),
        }
    }
}

impl ComposedExpression {
    pub fn evaluate(&self, value: &ExecutionInput) -> Result<bool, ChoiceEvaluationError> {
        match self {
            ComposedExpression::Not(expr) => expr.evaluate(value).map(|e| !e),
            ComposedExpression::And(expressions) => {
                for exp in expressions.iter() {
                    if !exp.evaluate(value)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ComposedExpression::Or(expressions) => {
                let mut ret = false;
                for exp in expressions.iter() {
                    let val = exp.evaluate(value)?;
                    ret |= val;
                }
                Ok(ret)
            }
        }
    }
}
