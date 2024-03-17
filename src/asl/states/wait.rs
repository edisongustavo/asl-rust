use serde::Deserialize;
use serde_json::Number;
use crate::asl::types::{DynamicValue, Timestamp};

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum WaitDuration {
    Seconds(Number),
    SecondsPath(DynamicValue),
    Timestamp(Timestamp),
    TimestampPath(DynamicValue),
}
