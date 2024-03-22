use crate::asl::states::all_states::EndOrNext;
use crate::asl::types::{DynamicValue, TimestampType};
use serde::Deserialize;
use serde_json::Number;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum WaitDuration {
    Seconds(Number),
    SecondsPath(DynamicValue),
    Timestamp(TimestampType),
    TimestampPath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Wait {
    #[serde(flatten)]
    pub duration: WaitDuration,
    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
    #[serde(flatten)]
    pub end_or_next: EndOrNext,
}
