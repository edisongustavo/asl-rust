use thiserror::Error;
use crate::asl::error_handling::{matches_error, StateMachineExecutionPredefinedErrors};
use crate::asl::execution::{Execution, StateExecutionHandler, StateMachineExecutionError};
use crate::asl::states::all_states::EndOrNext;
use crate::asl::states::task::Task;
use crate::asl::types::ExecutionInput;
use crate::asl::HandlerOutput;

#[derive(Error, Debug)]
pub(crate) enum TaskStateExecutionError {
    #[error("No catcher matched")]
    NoCatcherMatched,

    #[error("No catcher available")]
    NoCatchersAvailable,
}

impl From<TaskStateExecutionError> for StateMachineExecutionError {
    fn from(value: TaskStateExecutionError) -> Self {
        StateMachineExecutionError {
            error: StateMachineExecutionPredefinedErrors::StatesTaskFailed,
            cause: Some(value.to_string())
        }
    }
}

impl<'a, H: StateExecutionHandler> Execution<'a, H> {
    pub fn handle_task(
        &self,
        task: &Task,
        input: &ExecutionInput,
    ) -> Result<HandlerOutput, TaskStateExecutionError> {
        // TODO: transform input with input_path and parameters. Handle Context too.
        let task_input = input.value.clone();

        // TODO: implement timeout, heartbeat
        let task_result = self.state_execution_handler.execute_task(
            &task.resource,
            &task_input,
            &task.credentials,
        );

        match task_result {
            Ok(output) => {
                //TODO: transform output with output_path, result_path
                let ret = HandlerOutput {
                    output,
                    next_state: task.end_or_next.clone(),
                };
                Ok(ret)
            }
            Err(user_task_execution_error) => {
                // function failed
                match &task.catch {
                    Some(catchers) => {
                        for catcher in catchers {
                            if catcher
                                .error_equals
                                .iter()
                                .any(|catcher_error| matches_error(&user_task_execution_error.to_string(), catcher_error))
                            {
                                let next_state = EndOrNext::Next(catcher.next.clone());
                                let ret = HandlerOutput {
                                    output: None,
                                    next_state,
                                };
                                return Ok(ret);
                            }
                        }
                        // No catcher matched
                        Err(TaskStateExecutionError::NoCatcherMatched)
                    }
                    // No catchers available
                    None => Err(TaskStateExecutionError::NoCatchersAvailable),
                }
            }
        }
    }
}
