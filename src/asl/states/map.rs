use std::collections::HashMap;
use serde::Deserialize;
use serde_json::{Value};
use crate::asl::states::all_states::State;
use crate::asl::types::{DynamicValue, Payload};


#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub struct MapStateIterator {
    start_at: String,
    states: HashMap<String, State>,
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
