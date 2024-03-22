use crate::asl::error_handling::StateMachineExecutionPredefinedErrors;
use crate::asl::execution::{Execution, StateExecutionHandler, StateMachineExecutionError};
use crate::asl::states::all_states::EndOrNext;
use crate::asl::states::choice::{Choice, ChoiceEvaluationError};
use crate::asl::types::ExecutionInput;
use crate::asl::HandlerOutput;
use thiserror::Error;
use crate::asl::itertools_utils::try_find;

#[derive(Error, Debug)]
pub(crate) enum ChoiceStateExecutionError {
    #[error("No choice matched")]
    NoChoiceMatched,

    #[error("Malformed choice due to: {0}")]
    MalformedChoiceRule(#[source] ChoiceEvaluationError),
}

impl From<ChoiceStateExecutionError> for StateMachineExecutionError {
    fn from(value: ChoiceStateExecutionError) -> Self {
        StateMachineExecutionError {
            error: StateMachineExecutionPredefinedErrors::StatesNoChoiceMatched,
            cause: Some(value.to_string()),
        }
    }
}

impl<'a, H: StateExecutionHandler> Execution<'a, H> {
    pub fn handle_choice(
        &self,
        choice: &Choice,
        input: &ExecutionInput,
    ) -> Result<HandlerOutput, ChoiceStateExecutionError> {
        // let matched_choice = choice.choices.iter().find(|choice| {
        //         choice
        //             .evaluate(input)
        //             .map_err(|err| ChoiceStateExecutionError::MalformedChoiceRule(err))?
        //     })
        // });

        // let it = convert(choice.choices.iter());

        // let matched_choice = try_find(choice.choices, |choice.evaluate(input|)?
        let matched_choice = try_find(choice.choices.iter(), |choice| {
            choice
                .evaluate(input)
                .map_err(|err| ChoiceStateExecutionError::MalformedChoiceRule(err))
        })?;

        // let matched_choice = choice.choices.iter().try_find2(|choice| choice.evaluate(input)).map_err(|err| ChoiceStateExecutionError::MalformedChoiceRule(err))?;

        // let iter = choice.choices.iter().find_map(|choice| {
        //     let ret = choice
        //         .evaluate(input)
        //         .map_err(|err| ChoiceStateExecutionError::MalformedChoiceRule(err))
        // });
        // let matched_choice = convert(iter).find(|c| *c)?;
        // let matched_choice = None;
        match matched_choice {
            None => {
                // No choice matched the rule, so use the Default field if it was specified
                match &choice.default {
                    None => Err(ChoiceStateExecutionError::NoChoiceMatched),
                    Some(default_state_name) => {
                        // Contains a `default` field
                        Ok(HandlerOutput {
                            output: None,
                            next_state: EndOrNext::Next(default_state_name.clone()),
                        })
                    }
                }
            }
            Some(choice) => Ok(HandlerOutput {
                output: None,
                next_state: EndOrNext::Next(choice.next.clone()),
            }),
        }
    }
}
