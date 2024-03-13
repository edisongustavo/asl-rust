use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use crate::asl::error_handling::{Catcher, Retrier};
use crate::asl::states::choice::ChoiceRule;
use crate::asl::states::fail::{FailStateCauseField, FailStateErrorField};
use crate::asl::states::map::{ItemBatcherConfiguration, MapStateIterator, ResultWriterConfiguration};
use crate::asl::states::task::{HeartbeatSecondsOrPath, TimeoutSecondsOrPath};
use crate::asl::states::wait::WaitDuration;
use crate::asl::types::{InvertedJsonPath, MyJsonPath, Parameters, Payload, ResultSelector};

pub mod choice;
pub mod fail;
pub mod wait;
pub mod task;
pub mod map;

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

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum EndOrNext {
    End(bool), //TODO: Check how to remove this "bool"
    Next(String)
}
