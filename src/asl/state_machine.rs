use std::collections::HashMap;
use thiserror::Error;
use serde::Deserialize;
use serde_json::{Error as SerdeError, Number, Value};
use crate::asl::error_handling::{Catcher, Retrier};
use crate::asl::states::choice::ChoiceRule;
use crate::asl::states::fail::{FailStateCauseField, FailStateErrorField};
use crate::asl::states::task::{HeartbeatSecondsOrPath, TimeoutSecondsOrPath};
use crate::asl::states::wait::WaitDuration;
use crate::asl::states::map::{ItemBatcherConfiguration, MapStateIterator, ResultWriterConfiguration};
use crate::asl::types::{InvertedJsonPath, MyJsonPath, Parameters, Payload, ResultSelector};

struct Execution {}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing a field")]
    MissingField,
    #[error("Missing the 'StartsAt' field")]
    StartStateNotDefinedInListOfStates,

    #[error("Malformed input: {0}")]
    MalformedInput(SerdeError),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum EndOrNext {
    End(bool),
    Next(String)
}

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
#[serde(rename_all = "PascalCase", tag = "Type")]
pub enum State {
    /// See docs: https://states-language.net/spec.html#task-state
    #[serde(rename_all = "PascalCase")]
    Task {
        /// A Task State MUST include a "Resource" field, whose value MUST be a URI that uniquely
        /// identifies the specific task to execute.
        /// The States language does not constrain the URI scheme nor any other part of the URI.
        resource: String,

        /// Tasks can optionally specify timeouts. Timeouts (the "TimeoutSeconds" and "HeartbeatSeconds" fields) are specified in seconds and MUST be positive integers.
        ///
        /// Both the total and heartbeat timeouts can be provided indirectly. A Task State may have "TimeoutSecondsPath" and "HeartbeatSecondsPath" fields which MUST be Reference Paths which, when resolved, MUST select fields whose values are positive integers. A Task State MUST NOT include both "TimeoutSeconds" and "TimeoutSecondsPath" or both "HeartbeatSeconds" and "HeartbeatSecondsPath".
        //
        /// If provided, the "HeartbeatSeconds" interval MUST be smaller than the "TimeoutSeconds" value.
        ///
        /// If not provided, the default value of "TimeoutSeconds" is 60.
        #[serde(flatten, default)]
        timeout: Option<TimeoutSecondsOrPath>,

        /// See docs for 'timeout' field
        #[serde(flatten)]
        heartbeat: Option<HeartbeatSecondsOrPath>, //TODO: validation! If provided, the "HeartbeatSeconds" interval MUST be smaller than the "TimeoutSeconds" value.

        /// A Task State MAY include a "Credentials" field, whose value MUST be a JSON object whose
        /// value is defined by the interpreter.
        /// The States language does not constrain the value of the "Credentials" field.
        /// The interpreter will use the specified credentials to execute the work identified by the state's "Resource" field.
        credentials: Option<Value>,

        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
        #[serde(flatten)]
        end_or_next: EndOrNext,
        result_path: Option<MyJsonPath>,
        parameters: Option<Parameters>,
        result_selector: Option<ResultSelector>,
        retry: Option<Vec<Retrier>>,
        catch: Option<Vec<Catcher>>,
    },
    ///
    #[serde(rename_all = "PascalCase")]
    Parallel {
        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
        #[serde(flatten)]
        end_or_next: EndOrNext,
        result_path: Option<MyJsonPath>,
        parameters: Option<Parameters>,
        result_selector: Option<ResultSelector>,
        retry: Option<Vec<Retrier>>,
        catch: Option<Vec<Catcher>>,
    },
    ///
    #[serde(rename_all = "PascalCase")]
    Map {
        max_concurrency: Option<u32>,
        #[serde(alias="Iterator")]
        item_processor: MapStateIterator,
        items_path: Option<MyJsonPath>,
        item_selector: Option<HashMap<InvertedJsonPath, MyJsonPath>>,
        item_batcher: Option<ItemBatcherConfiguration>,
        result_writer: Option<ResultWriterConfiguration>,
        tolerated_failure_count: Option<u32>,
        tolerated_failure_percentage: Option<u32>,

        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
        #[serde(flatten)]
        end_or_next: EndOrNext,
        result_path: Option<MyJsonPath>,
        #[deprecated] // Use `item_selector` instead
        parameters: Option<Parameters>,
        result_selector: Option<ResultSelector>,
        retry: Option<Vec<Retrier>>,
        catch: Option<Vec<Catcher>>,
    },
    #[serde(rename_all = "PascalCase")]
    Pass {
        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
        #[serde(flatten)]
        end_or_next: EndOrNext,
        result_path: Option<MyJsonPath>,
        parameters: Option<Payload>,
    },
    #[serde(rename_all = "PascalCase")]
    Wait {
        #[serde(flatten)]
        duration: WaitDuration,
        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
        #[serde(flatten)]
        end_or_next: EndOrNext,
    },

    ///
    #[serde(rename_all = "PascalCase")]
    Choice {
        choices: Vec<ChoiceRule>,

        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
    },
    #[serde(rename_all = "PascalCase")]
    Succeed {
        // Common fields
        comment: Option<String>,
        input_path: Option<MyJsonPath>,
        output_path: Option<MyJsonPath>,
    },
    #[serde(rename_all = "PascalCase")]
    Fail {
        #[serde(flatten)]
        error: Option<FailStateErrorField>,
        #[serde(flatten)]
        cause: Option<FailStateCauseField>,
        // Common fields
        comment: Option<String>,
    },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct StateMachineDefinition {
    states: HashMap<String, State>,
    comment: Option<String>,
    start_at: String,
    version: Option<String>,
    timeout_seconds: Option<Number>,
}

type ResourceTypesActions = HashMap<&'static str, fn(&Value) -> Option<Value>>;

struct StateMachine {
    definition: StateMachineDefinition,
    resources: ResourceTypesActions,
}

impl StateMachine {
    // fn parse(definition: &str, resources: ResourceTypesActions) -> Result<StateMachine, ParseError> {
    fn parse(definition: &str) -> Result<StateMachine, ParseError> {
        let definition = serde_json::from_str(definition).map_err(|e: SerdeError| ParseError::MalformedInput(e))?;
        let state_machine = StateMachine {
            definition,
            resources: HashMap::new(), // TODO: Check how to define handling of resource types
        };
        // TODO: validate state machine

        Ok(state_machine)
    }

    fn start(input: &Value) -> Execution {
        Execution {}
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use super::*;
    use rstest::*;
    use itertools::Itertools;
    use anyhow::Result;

    #[rstest]
    fn parse_hello_world_state_machine() -> Result<()> {
        let definition = include_str!("test-data/hello-world.json");
        // let resources = hash_map![
        //     "return" => |input: &Value| -> Option<Value> { Some(input.to_owned()) }
        // ];
        // let ret = resources.get("return").unwrap()(&serde_json::json!(1));
        let state_machine = StateMachine::parse(definition)?;

        // Testing internals, but ok for now
        assert_eq!(state_machine.definition.states.keys().collect_vec(), vec!["Hello World"]);
        let state_hello_world = &state_machine.definition.states["Hello World"];
        assert_eq!(state_hello_world, &State::Task {
            comment: None,
            end_or_next: EndOrNext::End(true),
            resource: String::from("return"),
            credentials: None,
            input_path: None,
            output_path: None,
            result_path: None,
            parameters: None,
            result_selector: None,
            retry: None,
            catch: None,
            heartbeat: None,
            // timeout: Some(TimeoutSeconds(60)), //TODO: Check why this is not the case
            timeout: None,
        });

        // API testing
        // let start_state = state_machine.start_state();
        //
        // let execution = state_machine.start(r#"
        //     { "arg": "Hello world" }
        // "#);
        // let result = execution.next();
        // assert_eq!(result?, "Hello world");
        // // assert_eq!(execution.state);
        Ok(())
    }

    #[rstest]
    fn parse_valid_cases(#[files("src/**/test-data/asl-validator/valid-*.json")] path: PathBuf) -> Result<()> {
        let definition = fs::read_to_string(path)?;
        StateMachine::parse(definition.as_str())?;
        Ok(())
    }

    // #[rstest]
    // fn parse_invalid_cases(#[files("src/**/test-data/asl-validator/invalid-*.json")] path: PathBuf) -> Result<()> {
    //     let definition = fs::read_to_string(path)?;
    //     let ret = StateMachine::parse(definition.as_str());
    //     assert_eq!(ret.is_err(), true);
    //     Ok(())
    // }
}
