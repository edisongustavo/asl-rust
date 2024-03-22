use crate::asl::error_handling::{Catcher, Retrier};
use crate::asl::states::all_states::{EndOrNext, States};
use crate::asl::types::{DynamicValue, InvertedJsonPath, Parameters, Payload, ResultSelector};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct MapStateIterator {
    start_at: String,
    states: HashMap<String, States>,
    processor_config: Option<Value>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ResultWriterConfiguration {}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ToleratedFailurePercentage {
    ToleratedFailurePercentage(u32),
    ToleratedFailurePercentagePath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum ToleratedFailureCount {
    ToleratedFailureCount(u32),
    ToleratedFailureCountPath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum MaxItemsPerBatch {
    MaxItemsPerBatch(u32),
    MaxItemsPerBatchPath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub enum MaxInputBytesPerBatch {
    MaxInputBytesPerBatch(u32),
    MaxInputBytesPerBatchPath(DynamicValue),
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct ItemBatcherConfiguration {
    batch_input: Option<Payload>,
    #[serde(flatten)]
    max_items_per_batch: Option<MaxItemsPerBatch>,
    #[serde(flatten)]
    max_input_bytes_per_batch: Option<MaxInputBytesPerBatch>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct Map {
    pub max_concurrency: Option<u32>,
    #[serde(alias = "Iterator")]
    pub item_processor: MapStateIterator,
    pub items_path: Option<DynamicValue>,
    pub item_selector: Option<HashMap<InvertedJsonPath, DynamicValue>>,
    pub item_batcher: Option<ItemBatcherConfiguration>,
    pub result_writer: Option<ResultWriterConfiguration>,
    pub tolerated_failure_count: Option<u32>,
    pub tolerated_failure_percentage: Option<u32>,

    // Common fields
    pub comment: Option<String>,
    pub input_path: Option<DynamicValue>,
    pub output_path: Option<DynamicValue>,
    #[serde(flatten)]
    pub end_or_next: EndOrNext,
    pub result_path: Option<DynamicValue>,
    #[deprecated] // Use `item_selector` instead
    pub parameters: Option<Parameters>,
    pub result_selector: Option<ResultSelector>,
    pub retry: Option<Vec<Retrier>>,
    pub catch: Option<Vec<Catcher>>,
}
