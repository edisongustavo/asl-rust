use serde::Deserialize;
use serde_json::Number;
use crate::asl::types::{MyJsonPath, Timestamp};

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum Operation {
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
enum ChoiceExpression {
    #[serde(rename_all = "PascalCase")]
    BooleanExpression {
        variable: MyJsonPath,

        #[serde(flatten)]
        operation: Operation,
    },
    ComposedExpression(ComposedExpression)
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ChoiceRule {
    #[serde(flatten)]
    expression: ChoiceExpression,
    next: String,
}
