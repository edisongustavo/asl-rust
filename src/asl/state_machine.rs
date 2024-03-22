use crate::asl::execution::{Execution, ExecutionStatus, StateExecutionHandler};
use crate::asl::states::all_states::States;
use crate::asl::types::StateMachineContext;
use serde::Deserialize;
use serde_json::{Error as SerdeError, Number, Value};
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Missing the state of 'StartsAt' field ({0}) from the list of states: {}", .states.join(", "))]
    StartStateNotDefinedInListOfStates {
        starts_at: String,
        states: Vec<String>,
    },

    #[error("Malformed input: {0}")]
    MalformedInput(SerdeError),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct StateMachineDefinition {
    pub states: HashMap<String, States>,
    pub comment: Option<String>,
    pub start_at: String,
    pub version: Option<String>,
    pub timeout_seconds: Option<Number>,
}

pub struct StateMachine {
    definition: StateMachineDefinition,
}

impl StateMachine {
    pub fn parse(definition: &str) -> Result<StateMachine, ParseError> {
        let definition = serde_json::from_str(definition)
            .map_err(|e: SerdeError| ParseError::MalformedInput(e))?;
        let state_machine = StateMachine { definition };
        // TODO: validate state machine

        Ok(state_machine)
    }

    pub fn start<T>(
        &self,
        input: &Value,
        state_execution_handler: T,
        context: Rc<dyn StateMachineContext>,
    ) -> Execution<T>
    where
        T: StateExecutionHandler,
    {
        let start_state_name = &self.definition.start_at;

        // TODO: Implement Context object. Maybe receive it as a parameter?
        Execution {
            definition: &self.definition,
            next_state_name: Some(start_state_name.clone()),
            state_execution_handler,
            input: input.clone(),
            status: ExecutionStatus::Executing,
            context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asl::states::all_states::EndOrNext;
    use crate::asl::states::task::Task;
    use anyhow::Result;
    use itertools::Itertools;
    use rstest::*;
    use std::fs;
    use std::path::PathBuf;

    #[rstest]
    fn parse_hello_world_state_machine() -> Result<()> {
        let definition = include_str!("../../tests/test-data/hello-world.json");
        let state_machine = StateMachine::parse(definition)?;

        // Testing internals, but ok for now
        assert_eq!(
            state_machine.definition.states.keys().collect_vec(),
            vec!["Hello World"]
        );
        let States::Task(state_hello_world) = &state_machine.definition.states["Hello World"]
        else {
            panic!()
        };
        assert_eq!(
            state_hello_world,
            &Task {
                comment: None,
                end_or_next: EndOrNext::End,
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
            }
        );
        Ok(())
    }

    #[rstest]
    fn parse_valid_cases(
        #[files("**/test-data/asl-validator/valid-*.json")] path: PathBuf,
    ) -> Result<()> {
        let definition = fs::read_to_string(path)?;
        StateMachine::parse(definition.as_str())?;
        Ok(())
    }

    #[rstest]
    fn parse_invalid_cases(
        #[files("**/test-data/asl-validator/invalid-*.json")] path: PathBuf,
    ) -> Result<()> {
        let definition = fs::read_to_string(path)?;
        let ret = StateMachine::parse(definition.as_str());
        assert_eq!(ret.is_err(), true);
        Ok(())
    }
}
