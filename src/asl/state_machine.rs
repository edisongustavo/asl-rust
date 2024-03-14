use crate::asl::states::choice::{ChoiceExpression, Operation};
use std::collections::HashMap;
use std::time::Duration;
use itertools::Itertools;
use thiserror::Error;
use serde::Deserialize;
use serde_json::{Error as SerdeError, Number, Value};
use crate::asl::states::{EndOrNext, State};
use crate::asl::states::wait::WaitDuration;

#[derive(Debug)]
struct Execution<'a, T>
    where T: StateExecutionHandler
{
    definition: &'a StateMachineDefinition,
    next_state: Option<&'a State>,
    next_state_name: Option<&'a str>,
    state_execution_handler: T,
    input: Value,
    // If the state machine reached a Fail state
    state_machine_reached_fail_state: bool,
}

#[derive(Error, Debug)]
enum StateExecutionError {}

impl<'a, T> Execution<'a, T>
    where T: StateExecutionHandler
{
    fn status(&self) -> ExecutionStatus {
        match self.next_state {
            None => {
                if self.state_machine_reached_fail_state {
                    ExecutionStatus::FinishedWithFailure
                } else {
                    ExecutionStatus::FinishedWithSuccess
                }
            }
            Some(_) => ExecutionStatus::Executing,
        }
    }

    fn next_state_name(&self) -> Option<&str> {
        self.next_state_name
    }
}

/// The status of the state machine execution
#[derive(Debug, Eq, PartialEq)]
enum ExecutionStatus {
    // NotStarted,
    Executing,
    FinishedWithSuccess,
    FinishedWithFailure,
}

///
pub trait StateExecutionHandler {
    /// TODO: document
    fn execute_task(&self, resource: &str, input: &Value) -> Result<Option<Value>, TaskExecutionError>;

    /// TODO: document
    fn wait(&self, seconds: f64) -> Result<(), WaitExecutionError> {
        std::thread::sleep(Duration::from_secs_f64(seconds));
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
enum StateExecutionOutput {
    Result(Value),
    Failure,
}

impl<'a, T> Iterator for Execution<'a, T>
    where T: StateExecutionHandler
{
    // type Item = Result<Execution<'a>, ExecutionError>;
    // type Item = Value;
    type Item = StateExecutionOutput;
    // type Item = Result<Option<Value>, ExecutionError>; // TODO: Make this work

    fn next(&mut self) -> Option<Self::Item> {
        let Some(state_to_execute) = &self.next_state else {
            // State machine is finished
            return None;
        };

        // TODO: implement handling of input transformation
        let state_input = &self.input;

        // Execute state
        let state_execution_output = match state_to_execute {
            State::Task { resource, .. } => {
                let ret = self.state_execution_handler.execute_task(resource, state_input);
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
            State::Pass { .. } => {
                //TODO: implement
                None
            }
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

                let ret = self.state_execution_handler.wait(resolved_duration);
                if ret.is_err() {
                    self.state_machine_reached_fail_state = false;
                }
                ret.expect("Wait failed!"); // TODO: handle the Result<>
                None // Waits return nothing
            }
            State::Choice { .. } => None, // Choice doesn't execute anything
            State::Succeed { .. } => None,
            State::Fail { .. } => None,
        };

        // TODO: mix state_execution_output with input (ResultPath, etc.)
        let output = state_execution_output.unwrap_or(state_input.clone());

        // Move to next state
        match state_to_execute {
            State::Task { end_or_next, .. } |
            State::Parallel { end_or_next, .. } |
            State::Map { end_or_next, .. } |
            State::Pass { end_or_next, .. } |
            State::Wait { end_or_next, .. } => {
                match end_or_next {
                    EndOrNext::End(_) => {
                        self.next_state = None;
                    }
                    EndOrNext::Next(next_state) => {
                        self.next_state_name = Some(next_state);
                        self.next_state = Some(
                            self.definition.states
                                .get(next_state)
                                .unwrap_or_else(|| panic!("Can't find state '{next_state}' (next state) in list of states. Current state is: {0}. This is a validation bug.", self.next_state_name.unwrap()))
                        );
                    }
                }
            }
            State::Choice { choices, .. } => {
                // TODO:
                let variable_value = output.clone(); //TODO: get variable value from output

                for choice in choices {
                    let result = choice.evaluate(&variable_value);
                    let b = result.expect("Error evaluating choice"); //TODO: Handle the Result<>
                    if b {
                        break
                    }
                }
            }
            State::Succeed { .. } => {
                self.next_state = None;
            }
            State::Fail { .. } => {
                self.state_machine_reached_fail_state = true;
                self.next_state = None;
                return Some(StateExecutionOutput::Failure);
            }
        }

        if self.next_state.is_none() {
            self.next_state_name = None;
        }

        Some(StateExecutionOutput::Result(output))
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing the state of 'StartsAt' field ({0}) from the list of states: {}", .states.join(", "))]
    StartStateNotDefinedInListOfStates { starts_at: String, states: Vec<String> },

    #[error("Malformed input: {0}")]
    MalformedInput(SerdeError),
}

#[derive(Error, Debug)]
pub enum TaskExecutionError {
    #[error("Task resource {0} is not recognized by the user function.")]
    UnknownTaskResource(String),

    #[error("Task failed with error: {0}")]
    TaskFailed(&'static str),
}

#[derive(Error, Debug)]
pub enum WaitExecutionError {
    #[error("Wait error.")]
    Unknown // TODO: What kind of wait errors we can expect?
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct StateMachineDefinition {
    states: HashMap<String, State>,
    comment: Option<String>,
    start_at: String,
    version: Option<String>,
    timeout_seconds: Option<Number>,
}

struct StateMachine {
    definition: StateMachineDefinition,
}

impl StateMachine {
    fn parse(definition: &str) -> Result<StateMachine, ParseError> {
        let definition = serde_json::from_str(definition).map_err(|e: SerdeError| ParseError::MalformedInput(e))?;
        let state_machine = StateMachine {
            definition
        };
        // TODO: validate state machine

        Ok(state_machine)
    }

    fn start<T>(&self, input: Value, state_execution_handler: T) -> Execution<T>
        where T: StateExecutionHandler
    {
        let start_state_name = &self.definition.start_at;
        let start_state = self.definition.states.get(start_state_name).expect(
            format!("Can't find the start state {start_state_name} from the list of states: {}. This ia bug in the parser.",
                    self.definition.states.keys().join(", ")).as_str());
        Execution {
            definition: &self.definition,
            next_state: Some(start_state),
            next_state_name: Some(start_state_name),
            state_execution_handler,
            input,
            state_machine_reached_fail_state: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use super::*;
    use rstest::*;
    use itertools::Itertools;
    use anyhow::Result;
    use serde_json::Map;
    use crate::asl::states::EndOrNext;

    #[rstest]
    fn parse_hello_world_state_machine() -> Result<()> {
        let definition = include_str!("test-data/hello-world.json");
        let state_machine = StateMachine::parse(definition)?;

        // Testing internals, but ok for now
        assert_eq!(state_machine.definition.states.keys().collect_vec(), vec!["Hello World"]);
        let state_hello_world = &state_machine.definition.states["Hello World"];
        assert_eq!(state_hello_world, &State::Task {
            comment: None,
            end_or_next: EndOrNext::End(true),
            resource: String::from("Return"),
            credentials: None,
            input_path: None,
            output_path: None,
            result_path: None,
            parameters: None,
            result_selector: None,
            retry: None,
            catch: None,
            heartbeat: None,
            // timeout: Some(TimeoutSeconds(60)), //TODO: Check why this is not the case
            timeout: None,
        });
        Ok(())
    }

    #[rstest]
    fn parse_valid_cases(#[files("src/**/test-data/asl-validator/valid-*.json")] path: PathBuf) -> Result<()> {
        let definition = fs::read_to_string(path)?;
        StateMachine::parse(definition.as_str())?;
        Ok(())
    }

    #[rstest]
    fn parse_invalid_cases(#[files("src/**/test-data/asl-validator/invalid-*.json")] path: PathBuf) -> Result<()> {
        let definition = fs::read_to_string(path)?;
        let ret = StateMachine::parse(definition.as_str());
        assert_eq!(ret.is_err(), true);
        Ok(())
    }

    // #[rstest]
    // fn execute_valid_cases(#[files("src/**/test-data/asl-validator/valid-*.json")] path: PathBuf) -> Result<()> {
    //     let definition = fs::read_to_string(path)?;
    //     let state_machine = StateMachine::parse(definition.as_str())?;
    //     state_machine.start(serde_json::from_str(r#"
    //             "#)?,
    //     );
    //     Ok(())
    // }

    struct TestStateExecutionHandler {}

    impl StateExecutionHandler for TestStateExecutionHandler {
        fn execute_task(&self, resource: &str, input: &Value) -> std::result::Result<Option<Value>, TaskExecutionError> {
            // let mut ret = input.clone();
            // let ret = match input {
            //     Value::Object(mut map) => {
            //         let values = map.get_mut("results").unwrap_or(Value::Array(vec![]));
            //         map.insert("results", values);
            //     }
            //     _ => ret
            // };
            // Ok(Some(ret))
            // match resource {
            //     "Return" => {
            //         if input.is_null() {
            //             return Err(TaskExecutionError::TaskFailed("Null value is not accepted"));
            //         }
            //         Ok(Some(input.to_owned()))
            //     }
            //     _ => Err(TaskExecutionError::UnknownTaskResource(resource.to_string()))
            // }
            Ok(Some(input.to_owned()))
        }
    }

    #[rstest]
    fn execute_hello_world() -> Result<()> {
        let definition = include_str!("test-data/hello-world.json");
        let state_machine = StateMachine::parse(definition)?;

        let input = serde_json::from_str(r#"
            "Hello world"
        "#)?;
        let mut execution = state_machine.start(input, TestStateExecutionHandler {});
        assert_eq!(ExecutionStatus::Executing, execution.status());
        // assert_eq!(None, execution.current_state_name());
        assert_eq!(Some("Hello World"), execution.next_state_name());

        let val = Value::from("Hello world");
        assert_eq!(Some(StateExecutionOutput::Result(val)), execution.next());
        assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status());
        assert_eq!(None, execution.next_state_name());

        assert_eq!(None, execution.next());
        assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status());
        assert_eq!(None, execution.next_state_name());
        Ok(())
    }

    #[rstest]
    fn execute_hello_world_succeed_state() -> Result<()> {
        let definition = include_str!("test-data/hello-world-succeed-state.json");
        let state_machine = StateMachine::parse(definition)?;

        let input = serde_json::from_str(r#"
            "Hello world"
        "#)?;
        let mut execution = state_machine.start(input, TestStateExecutionHandler {});
        assert_eq!(ExecutionStatus::Executing, execution.status());

        let val = Value::from("Hello world");
        assert_eq!(Some(StateExecutionOutput::Result(val)), execution.next());
        assert_eq!(ExecutionStatus::Executing, execution.status());

        let val = Value::from("Hello world");
        assert_eq!(Some(StateExecutionOutput::Result(val)), execution.next());
        assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status());

        assert_eq!(None, execution.next());
        assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status());
        Ok(())
    }

    #[rstest]
    fn execute_hello_world_fail_state() -> Result<()> {
        let definition = include_str!("test-data/hello-world-fail-state.json");
        let state_machine = StateMachine::parse(definition)?;

        let input = serde_json::from_str(r#"
            "Hello world"
        "#)?;
        let mut execution = state_machine.start(input, TestStateExecutionHandler {});
        assert_eq!(ExecutionStatus::Executing, execution.status());
        assert_eq!(None, execution.next_state_name());

        assert_eq!(Some(StateExecutionOutput::Result(Value::from("Hello world"))), execution.next());
        assert_eq!(ExecutionStatus::Executing, execution.status());
        // assert_eq!(Some("Hello World"), execution.current_state_name());

        assert_eq!(Some(StateExecutionOutput::Failure), execution.next());
        assert_eq!(ExecutionStatus::FinishedWithFailure, execution.status());

        assert_eq!(None, execution.next());
        assert_eq!(ExecutionStatus::FinishedWithFailure, execution.status());

        // TODO: How to provide execution history? An external wrapper?
        // assert_eq!(vec![
        //     "Task: Hello World",
        //     "Fail: Fail State",
        // ], execution.execution_history());
        Ok(())
    }

    #[rstest]
    fn execute_choice() -> Result<()> {
        let definition = include_str!("test-data/asl-validator/valid-choice-state.json");
        let state_machine = StateMachine::parse(definition)?;

        let input = serde_json::from_str(r#"
            {"foo": 1}
        "#)?;
        let mut execution = state_machine.start(input, TestStateExecutionHandler {});
        let val = Value::from("Hello world");
        assert_eq!(Some(StateExecutionOutput::Result(val)), execution.next());
        assert_eq!(ExecutionStatus::Executing, execution.status());
        assert_eq!(Some("ChoiceState"), execution.next_state_name());

        assert_eq!(None, execution.next());
        assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status());
        Ok(())
    }

    //
    // #[rstest]
    // fn execute_catch_failure() -> Result<()> {
    //     let definition = include_str!("test-data/hello-world.json");
    //     let state_machine = StateMachine::parse(definition)?;
    //
    //     let input = serde_json::from_str(r#"
    //         "Hello world"
    //     "#)?;
    //     let mut execution = state_machine.start(input, TestStateExecutionHandler {});
    //     assert_eq!(Some(StateResult::with_value(Value::from("Hello world"))), execution.next());
    //     assert_eq!(None, execution.next());
    //     Ok(())
    // }
}
