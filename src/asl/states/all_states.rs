use crate::asl::states::choice::Choice;
use crate::asl::states::fail::Fail;
use crate::asl::states::map::Map;
use crate::asl::states::parallel::Parallel;
use crate::asl::states::pass::Pass;
use crate::asl::states::succeed::Succeed;
use crate::asl::states::task::Task;
use crate::asl::states::wait::Wait;
use serde::Deserialize;

/// According to the docs, these are the available common fields for the states:
///
/// |                                | Task     | Parallel | Map      | Pass     | Wait     | Choice   | Succeed  | Fail     |
/// | ------------------------------ | -------- | -------- | -------- | -------- | -------- | -------- | -------- | -------- |
/// | Type                           | Required | Required | Required | Required | Required | Required | Required | Required |
/// | Comment                        | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  |
/// | InputPath, OutputPath          | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  | Allowed  |          |
/// | *One of:* Next *or* "End":true | Required | Required | Required | Required | Required |          |          |          |
/// | ResultPath                     | Allowed  | Allowed  | Allowed  | Allowed  |          |          |          |          |
/// | Parameters                     | Allowed  | Allowed  | Allowed  | Allowed  |          |          |          |          |
/// | ResultSelector                 | Allowed  | Allowed  | Allowed  |          |          |          |          |          |
/// | Retry, Catch                   | Allowed  | Allowed  | Allowed  |          |          |          |          |          |
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "Type")]
pub enum States {
    /// See docs: https://states-language.net/spec.html#task-state
    Task(Task),
    Parallel(Parallel),
    Map(Map),
    Pass(Pass),
    Wait(Wait),
    Choice(Choice),
    Succeed(Succeed),
    Fail(Fail),
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(from = "RawEndOrNext")]
pub enum EndOrNext {
    End,
    Next(String),
}

impl EndOrNext {
    pub fn into_next_state_name(self) -> Option<String> {
        match self {
            EndOrNext::End => None,
            EndOrNext::Next(name) => Some(name),
        }
    }
}

// HACK: Use this type to mask the fact that the "End" field cannot be modeled with raw serde as
//       specified in the spec. We have to ignore its value.
#[derive(Deserialize)]
enum RawEndOrNext {
    #[allow(dead_code)]
    End(bool),
    Next(String),
}

impl From<RawEndOrNext> for EndOrNext {
    fn from(re: RawEndOrNext) -> Self {
        match re {
            RawEndOrNext::End(_) => EndOrNext::End,
            RawEndOrNext::Next(val) => EndOrNext::Next(val),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use rstest::*;

    #[rstest]
    #[case(r#"{"End": true}"#, EndOrNext::End)]
    #[case(r#"{"End": false}"#, EndOrNext::End)]
    #[case(r#"{"Next": "foo"}"#, EndOrNext::Next("foo".to_string()))]
    fn parse_end_or_next(#[case] definition: &str, #[case] expected: EndOrNext) -> Result<()> {
        let actual: EndOrNext = serde_json::from_str(definition)?;
        assert_eq!(actual, expected);
        Ok(())
    }
}
