use crate::asl::error_handling::StateMachineExecutionPredefinedErrors;
use crate::asl::execution::{Execution, StateExecutionHandler, StateMachineExecutionError};
use crate::asl::states::fail::{Fail, FailStateCauseField, FailStateErrorField};
use crate::asl::types::{DynamicValue, DynamicValueEvaluateError, ExecutionInput};
use thiserror::Error;

pub(crate) struct FailStateOutput {
    pub error: String,
    pub cause: Option<String>,
}

#[derive(Error, Debug)]
pub(crate) enum FailStateExecutionError {
    #[error("Invalid 'ErrorPath' field due to {0}")]
    MalformedErrorPath(#[source] HandlePathError),

    #[error("Invalid 'CausePath' field due to {0}")]
    MalformedCausePath(#[source] HandlePathError),

    #[error("'ErrorPath' field evaluated value is not a valid string within the input.")]
    ErrorPathIsNotAString,
}

impl From<FailStateExecutionError> for StateMachineExecutionError {
    fn from(value: FailStateExecutionError) -> Self {
        StateMachineExecutionError {
            error: StateMachineExecutionPredefinedErrors::Custom(
                "Malformed Fail State".to_string(),
            ),
            cause: Some(value.to_string()),
        }
    }
}

impl<'a, H: StateExecutionHandler> Execution<'a, H> {
    pub fn handle_fail(
        &self,
        fail: &Fail,
        input: &ExecutionInput,
    ) -> Result<FailStateOutput, FailStateExecutionError> {
        // impl HandleState for Fail {
        //     fn handle(&self, input: &ExecutionInput) -> Result<HandlerOutput, StateMachineExecutionError> {
        // A Fail state MAY have "ErrorPath" and "CausePath" fields whose values MUST be
        // Reference Paths or Intrinsic Functions which, when resolved, MUST be string values.
        // A Fail state MUST NOT include both "Error" and "ErrorPath" or both "Cause" and "CausePath".
        let error = match &fail.error {
            // TODO: Investigate if this is the right error type emitted by a State Machine when it reaches a Fail State.
            //       More specifically, this page should help: https://docs.aws.amazon.com/step-functions/latest/apireference/API_DescribeExecution.html#API_DescribeExecution_ResponseSyntax
            None => String::from("Reached Fail State"),
            Some(error_field) => match error_field {
                FailStateErrorField::Error(err) => err.into(),
                FailStateErrorField::ErrorPath(err_path) => {
                    evaluate_path_as_string(input, err_path)
                        .map_err(|err| FailStateExecutionError::MalformedErrorPath(err))?
                        .ok_or(FailStateExecutionError::ErrorPathIsNotAString)?
                }
            },
        };
        let cause = match fail.cause.as_ref() {
            None => None,
            Some(cause_field) => match cause_field {
                FailStateCauseField::Cause(cause_string) => Some(cause_string.into()),
                FailStateCauseField::CausePath(cause_path) => {
                    evaluate_path_as_string(input, cause_path)
                        .map_err(|err| FailStateExecutionError::MalformedCausePath(err))?
                }
            },
        };
        Ok(FailStateOutput { error, cause })
    }
}

#[derive(Error, Debug)]
enum HandlePathError {
    #[error("Evaluation error: {0}")]
    DynamicValueEvaluateError(#[from] DynamicValueEvaluateError),
    #[error("Evaluated value is not a string. Got: {0}")]
    ValueDoesntPointToAString(String),
}

fn evaluate_path_as_string(
    input: &ExecutionInput,
    path: &DynamicValue,
) -> Result<Option<String>, HandlePathError> {
    match path.evaluate(input)? {
        None => Ok(None),
        Some(val) => {
            let ret_string = val.as_str().map(|s| s.to_string());
            let ret = ret_string
                .ok_or_else(|| HandlePathError::ValueDoesntPointToAString(val.to_string()))?;
            Ok(Some(ret))
        }
    }
}
