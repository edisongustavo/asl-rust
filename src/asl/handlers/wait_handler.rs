use crate::asl::error_handling::StateMachineExecutionPredefinedErrors;
use crate::asl::execution::{Execution, StateExecutionHandler, StateMachineExecutionError};
use crate::asl::states::wait::{Wait, WaitDuration};
use crate::asl::types::{DynamicValue, ExecutionInput, TimestampType};
use crate::asl::HandlerOutput;
use chrono::ParseError;
use serde_json::{Number, Value};
use std::str::FromStr;
use thiserror::Error;
use crate::asl::state_machine::StateMachine;

fn evaluate_path(input: &ExecutionInput, path: &DynamicValue) -> Value {
    let path_evaluated = path
        .evaluate(input)
        .unwrap_or_else(|err| panic!("Can't evaluate path due to {}", err.to_string()));
    let value = path_evaluated.expect("Path evaluated to an empty value");
    value
}

enum UnwrappedWaitDuration {
    Seconds(Number),
    Timestamp(TimestampType),
}

#[derive(Error, Debug)]
pub(crate) enum WaitStateExecutionError {
    #[error("SecondsPath evaluated to a value that is not a number. Got: {0}")]
    SecondsPathNotANumber(Value),

    #[error("TimestampPath evaluated to a value that is not a string. Got: {0}")]
    TimestampPathNotAString(Value),

    #[error("TimestampPath evaluated to a malformed timestamp string. Error: {0}")]
    MalformedTimestamp(#[source] ParseError),
}

impl From<WaitStateExecutionError> for StateMachineExecutionError {
    fn from(value: WaitStateExecutionError) -> Self {
        StateMachineExecutionError {
            error: StateMachineExecutionPredefinedErrors::Custom("Wait State failed".to_string()),
            cause: Some(value.to_string())
        }
    }
}

impl<'a, H: StateExecutionHandler> Execution<'a, H> {
    pub fn handle_wait(
        &self,
        wait: &Wait,
        input: &ExecutionInput,
    ) -> Result<HandlerOutput, WaitStateExecutionError> {
        let unwrapped_wait_duration = match &wait.duration {
            WaitDuration::SecondsPath(path) => {
                let value = evaluate_path(input, &path);
                let number = value
                    .as_number()
                    .ok_or_else(|| WaitStateExecutionError::SecondsPathNotANumber(value.clone()))?;
                UnwrappedWaitDuration::Seconds(number.clone())
            }
            WaitDuration::TimestampPath(path) => {
                let value = evaluate_path(input, &path);
                let timestamp_string = value
                    .as_str()
                    .ok_or_else(|| WaitStateExecutionError::TimestampPathNotAString(value.clone()))?;
                let timestamp = TimestampType::from_str(timestamp_string)
                    .map_err(|e| WaitStateExecutionError::MalformedTimestamp(e))?;
                UnwrappedWaitDuration::Timestamp(timestamp)
            }
            WaitDuration::Seconds(seconds) => UnwrappedWaitDuration::Seconds(seconds.clone()),
            WaitDuration::Timestamp(timestamp) => {
                UnwrappedWaitDuration::Timestamp(timestamp.clone())
            }
        };

        let resolved_duration = match &unwrapped_wait_duration {
            UnwrappedWaitDuration::Seconds(seconds) => seconds
                .as_f64()
                .expect("Can't convert number to f64. Maybe it's too big."),
            UnwrappedWaitDuration::Timestamp(timestamp) => timestamp.seconds_to_now(),
        };

        self.state_execution_handler.wait(resolved_duration);
        Ok(HandlerOutput {
            output: None,
            next_state: wait.end_or_next.clone(),
        })
    }
}
