use crate::asl::error_handling::{Catcher, Retrier};
use crate::asl::states::all_states::EndOrNext;
use crate::asl::types::{DynamicValue, Parameters, ResultSelector};
use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Parallel {
    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
    #[serde(flatten)]
    pub end_or_next: EndOrNext,
    pub result_path: Option<DynamicValue>,
    pub parameters: Option<Parameters>,
    pub result_selector: Option<ResultSelector>,
    pub retry: Option<Vec<Retrier>>,
    pub catch: Option<Vec<Catcher>>,
}
