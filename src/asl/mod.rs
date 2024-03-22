use crate::asl::states::all_states::EndOrNext;
use serde_json::Value;

pub mod error_handling;
pub mod execution;
pub mod handlers;
pub mod instrinsic_functions;
pub mod state_machine;
pub mod states;
pub mod types;
pub mod itertools_utils;

// TODO: Move into the handlers
pub struct HandlerOutput {
    pub output: Option<Value>,
    pub next_state: EndOrNext,
}
