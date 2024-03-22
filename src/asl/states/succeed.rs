use crate::asl::types::DynamicValue;
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Succeed {
    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
}
