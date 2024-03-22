use crate::asl::types::DynamicValue;
use serde::Deserialize;
use serde_json::Number;
use thiserror::Error;

// TODO: Maybe this could be a parameter. It could be a string or a parameter type of the StateMachine...
#[derive(Deserialize, Debug, PartialEq, Eq)]
enum JitterStrategy {
    // TODO: Check which values we want to implement here
    FULL,
}

/// See https://states-language.net/spec.html#appendix-a
#[derive(Deserialize, Error, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum StateMachineExecutionPredefinedErrors {
    /// A wildcard which matches any Error Name.
    #[serde(rename = "States.ALL")]
    #[error("Catch-all error")]
    StatesALL,

    /// A Task State failed to heartbeat for a time longer than the "HeartbeatSeconds" value.
    #[serde(rename = "States.HeartbeatTimeout")]
    #[error(
        "A Task State failed to heartbeat for a time longer than the 'HeartbeatSeconds' value."
    )]
    StatesHeartbeatTimeout,

    /// A Task State either ran longer than the "TimeoutSeconds" value, or failed to heartbeat for a
    /// time longer than the "HeartbeatSeconds" value.
    #[serde(rename = "States.Timeout")]
    #[error("A Task State either ran longer than the 'TimeoutSeconds' value, or failed to heartbeat for a time longer than the 'HeartbeatSeconds' value.")]
    StatesTimeout,

    /// A Task State failed during the handlers.
    #[serde(rename = "States.TaskFailed")]
    #[error("A Task State failed during the handlers.")]
    StatesTaskFailed,

    /// A Task State failed because it had insufficient privileges to execute the specified code.
    #[serde(rename = "States.Permissions")]
    #[error(
        "A Task State failed because it had insufficient privileges to execute the specified code."
    )]
    StatesPermissions,

    /// A state’s "ResultPath" field cannot be applied to the input the state received.
    #[serde(rename = "States.ResultPathMatchFailure")]
    #[error("A state’s 'ResultPath' field cannot be applied to the input the state received.")]
    StatesResultPathMatchFailure,

    /// Within a state’s "Parameters" field, the attempt to replace a field whose name ends in ".$" using a Path failed.
    #[serde(rename = "States.ParameterPathFailure")]
    #[error("Within a state’s 'Parameters' field, the attempt to replace a field whose name ends in '.$' using a Path failed.")]
    StatesParameterPathFailure,

    /// A branch of a Parallel State failed.
    #[serde(rename = "States.BranchFailed")]
    #[error("A branch of a Parallel State failed.")]
    StatesBranchFailed,

    /// A Choice State failed to find a match for the condition field extracted from its input.
    #[serde(rename = "States.NoChoiceMatched")]
    #[error("A branch of a Parallel State failed.")]
    StatesNoChoiceMatched,

    /// Within a Payload Template, the attempt to invoke an Intrinsic Function failed.
    #[serde(rename = "States.IntrinsicFailure")]
    #[error("Within a Payload Template, the attempt to invoke an Intrinsic Function failed.")]
    StatesIntrinsicFailure,

    /// A Map state failed because the number of failed items exceeded the configured tolerated failure threshold.
    #[serde(rename = "States.ExceedToleratedFailureThreshold")]
    #[error("A Map state failed because the number of failed items exceeded the configured tolerated failure threshold.")]
    StatesExceedToleratedFailureThreshold,

    /// A Map state failed to read all items as specified by the "ItemReader" field.
    #[serde(rename = "States.ItemReaderFailed")]
    #[error("A Map state failed to read all items as specified by the 'ItemReader' field.")]
    StatesItemReaderFailed,

    /// A Map state failed to write all results as specified by the "ResultWriter" field.
    #[serde(rename = "States.ResultWriterFailed")]
    #[error("A Map state failed to write all results as specified by the 'ResultWriter' field.")]
    StatesResultWriterFailed,

    // #[error("Custom error: '{error}', cause: {cause:?}")]
    // Custom {
    //     error: String,
    //     cause: Option<String>,
    // },
    #[error("{0}")]
    Custom(String),
}

pub fn matches_error(expected_error: &str, catch: &StateMachineExecutionPredefinedErrors) -> bool {
    match catch {
        StateMachineExecutionPredefinedErrors::StatesALL => true,
        StateMachineExecutionPredefinedErrors::Custom(error) => error == expected_error,
        e => false, // TODO: implement. Use the 'strum' crate?
                    // StateMachineExecutionError::StatesHeartbeatTimeout => {}
                    // StateMachineExecutionError::StatesTimeout => {}
                    // StateMachineExecutionError::StatesTaskFailed => {}
                    // StateMachineExecutionError::StatesPermissions => {}
                    // StateMachineExecutionError::StatesResultPathMatchFailure => {}
                    // StateMachineExecutionError::StatesParameterPathFailure => {}
                    // StateMachineExecutionError::StatesBranchFailed => {}
                    // StateMachineExecutionError::StatesNoChoiceMatched => {}
                    // StateMachineExecutionError::StatesIntrinsicFailure => {}
                    // StateMachineExecutionError::StatesExceedToleratedFailureThreshold => {}
                    // StateMachineExecutionError::StatesItemReaderFailed => {}
                    // StateMachineExecutionError::StatesResultWriterFailed => {}
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Retrier {
    error_equals: Vec<StateMachineExecutionPredefinedErrors>,

    #[serde(default = "max_attempts_default")]
    max_attempts: u32,
    #[serde(default = "interval_seconds_default")]
    interval_seconds: Number,
    max_delay_seconds: Option<Number>,
    #[serde(default = "backoff_rate_default")]
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
    Number::from_f64(2.0).expect(
        "Can't convert default hardcoded value. Something's off in the configuration of Serde",
    )
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Catcher {
    pub error_equals: Vec<StateMachineExecutionPredefinedErrors>,
    pub next: String,
    pub result_path: Option<DynamicValue>,
}
