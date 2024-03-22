use crate::asl::error_handling::StateMachineExecutionPredefinedErrors;
use crate::asl::execution::ExecutionStatus::{Executing, FinishedWithFailure};
use crate::asl::state_machine::StateMachineDefinition;
use crate::asl::states::all_states::{EndOrNext, States};
use crate::asl::states::map::Map;
use crate::asl::states::parallel::Parallel;
use crate::asl::states::pass::Pass;
use crate::asl::types::{ExecutionInput, StateMachineContext};
use crate::asl::HandlerOutput;
use serde_json::Value;
use std::fmt::Display;
use std::rc::Rc;
use std::time::Duration;
use thiserror::Error;

pub struct Execution<'a, H>
where
    H: StateExecutionHandler,
{
    pub(crate) definition: &'a StateMachineDefinition,
    pub(crate) next_state_name: Option<String>,
    pub(crate) state_execution_handler: H,
    pub(crate) input: Value,
    pub status: ExecutionStatus, // TODO: review if I want this as `pub`
    pub(crate) context: Rc<dyn StateMachineContext>,
}

#[derive(Error, Debug)]
enum StateExecutionError {}

/// The status of the state machine handlers
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ExecutionStatus {
    // NotStarted,
    Executing,
    FinishedWithSuccess(Option<Value>),
    // FinishedWithFailure {
    //     error: Option<StateMachineExecutionPredefinedErrors>,
    //     cause: Option<String>,
    // },
    FinishedWithFailure(StateMachineExecutionError),
}

/// TODO: document
pub trait StateExecutionHandler {
    /// TODO: document
    type TaskExecutionError: Display;

    /// TODO: document
    fn execute_task(
        &self,
        resource: &str,
        input: &Value,
        credentials: &Option<Value>,
    ) -> Result<Option<Value>, Self::TaskExecutionError>;

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

impl<'a, H> Iterator for Execution<'a, H>
where
    H: StateExecutionHandler,
{
    type Item = StateExecutionOutput;

    fn next(&mut self) -> Option<Self::Item> {
        let (state_input, state_name, state) = self.fetch_next_state()?;

        let state_execution_result = self.execute_state(&state_input, state);

        self.status = Self::calculate_status(&state_execution_result);
        self.next_state_name = Self::decide_next_state(&state_execution_result);

        let result = state_execution_result.ok().and_then(|val| val.output);
        let output = StateExecutionOutput {
            status: self.status.clone(),
            state_name: Some(state_name),
            result,
        };
        Some(output)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StateMachineExecutionError {
    pub error: StateMachineExecutionPredefinedErrors,
    pub cause: Option<String>,
}

type ExecuteStateResult = Result<HandlerOutput, StateMachineExecutionError>;

impl<'a, H> Execution<'a, H>
where
    H: StateExecutionHandler,
{
    fn fetch_next_state(&self) -> Option<(ExecutionInput, String, &States)> {
        if self.next_state_name.is_none() {
            return None;
        }

        let name = self.next_state_name.clone().unwrap(); //TODO: I gave up fighting the borrow  checker :(

        // TODO: implement handling of input transformation
        let input = ExecutionInput {
            value: self.input.clone(), //TODO: extra clone :(
            context: self.context.clone(),
        };
        let state = self.definition.states.get(&name).unwrap_or_else(|| {
            panic!(
                "Can't find state '{name}' (next state) in list of states. \
                                    Current state is: {0}. This is a validation bug.",
                self.next_state_name.as_ref().unwrap()
            )
        });
        Some((input, name, state))
    }

    fn decide_next_state(state_execution_result: &ExecuteStateResult) -> Option<String> {
        match state_execution_result {
            Ok(handler_output) => match &handler_output.next_state {
                EndOrNext::End => None,
                EndOrNext::Next(next_state_name) => Some(next_state_name.clone()),
            },
            Err(_) => None,
        }
    }

    fn calculate_status(state_execution_result: &ExecuteStateResult) -> ExecutionStatus {
        match state_execution_result {
            Ok(handler_output) => match handler_output.next_state {
                EndOrNext::End => {
                    ExecutionStatus::FinishedWithSuccess(handler_output.output.clone())
                }
                EndOrNext::Next(_) => Executing,
            },
            Err(err) => FinishedWithFailure(err.clone()),
        }
    }

    fn execute_state(
        &self,
        state_input: &ExecutionInput,
        state_to_execute: &States,
    ) -> ExecuteStateResult {
        match state_to_execute {
            States::Task(task) => Ok(self.handle_task(task, state_input)?),
            States::Parallel(Parallel { end_or_next, .. }) => {
                //TODO: implement
                Ok(HandlerOutput {
                    output: None,
                    next_state: end_or_next.clone(),
                })
            }
            States::Map(Map { end_or_next, .. }) => {
                //TODO: implement
                Ok(HandlerOutput {
                    output: None,
                    next_state: end_or_next.clone(),
                })
            }
            States::Pass(Pass { end_or_next, .. }) => Ok(HandlerOutput {
                output: None,
                next_state: end_or_next.clone(),
            }),
            States::Wait(wait) => Ok(self.handle_wait(wait, state_input)?),
            States::Choice(choice) => Ok(self.handle_choice(choice, state_input)?),
            States::Succeed(_succeed) => Ok(HandlerOutput {
                // According to https://docs.aws.amazon.com/step-functions/latest/apireference/API_DescribeExecution.html#StepFunctions-DescribeExecution-response-output
                // the 'output' field will have a value if the execution succeeds
                output: Some(state_input.value.clone()), // Pass over the value of the input
                next_state: EndOrNext::End,
            }),
            States::Fail(fail) => {
                let output = self.handle_fail(fail, state_input)?;
                Err(StateMachineExecutionError {
                    error: StateMachineExecutionPredefinedErrors::Custom(output.error),
                    cause: output.cause,
                })
            }
        }
    }
}
