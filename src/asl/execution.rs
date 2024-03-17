use crate::asl::state_machine::StateMachineDefinition;
use crate::asl::states::all_states::{EndOrNext, State};
use crate::asl::states::wait::WaitDuration;
use crate::asl::types::{ExecutionInput, StateMachineContext};
use itertools::Itertools;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;
use crate::asl::error_handling::StateMachineExecutionError;
use crate::asl::execution::ExecutionStatus::FinishedWithFailure;
use crate::asl::states::fail::{FailStateCauseField, FailStateErrorField};

pub struct Execution<'a, T>
where
    T: StateExecutionHandler,
{
    pub(crate) definition: &'a StateMachineDefinition,
    pub(crate) next_state_name: Option<&'a str>,
    pub(crate) state_execution_handler: T,
    pub(crate) input: Value,
    // If the state machine reached a Fail state
    pub status: ExecutionStatus, // TODO: review if I want this as `pub`
    pub(crate) context: Box<dyn StateMachineContext>,
}

#[derive(Error, Debug)]
enum StateExecutionError {}

/// The status of the state machine execution
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecutionStatus {
    // NotStarted,
    Executing,
    FinishedWithSuccess,
    FinishedWithFailure { error: Option<StateMachineExecutionError>, cause: Option<String> },
}

impl ExecutionStatus {
    pub fn with_error_and_cause(error: &str, cause: &str) -> ExecutionStatus {
        FinishedWithFailure {
            error: Some(StateMachineExecutionError::Custom(String::from(error))),
            cause: Some(String::from(cause))
        }
    }
}

///
pub trait StateExecutionHandler {
    /// TODO: document
    fn execute_task(
        &self,
        resource: &str,
        input: &Value,
    ) -> Result<Option<Value>, TaskExecutionError>;

    /// TODO: document
    fn wait(&self, seconds: f64) {
        std::thread::sleep(Duration::from_secs_f64(seconds));
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StateExecutionOutput {
    pub status: ExecutionStatus,
    pub state_name: Option<String>,
    pub result: Option<Value>,
}

impl<'a, T> Iterator for Execution<'a, T>
where
    T: StateExecutionHandler,
{
    type Item = StateExecutionOutput;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(state_to_execute_name) = self.next_state_name else {
            // State machine is finished
            return None;
        };

        // TODO: implement handling of input transformation
        let state_input = &self.input;
        let state_to_execute = self.definition.states
                    .get(state_to_execute_name)
                    .unwrap_or_else(|| panic!("Can't find state '{state_to_execute_name}' (next state) in list of states. Current state is: {0}. This is a validation bug.", self.next_state_name.unwrap()));

        // Execute state
        let state_execution_output = match state_to_execute {
            State::Task { resource, .. } => {
                let ret = self
                    .state_execution_handler
                    .execute_task(resource, state_input);
                // TODO: handle catch options in Task
                let task_result = ret.expect("Task resource execution failed!");
                task_result
            }
            State::Parallel { .. } => {
                //TODO: implement
                None
            }
            State::Map { .. } => {
                //TODO: implement
                None
            }
            State::Pass { .. } => None,
            State::Wait { duration, .. } => {
                // TODO: resolve the wait duration properly:
                //       - Implement JSON path
                //       - Implement timestamp type
                //
                let resolved_duration = if let WaitDuration::Seconds(duration_number) = duration {
                    duration_number.as_f64().expect("Invalid duration number")
                } else {
                    // TODO: Remove this after implementing the above TODOs. For now just wait 100ms
                    //       if asked to wait
                    0.1f64
                };
                self.state_execution_handler.wait(resolved_duration);
                None // Waits return nothing
            }
            State::Choice { .. } => None, // Choice doesn't execute anything
            State::Succeed { .. } => None,
            State::Fail { .. } => None,
        };

        // TODO: mix state_execution_output with input (ResultPath, etc.)
        let task_output = state_execution_output.unwrap_or(state_input.clone());

        // Move to next state
        match state_to_execute {
            State::Task { end_or_next, .. }
            | State::Parallel { end_or_next, .. }
            | State::Map { end_or_next, .. }
            | State::Pass { end_or_next, .. }
            | State::Wait { end_or_next, .. } => match end_or_next {
                EndOrNext::End(_) => {
                    self.next_state_name = None;
                }
                EndOrNext::Next(next_state) => {
                    self.next_state_name = Some(next_state);
                }
            },
            State::Choice {
                choices, default, ..
            } => {
                let matched_choice = choices.iter().find(|choice| {
                    let result = choice.evaluate(&ExecutionInput {
                        value: &task_output,
                        context: &self.context,
                    });
                    let b = result.expect("Error evaluating choice"); //TODO: Handle the Result<>
                    b
                });
                match matched_choice {
                    None => {
                        // No choice matched the rule, so use the Default field if it was
                        match default {
                            None => {
                                //TODO: Maybe we want to use the fallible_iterators crate for this situation.
                                self.next_state_name = None;
                                self.status = FinishedWithFailure { error: Some(StateMachineExecutionError::StatesNoChoiceMatched), cause: None };
                            }
                            Some(default_state_name) => {
                                self.next_state_name = Some(default_state_name);
                            }
                        }
                    }
                    Some(choice) => {
                        self.next_state_name = Some(&choice.next);
                    }
                }
            }
            State::Succeed { .. } => {
                self.next_state_name = None;
            }
            State::Fail { error, cause, .. } => {
                // A Fail state MAY have "ErrorPath" and "CausePath" fields whose values MUST be
                // Reference Paths or Intrinsic Functions which, when resolved, MUST be string values.
                // A Fail state MUST NOT include both "Error" and "ErrorPath" or both "Cause" and "CausePath".
                let error = error.as_ref().map(|e| {
                    let FailStateErrorField::Error(e) = e else {
                        todo!()
                    };
                    StateMachineExecutionError::Custom(e.clone())
                });
                let cause = cause.as_ref().map(|e| {
                    let FailStateCauseField::Cause(e) = e else {
                        todo!()
                    };
                    e.clone()
                });
                self.status = FinishedWithFailure{ error, cause };
                self.next_state_name = None;
            }
        }

        //Update status, unless it's already marked as a failure.
        match self.status {
            FinishedWithFailure { .. } => {}
            _ => {
                self.status = match self.next_state_name {
                    None => ExecutionStatus::FinishedWithSuccess,
                    Some(_) => ExecutionStatus::Executing,
                }
            }
        }
        let ret = StateExecutionOutput {
            status: self.status.clone(),
            state_name: Some(state_to_execute_name.to_owned()),
            result: Some(task_output), //TODO: This is probably wrong
        };
        Some(ret)
    }
}

#[derive(Error, Debug)]
pub enum TaskExecutionError {
    #[error("Task resource {0} is not recognized by the user function.")]
    UnknownTaskResource(String),

    #[error("Task failed with error: {0}")]
    TaskFailed(&'static str),
}
