use crate::asl::states::all_states::EndOrNext;
use crate::asl::types::{DynamicValue, Payload};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Pass {
    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
    #[serde(flatten)]
    pub end_or_next: EndOrNext,
    pub result_path: Option<DynamicValue>,
    pub parameters: Option<Payload>,
}
