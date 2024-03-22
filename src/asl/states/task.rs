use crate::asl::error_handling::{Catcher, Retrier};
use crate::asl::states::all_states::EndOrNext;
use crate::asl::types::{DynamicValue, Parameters, ResultSelector};
use serde::Deserialize;
use serde_json::{Number, Value};

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum TimeoutSecondsOrPath {
    TimeoutSeconds(Number),
    TimeoutSecondsPath(DynamicValue),
}

impl Default for TimeoutSecondsOrPath {
    fn default() -> Self {
        TimeoutSecondsOrPath::TimeoutSeconds(Number::from(60))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum HeartbeatSecondsOrPath {
    HeartbeatSeconds(u32),
    HeartbeatSecondsPath(DynamicValue),
}

/// See docs: https://states-language.net/spec.html#task-state
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Task {
    /// A Task State MUST include a "Resource" field, whose value MUST be a URI that uniquely
    /// identifies the specific task to execute.
    /// The States language does not constrain the URI scheme nor any other part of the URI.
    pub resource: String,

    /// Tasks can optionally specify timeouts. Timeouts (the "TimeoutSeconds" and "HeartbeatSeconds" fields) are specified in seconds and MUST be positive integers.
    ///
    /// Both the total and heartbeat timeouts can be provided indirectly. A Task State may have "TimeoutSecondsPath" and "HeartbeatSecondsPath" fields which MUST be Reference Paths which, when resolved, MUST select fields whose values are positive integers. A Task State MUST NOT include both "TimeoutSeconds" and "TimeoutSecondsPath" or both "HeartbeatSeconds" and "HeartbeatSecondsPath".
    //
    /// If provided, the "HeartbeatSeconds" interval MUST be smaller than the "TimeoutSeconds" value.
    ///
    /// If not provided, the default value of "TimeoutSeconds" is 60.
    #[serde(flatten, default)]
    pub timeout: Option<TimeoutSecondsOrPath>,

    /// See docs for 'timeout' field
    #[serde(flatten)]
    pub heartbeat: Option<HeartbeatSecondsOrPath>, //TODO: validation! If provided, the "HeartbeatSeconds" interval MUST be smaller than the "TimeoutSeconds" value.

    /// A Task State MAY include a "Credentials" field, whose value MUST be a JSON object whose
    /// value is defined by the interpreter.
    /// The States language does not constrain the value of the "Credentials" field.
    /// The interpreter will use the specified credentials to execute the work identified by the state's "Resource" field.
    pub credentials: Option<Value>,

    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
    #[serde(flatten)]
    pub end_or_next: EndOrNext,
    pub result_path: Option<DynamicValue>,
    pub parameters: Option<Parameters>,
    pub result_selector: Option<ResultSelector>,
    pub retry: Option<Vec<Retrier>>,
    pub catch: Option<Vec<Catcher>>,
}
