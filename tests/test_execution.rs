use serde_json::value::Value;

use asl::asl::execution::ExecutionStatus;
use asl::asl::execution::{StateExecutionHandler, StateExecutionOutput, TaskExecutionError};
use asl::asl::state_machine::StateMachine;

struct TestStateExecutionHandler {}

impl StateExecutionHandler for TestStateExecutionHandler {
    fn execute_task(
        &self,
        resource: &str,
        input: &Value,
    ) -> std::result::Result<Option<Value>, TaskExecutionError> {
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

use anyhow::Result;
use itertools::Itertools;
use rstest::*;

#[rstest]
fn execute_hello_world() -> Result<()> {
    let definition = include_str!("test-data/hello-world.json");
    let state_machine = StateMachine::parse(definition)?;

    let input = serde_json::from_str(
        r#"
            "Hello world"
        "#,
    )?;
    let mut execution = state_machine.start(&input, TestStateExecutionHandler {});

    let val = Value::from("Hello world");

    let state_output = execution.next();
    assert_eq!(
        Some(StateExecutionOutput {
            status: ExecutionStatus::FinishedWithSuccess,
            state_name: Some("Hello World".to_string()),
            result: Some(val.clone())
        }),
        state_output,
    );
    assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status);

    let state_output = execution.next();
    assert_eq!(state_output, None);
    assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status);
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

    let mut execution = state_machine.start(&input, TestStateExecutionHandler {});
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
            status: ExecutionStatus::FinishedWithSuccess,
            state_name: Some("Succeed State".to_string()),
            result: Some(val.clone())
        })
    );
    assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status);

    // Iterator is exhausted
    assert_eq!(None, execution.next());
    assert_eq!(ExecutionStatus::FinishedWithSuccess, execution.status);
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
    let mut execution = state_machine.start(&val, TestStateExecutionHandler {});
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
    let expected_status = ExecutionStatus::with_error_and_cause("ErrorA", "Kaiju attack");
    assert_eq!(
        state_output,
        Some(StateExecutionOutput {
            status: expected_status.clone(),
            state_name: Some("Fail State".to_string()),
            result: Some(val.clone())
        })
    );
    assert_eq!(expected_status, execution.status);

    // Iterator is exhausted
    assert_eq!(None, execution.next());
    assert_eq!(expected_status, execution.status);
    Ok(())
}

#[rstest]
#[case(r#"{"foo": 1}"#, vec!["FirstState", "ChoiceState", "FirstMatchState", "NextState"], ExecutionStatus::FinishedWithSuccess)]
#[case(r#"{"foo": 2}"#, vec!["FirstState", "ChoiceState", "SecondMatchState", "NextState"], ExecutionStatus::FinishedWithSuccess)]
#[case(r#"{"foo": 3}"#, vec!["FirstState", "ChoiceState", "SecondMatchState", "NextState"], ExecutionStatus::FinishedWithSuccess)]
#[case(r#"{"foo": 4}"#, vec!["FirstState", "ChoiceState", "DefaultState"], ExecutionStatus::with_error_and_cause("DefaultStateError", "No Matches!"))]
fn execute_choice(
    #[case] input: &str,
    #[case] expected_states: Vec<&str>,
    #[case] final_execution_status: ExecutionStatus,
) -> Result<()> {
    let definition = include_str!("test-data/asl-validator/valid-choice-state.json");
    let state_machine = StateMachine::parse(definition)?;

    let input = serde_json::from_str(input)?;
    let execution = state_machine.start(&input, TestStateExecutionHandler {});

    let execution_steps = execution.collect_vec();
    let actual_states = execution_steps
        .iter()
        .map(|e| e.state_name.as_ref().unwrap_or(&String::new()).clone())
        .collect_vec();
    assert_eq!(expected_states, actual_states);
    assert_eq!(
        final_execution_status,
        execution_steps.last().unwrap().status
    );

    Ok(())
}

//
// #[rstest]
// fn execute_catch_failure() -> Result<()> {
//     let definition = include_str!("tests-data/hello-world.json");
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
// }
