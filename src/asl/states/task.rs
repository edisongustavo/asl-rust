use serde_json::Number;
use serde::Deserialize;
use crate::asl::types::MyJsonPath;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum TimeoutSecondsOrPath {
    TimeoutSeconds(Number),
    TimeoutSecondsPath(MyJsonPath)
}

impl Default for TimeoutSecondsOrPath {
    fn default() -> Self {
        TimeoutSecondsOrPath::TimeoutSeconds(Number::from(60))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum HeartbeatSecondsOrPath {
    HeartbeatSeconds(u32),
    HeartbeatSecondsPath(MyJsonPath)
}
