use serde_json::Value;

// TODO: Implement Timestamp
pub type Timestamp = String;

// TODO: Implement JSONPath
pub type MyJsonPath = String;
pub type InvertedJsonPath = String;

// TODO: Model a Payload according to https://states-language.net/spec.html#payload-template
pub type Payload = Value;

pub type Parameters = Payload;
pub type ResultSelector = Payload;
