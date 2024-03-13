use std::collections::HashMap;
use itertools::Itertools;
use thiserror::Error;
use serde::Deserialize;
use serde_json::{Error as SerdeError, Number, Value};
use crate::asl::states::{EndOrNext, State};

#[derive(Debug)]
struct Execution<'a> {
    definition: &'a StateMachineDefinition,
    current_state: Option<&'a State>,
    task_resource_execution: HandleTaskResourceFunction,
    input: Value,
}

enum ExecutionError {
    // TODO: ????

}

type HandleTaskResourceFunction = fn(&str, &Value) -> Result<Option<Value>, TaskExecutionError>;

impl <'a> Iterator for Execution<'a> {
    // type Item = Result<Execution<'a>, ExecutionError>;
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(state) = &self.current_state else {
            // State machine is finished
            return None;
        };

        // Execute current state
        let ret = match state {
            State::Task { resource, .. } => {
                let ret = (self.task_resource_execution)(resource, &self.input);
                let foo = ret.expect("Task resource execution failed!");
                let bar = foo.unwrap();
                bar
            }
            // State::Parallel { .. } => {
            //
            // }
            // State::Map { .. } => {
            //
            // }
            // State::Pass { .. } => {
            //
            // }
            // State::Wait { .. } => {
            //
            // }
            // State::Choice { .. } => {
            //
            // }
            // State::Succeed { .. } => {
            //
            // }
            // State::Fail { .. } => {
            //
            // }
            // _ => Some(Ok(Some(Value::Null)))
            _ => Value::Null // TODO: This is wrong. It's using Null
        };

        // Move to next state
        match state {
            State::Task { end_or_next, .. } |
            State::Parallel { end_or_next, .. } |
            State::Map { end_or_next, .. } |
            State::Pass { end_or_next, .. } |
            State::Wait { end_or_next, .. } => {
                match end_or_next {
                    EndOrNext::End(_) => {
                        self.current_state = None;
                    }
                    EndOrNext::Next(next_state) => {
                        self.current_state = self.definition.states.get(next_state)
                    }
                }
            }
            State::Succeed { .. } => {
                self.current_state = None;
            }
            State::Fail { .. } => {

            }
            _ => unreachable!("Bug!")
        }

        return Some(ret);
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

    fn start(&self, input: Value, x: HandleTaskResourceFunction) -> Execution {
        let start_state_name = &self.definition.start_at;
        let start_state = self.definition.states.get(start_state_name).expect(
            format!("Can't find the start state {start_state_name} from the list of states: {}. This ia bug in the parser.",
                    self.definition.states.keys().join(", ")).as_str());
        Execution {
            definition: &self.definition,
            current_state: Some(start_state),
            task_resource_execution: x,
            input,
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
            resource: String::from("return"),
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
    #[rstest]
    fn execute_hello_wolrd() -> Result<()> {
        let definition = include_str!("test-data/hello-world.json");

        let state_machine = StateMachine::parse(definition)?;

        fn task_resources(resource: &str, input: &Value) -> std::result::Result<Option<Value>, TaskExecutionError> {
            match resource {
                "Return" => {
                    if (input.is_null()) {
                        return Err(TaskExecutionError::TaskFailed("Null value is not accepted"));
                    }
                    Ok(Some(input.to_owned()))
                },
                _ => Err(TaskExecutionError::UnknownTaskResource(resource.to_string()))
            }
        }
        let input = serde_json::from_str(r#"
            "Hello world"
        "#)?;
        let mut execution = state_machine.start(input, task_resources);
        let foo = execution.next();
        assert_eq!(Some(Value::from("Hello world")), foo);
        assert_eq!(None, execution.next());
        Ok(())
    }

}
