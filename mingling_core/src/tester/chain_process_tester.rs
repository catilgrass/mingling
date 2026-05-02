use std::fmt::Display;

use crate::{ChainProcess, Next, ProgramCollect};

/// Asserts that the chain process result has the expected output type and next state.
///
/// This function is used to verify that a `ChainProcess` result meets expectations. It optionally checks:
/// - `member_id`: whether the output type identifier matches the expected one.
/// - `next`: whether the next processing step state matches the expected one.
///
/// If the result is not `ChainProcess::Ok` or any checked field does not match, the function will panic with a detailed error message.
///
/// # Parameters
///
/// * `result` - A reference to the chain process result to assert.
/// * `next` - The expected next state (optional). If `None`, this check is skipped.
/// * `member_id` - The expected output type identifier (optional). If `None`, this check is skipped.
///
/// # Generic Constraints
///
/// * `C` - Must implement `ProgramCollect`, `Display`, `PartialEq` and have a `'static` lifetime.
///
/// # Panics
///
/// Panics in the following cases:
/// - The result is `ChainProcess::Err`, outputting the error message.
/// - When `member_id` is not `None` and does not equal the actual output's `member_id`, displaying the expected and actual values.
/// - When `next` is not `None` and does not equal the actual next state, displaying the expected and actual values.
pub fn assert_next_eq<C>(result: &ChainProcess<C>, next: Option<Next>, member_id: Option<C>)
where
    C: ProgramCollect + Display + PartialEq + 'static,
{
    match result {
        ChainProcess::Ok(any) => {
            if let Some(member_id) = member_id {
                if member_id != any.0.member_id {
                    panic!(
                        "Unexpected result type: expected {}, found {}",
                        member_id, any.0.member_id
                    );
                }
            }
            if let Some(next) = next {
                if next != any.1 {
                    panic!("Unexpected next state: expected {}, found {}", next, any.1);
                }
            }
        }
        ChainProcess::Err(chain_process_error) => {
            panic!("Chain process error: {}", chain_process_error);
        }
    }
}

/// Asserts that a chain process result has the expected output type and next state.
#[macro_export]
macro_rules! assert_next {
    ($result:expr, $expected_next:path, $expected_type:path) => {
        ::mingling::test::assert_next_eq(&$result, Some($expected_next), Some($expected_type))
    };
    ($result:expr, $expected_next:path) => {
        ::mingling::test::assert_next_eq(&$result, Some($expected_next), None)
    };
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Asserts that a chain process result is Ok and has the expected output type.
#[macro_export]
macro_rules! assert_chain_result {
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Alias for assert_chain_result.
#[macro_export]
macro_rules! assert_render_result {
    ($result:expr) => {
        ::mingling::test::assert_next_eq(&$result, None, None)
    };
}

/// Asserts that the result's output type matches the expected member_id.
#[macro_export]
macro_rules! assert_member_id {
    ($result:expr, $expected_type:path) => {
        ::mingling::test::assert_next_eq(&$result, None, Some($expected_type))
    };
}
