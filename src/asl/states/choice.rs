use itertools::Itertools;
use serde::Deserialize;
use serde_json::{Number, Value};
use thiserror::Error;
use crate::asl::types::{MyJsonPath, Timestamp};

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum Operation {
    StringEquals(String),
    StringEqualsPath(String),

    StringLessThan(String),
    StringLessThanPath(String),

    StringGreaterThan(String),
    StringGreaterThanPath(String),

    StringLessThanEquals(String),
    StringLessThanEqualsPath(String),

    StringGreaterThanEquals(String),
    StringGreaterThanEqualsPath(String),

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
    NumericEqualsPath(Number),

    NumericLessThan(Number),
    NumericLessThanPath(Number),

    NumericGreaterThan(Number),
    NumericGreaterThanPath(Number),

    NumericLessThanEquals(Number),
    NumericLessThanEqualsPath(Number),

    NumericGreaterThanEquals(Number),
    NumericGreaterThanEqualsPath(Number),

    BooleanEquals(bool),

    TimestampEquals(Timestamp),
    TimestampEqualsPath(Timestamp),

    TimestampLessThan(Timestamp),
    TimestampLessThanPath(Timestamp),

    TimestampGreaterThan(Timestamp),
    TimestampGreaterThanPath(Timestamp),

    TimestampLessThanEquals(Timestamp),
    TimestampLessThanEqualsPath(Timestamp),

    TimestampGreaterThanEquals(Timestamp),
    TimestampGreaterThanEqualsPath(Timestamp),

    IsNull,
    IsPresent,
    IsNumeric,
    IsString,
    IsBoolean,
    IsTimestamp,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum ComposedExpression {
    Not(Box<ChoiceExpression>),
    And(Vec<Box<ChoiceExpression>>),
    Or(Vec<Box<ChoiceExpression>>),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum ChoiceExpression {
    #[serde(rename_all = "PascalCase")]
    BooleanExpression {
        variable: MyJsonPath,

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
    #[error("Wrong type for value '{val:?}', expected a '{expected_type}'", )]
    WrongType { val: Value, expected_type: String },
}

impl TryFrom<Value> for Operation {
    type Error = ChoiceEvaluationError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Operation {
    fn check_type<T>(&self, value: &Value) {}
    pub fn evaluate(&self, value: &Value) -> Result<bool, ChoiceEvaluationError> {
        // TODO: is there a cleaner way to do this?
        match self {
            Operation::StringEquals(_) |
            Operation::StringEqualsPath(_) |
            Operation::StringLessThan(_) |
            Operation::StringLessThanPath(_) |
            Operation::StringGreaterThan(_) |
            Operation::StringGreaterThanPath(_) |
            Operation::StringLessThanEquals(_) |
            Operation::StringLessThanEqualsPath(_) |
            Operation::StringGreaterThanEquals(_) |
            Operation::StringGreaterThanEqualsPath(_) |
            Operation::StringMatches(_)
            => {
                let Value::String(_) = value else {
                    return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "string".to_owned() });
                };
            }
            Operation::NumericEquals(_) |
            Operation::NumericEqualsPath(_) |
            Operation::NumericLessThan(_) |
            Operation::NumericLessThanPath(_) |
            Operation::NumericGreaterThan(_) |
            Operation::NumericGreaterThanPath(_) |
            Operation::NumericLessThanEquals(_) |
            Operation::NumericLessThanEqualsPath(_) |
            Operation::NumericGreaterThanEquals(_) |
            Operation::NumericGreaterThanEqualsPath(_)
            => {
                let Value::Number(_) = value else {
                    return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "number".to_owned() });
                };
            }
            Operation::BooleanEquals(_) => {
                let Value::Bool(_) = value else {
                    return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "bool".to_owned() });
                };
            }
            Operation::TimestampEquals(_) => {}
            Operation::TimestampEqualsPath(_) => {}
            Operation::TimestampLessThan(_) => {}
            Operation::TimestampLessThanPath(_) => {}
            Operation::TimestampGreaterThan(_) => {}
            Operation::TimestampGreaterThanPath(_) => {}
            Operation::TimestampLessThanEquals(_) => {}
            Operation::TimestampLessThanEqualsPath(_) => {}
            Operation::TimestampGreaterThanEquals(_) => {}
            Operation::TimestampGreaterThanEqualsPath(_) => {}

            Operation::IsNull => {}
            Operation::IsPresent => {}
            Operation::IsNumeric => {}
            Operation::IsString => {}
            Operation::IsBoolean => {}
            Operation::IsTimestamp => {}
        }
        match self {
            Operation::StringEquals(expected) => {
                let Value::String(actual) = value else {
                    return Err(ChoiceEvaluationError::WrongType { val: value.clone(), expected_type: "string".to_owned() });
                };
                Ok(expected == actual)
            }
            _ => { Ok(false) }


            // Operation::StringMatches(_) => {}
            // Operation::NumericEquals(_) => {}
            // Operation::NumericEqualsPath(_) => {}
            // Operation::NumericLessThan(_) => {}
            // Operation::NumericLessThanPath(_) => {}
            // Operation::NumericGreaterThan(_) => {}
            // Operation::NumericGreaterThanPath(_) => {}
            // Operation::NumericLessThanEquals(_) => {}
            // Operation::NumericLessThanEqualsPath(_) => {}
            // Operation::NumericGreaterThanEquals(_) => {}
            // Operation::NumericGreaterThanEqualsPath(_) => {}
            // Operation::BooleanEquals(_) => {}
            // Operation::TimestampEquals(_) => {}
            // Operation::TimestampEqualsPath(_) => {}
            // Operation::TimestampLessThan(_) => {}
            // Operation::TimestampLessThanPath(_) => {}
            // Operation::TimestampGreaterThan(_) => {}
            // Operation::TimestampGreaterThanPath(_) => {}
            // Operation::TimestampLessThanEquals(_) => {}
            // Operation::TimestampLessThanEqualsPath(_) => {}
            // Operation::TimestampGreaterThanEquals(_) => {}
            // Operation::TimestampGreaterThanEqualsPath(_) => {}
            // Operation::IsNull => {}
            // Operation::IsPresent => {}
            // Operation::IsNumeric => {}
            // Operation::IsString => {}
            // Operation::IsBoolean => {}
            // Operation::IsTimestamp => {}
        }
    }
}

impl ChoiceRule {
    pub fn evaluate(&self, value: &Value) -> Result<bool, ChoiceEvaluationError> {
        self.expression.evaluate(value)
    }
}

impl ChoiceExpression {
    pub fn evaluate(&self, value: &Value) -> Result<bool, ChoiceEvaluationError> {
        match self {
            ChoiceExpression::BooleanExpression { variable, operation } => {
                let transformed_value = value; // TODO: apply JsonPath based on `variable`
                operation.evaluate(&transformed_value)
            }
            ChoiceExpression::ComposedExpression(expression) => {
                expression.evaluate(&value)
            }
        }
    }
}

impl ComposedExpression {
    pub fn evaluate(&self, value: &Value) -> Result<bool, ChoiceEvaluationError> {
        match self {
            ComposedExpression::Not(expr) => expr.evaluate(value).map(|e| !e),
            ComposedExpression::And(expressions) => {
                let result: Vec<bool> = expressions.iter()
                    .map(|expr| expr.evaluate(value))
                    .try_collect()?;
                Ok(result.iter().all())
            }
            ComposedExpression::Or(expressions) => expressions.iter().any(|expr| expr.evaluate(value)),
        }
    }
}