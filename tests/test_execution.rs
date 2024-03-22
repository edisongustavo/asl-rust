use serde_json::value::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use asl::asl::execution::{ExecutionStatus, StateMachineExecutionError};
use asl::asl::execution::{StateExecutionHandler, StateExecutionOutput};
use asl::asl::state_machine::StateMachine;

struct TestStateExecutionHandler {
    resource_name_to_output: HashMap<String, TaskBehavior>,
}

impl TestStateExecutionHandler {
    fn new() -> TestStateExecutionHandler {
        TestStateExecutionHandler {
            resource_name_to_output: hash_map![],
        }
    }
    fn with_map(
        resource_name_to_output: HashMap<String, TaskBehavior>,
    ) -> TestStateExecutionHandler {
        TestStateExecutionHandler {
            resource_name_to_output,
        }
    }
}

#[derive(Error, Debug)]
enum MyTaskExecutionError {
    #[error("Could not execute due to {0}")]
    Foo(String),
}

impl StateExecutionHandler for TestStateExecutionHandler {
    type TaskExecutionError = MyTaskExecutionError;

    fn execute_task(
        &self,
        resource: &str,
        input: &Value,
        _credentials: &Option<Value>,
    ) -> Result<Option<Value>, Self::TaskExecutionError> {
        let option = self.resource_name_to_output.get(resource);
        match option {
            None => Ok(Some(input.to_owned())), // resource is not mapped, so just forwards the input
            Some(desired_output) => match desired_output {
                TaskBehavior::Output(val) => Ok(Some(val.to_owned())), // resource is mapped, so returns the desired output
                TaskBehavior::Error(err) => Err(MyTaskExecutionError::Foo(err.clone())),
            },
        }
    }

    fn wait(&self, _seconds: f64) {
        //nop on purpose (sleeping a thread is bad for tests).
    }
}

use anyhow::Result;
use asl::asl::error_handling::StateMachineExecutionPredefinedErrors;
use asl::asl::execution::ExecutionStatus::FinishedWithFailure;
use asl::asl::types::EmptyContext;
use itertools::Itertools;
use map_macro::hash_map;
use rstest::*;
use serde_with::serde_derive::Deserialize;
use thiserror::Error;

#[rstest]
fn execute_hello_world() -> Result<()> {
    let definition = include_str!("test-data/hello-world.json");
    let state_machine = StateMachine::parse(definition)?;

    let input = serde_json::from_str(
        r#"
            "Hello world"
        "#,
    )?;
    let mut execution = state_machine.start(
        &input,
        TestStateExecutionHandler::new(),
        Rc::new(EmptyContext {}),
    );

    let val = Value::from("Hello world");

    let state_output = execution.next();
    assert_eq!(
        Some(StateExecutionOutput {
            status: ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
            state_name: Some("Hello World".to_string()),
            result: Some(val.clone())
        }),
        state_output,
    );
    assert_eq!(
        ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
        execution.status
    );

    let state_output = execution.next();
    assert_eq!(state_output, None);
    assert_eq!(
        ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
        execution.status
    );
    Ok(())
}

#[rstest]
fn execute_hello_world_succeed_state() -> Result<()> {
    let definition = include_str!("test-data/hello-world-succeed-state.json");
    let state_machine = StateMachine::parse(definition)?;

    let input = serde_json::from_str(
        r#"
            "Hello world"
        "#,
    )?;
    let val = Value::from("Hello world");

    let mut execution = state_machine.start(
        &input,
        TestStateExecutionHandler::new(),
        Rc::new(EmptyContext {}),
    );
    assert_eq!(ExecutionStatus::Executing, execution.status);

    // Advance state
    let state_output = execution.next();
    assert_eq!(
        state_output,
        Some(StateExecutionOutput {
            status: ExecutionStatus::Executing,
            state_name: Some("Hello World".to_string()),
            result: Some(val.clone())
        })
    );
    assert_eq!(ExecutionStatus::Executing, execution.status);

    // Advance state
    let state_output = execution.next();
    assert_eq!(
        state_output,
        Some(StateExecutionOutput {
            status: ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
            state_name: Some("Succeed State".to_string()),
            result: Some(val.clone())
        })
    );
    assert_eq!(
        ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
        execution.status
    );

    // Iterator is exhausted
    assert_eq!(None, execution.next());
    assert_eq!(
        ExecutionStatus::FinishedWithSuccess(Some(val.clone())),
        execution.status
    );
    Ok(())
}

#[rstest]
fn execute_hello_world_fail_state() -> Result<()> {
    let definition = include_str!("test-data/hello-world-fail-state.json");
    let state_machine = StateMachine::parse(definition)?;

    let val = serde_json::from_str(
        r#"
            "Hello world"
        "#,
    )?;
    let mut execution = state_machine.start(
        &val,
        TestStateExecutionHandler::new(),
        Rc::new(EmptyContext {}),
    );
    assert_eq!(ExecutionStatus::Executing, execution.status);

    // Advance state
    let state_output = execution.next();
    assert_eq!(
        state_output,
        Some(StateExecutionOutput {
            status: ExecutionStatus::Executing,
            state_name: Some("Hello World".to_string()),
            result: Some(val.clone())
        })
    );
    assert_eq!(ExecutionStatus::Executing, execution.status);

    // Advance state
    let state_output = execution.next();
    let expected_status = with_error_and_cause("ErrorA", "Kaiju attack");
    assert_eq!(
        state_output,
        Some(StateExecutionOutput {
            status: expected_status.clone(),
            state_name: Some("Fail State".to_string()),
            result: None,
        })
    );
    assert_eq!(expected_status, execution.status);

    // Iterator is exhausted
    assert_eq!(None, execution.next());
    assert_eq!(expected_status, execution.status);
    Ok(())
}

pub fn with_error_and_cause(error: &str, cause: &str) -> ExecutionStatus {
    FinishedWithFailure(StateMachineExecutionError {
        error: StateMachineExecutionPredefinedErrors::Custom(error.to_string()),
        cause: Some(String::from(cause)),
    })
}
pub fn with_success_and_output(output: &str) -> ExecutionStatus {
    let val = serde_json::from_str(output).expect("Invalid json specified");
    ExecutionStatus::FinishedWithSuccess(val)
}

// HACK: This is a kind of
#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum FinalStatus {
    Output(Value),
    #[serde(rename_all = "PascalCase")]
    Error {
        error: Option<StateMachineExecutionPredefinedErrors>,
        cause: Option<String>,
    },
}

#[derive(Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum TaskBehavior {
    Output(Value),
    Error(String),
}

#[derive(Deserialize, Debug)]
struct ExpectedExecution {
    input: Value,
    #[serde(flatten)]
    final_status: FinalStatus,
    states: Vec<String>,
    task_behavior: Option<HashMap<String, TaskBehavior>>,
}

impl PartialEq<ExecutionStatus> for FinalStatus {
    fn eq(&self, other: &ExecutionStatus) -> bool {
        match self {
            FinalStatus::Output(expected_value) => match other {
                ExecutionStatus::Executing => false,
                ExecutionStatus::FinishedWithSuccess(final_value) => final_value
                    .as_ref()
                    .map(|v| v == expected_value)
                    .unwrap_or(false),
                FinishedWithFailure { .. } => false,
            },
            FinalStatus::Error { error, cause, .. } => match other {
                ExecutionStatus::Executing => false,
                ExecutionStatus::FinishedWithSuccess(_) => false,
                FinishedWithFailure(StateMachineExecutionError {
                    error: other_error,
                    cause: other_cause,
                }) => {
                    error.as_ref().map(|e| e == other_error).unwrap_or(false)
                        && cause == other_cause
                }
            },
        }
    }
}

#[rstest]
fn execute_all(
    #[files("**/test-data/expected-executions-valid-cases/valid-*.json")] path: PathBuf,
) -> Result<()> {
    let all_expected_executions: Vec<ExpectedExecution> =
        serde_json::from_str(&fs::read_to_string(&path)?)?;

    let filename = path.file_name().unwrap();
    let definition = fs::read_to_string(format!(
        "tests/test-data/asl-validator/{}",
        filename.to_str().unwrap()
    ))?;
    let state_machine = StateMachine::parse(&definition)?;

    for (i, execution_expected_input) in all_expected_executions.iter().enumerate() {
        let input = &execution_expected_input.input;
        let map = execution_expected_input
            .task_behavior
            .clone()
            .unwrap_or(hash_map![]);
        let handler = TestStateExecutionHandler::with_map(map);
        let execution = state_machine.start(&input, handler, Rc::new(EmptyContext {}));

        let execution_steps = execution.collect_vec();
        let actual_states = &execution_steps
            .iter()
            .map(|e| e.state_name.as_ref().unwrap_or(&String::new()).clone())
            .collect_vec();
        let expected_states = &execution_expected_input.states;
        assert_eq!(
            expected_states, actual_states,
            "States are different for test case {i}."
        );
        let expected_status = &execution_expected_input.final_status;
        let actual_status = &execution_steps.last().unwrap().status;
        assert_eq!(
            expected_status, actual_status,
            "Status are different for test case {i}."
        );
    }

    Ok(())
}
