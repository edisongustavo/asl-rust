use serde_json::Number;
use serde::Deserialize;
use crate::asl::types::MyJsonPath;

// TODO: Maybe this could be a parameter. It could be a string or a parameter type of the StateMachine...
#[derive(Deserialize, Debug, PartialEq, Eq)]
enum JitterStrategy {
    // TODO: Check which values we want to implement here
    FULL,
}

/// See https://states-language.net/spec.html#appendix-a
#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged)]
enum ErrorName {
    /// A wildcard which matches any Error Name.
    #[serde(rename = "States.ALL")]
    StatesALL,

    /// A Task State failed to heartbeat for a time longer than the "HeartbeatSeconds" value.
    #[serde(rename = "States.HeartbeatTimeout")]
    StatesHeartbeatTimeout,

    /// A Task State either ran longer than the "TimeoutSeconds" value, or failed to heartbeat for a
    /// time longer than the "HeartbeatSeconds" value.
    #[serde(rename = "States.Timeout")]
    StatesTimeout,

    /// A Task State failed during the execution.
    #[serde(rename = "States.TaskFailed")]
    StatesTaskFailed,

    /// A Task State failed because it had insufficient privileges to execute the specified code.
    #[serde(rename = "States.Permissions")]
    StatesPermissions,

    /// A state’s "ResultPath" field cannot be applied to the input the state received.
    #[serde(rename = "States.ResultPathMatchFailure")]
    StatesResultPathMatchFailure,

    /// Within a state’s "Parameters" field, the attempt to replace a field whose name ends in ".$" using a Path failed.
    #[serde(rename = "States.ParameterPathFailure")]
    StatesParameterPathFailure,

    /// A branch of a Parallel State failed.
    #[serde(rename = "States.BranchFailed")]
    StatesBranchFailed,

    /// A Choice State failed to find a match for the condition field extracted from its input.
    #[serde(rename = "States.NoChoiceMatched")]
    StatesNoChoiceMatched,

    /// Within a Payload Template, the attempt to invoke an Intrinsic Function failed.
    #[serde(rename = "States.IntrinsicFailure")]
    StatesIntrinsicFailure,

    /// A Map state failed because the number of failed items exceeded the configured tolerated failure threshold.
    #[serde(rename = "States.ExceedToleratedFailureThreshold")]
    StatesExceedToleratedFailureThreshold,

    /// A Map state failed to read all items as specified by the "ItemReader" field.
    #[serde(rename = "States.ItemReaderFailed")]
    StatesItemReaderFailed,

    /// A Map state failed to write all results as specified by the "ResultWriter" field.
    #[serde(rename = "States.ResultWriterFailed")]
    StatesResultWriterFailed,

    Custom(String),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Retrier {
    error_equals: Vec<ErrorName>,

    #[serde(default="max_attempts_default")]
    max_attempts: u32,
    #[serde(default="interval_seconds_default")]
    interval_seconds: Number,
    max_delay_seconds: Option<Number>,
    #[serde(default="backoff_rate_default")]
    backoff_rate: Number,
    jitter_strategy: Option<JitterStrategy>,
}

fn max_attempts_default() -> u32 {
    3
}

fn interval_seconds_default() -> Number {
    Number::from(1)
}

fn backoff_rate_default() -> Number {
    Number::from_f64(2.0).expect("Can't convert default hardcoded value. Something's off in the configuration of Serde")
}


#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Catcher {
    error_equals: Vec<ErrorName>,
    next: String,
    result_path: Option<MyJsonPath>
}
