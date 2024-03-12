use serde::Deserialize;
use serde_json::Number;
use crate::asl::types::{MyJsonPath, Timestamp};

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum WaitDuration {
    Seconds(Number),
    SecondsPath(MyJsonPath),
    Timestamp(Timestamp),
    TimestampPath(MyJsonPath),
}
