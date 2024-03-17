use serde_json::Number;
use serde::Deserialize;
use crate::asl::types::DynamicValue;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum TimeoutSecondsOrPath {
    TimeoutSeconds(Number),
    TimeoutSecondsPath(DynamicValue)
}

impl Default for TimeoutSecondsOrPath {
    fn default() -> Self {
        TimeoutSecondsOrPath::TimeoutSeconds(Number::from(60))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum HeartbeatSecondsOrPath {
    HeartbeatSeconds(u32),
    HeartbeatSecondsPath(DynamicValue)
}

// pub struct TaskExecutionError {
//     error_name: ErrorName,
//     // cause:
//     // #[error("Missing the 'StartsAt' field")]
//     // StartStateNotDefinedInListOfStates,
//     //
//     // #[error("Malformed input: {0}")]
//     // MalformedInput(SerdeError),
// }

