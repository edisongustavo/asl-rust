use crate::asl::types::{DynamicValue, Timestamp};
use serde::Deserialize;
use serde_json::Number;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum WaitDuration {
    Seconds(Number),
    SecondsPath(DynamicValue),
    Timestamp(Timestamp),
    TimestampPath(DynamicValue),
}
