//! `Mingling` Example - Exit Code
//!
//! This example demonstrates how to modify the program's exit code using `ExitCodeSetup`.
//! By default, the program exits with code 0. This example shows:
//! 1. Using `dispatcher!` to define an error command,
//! 2. Using `chain!` to handle errors and set a custom exit code via `ProgramExitCode`,
//! 3. Using `renderer!` to print an error message.
//!
//! # How to Run
//! ```bash
//! cargo run --manifest-path ./examples/example-exit-code/Cargo.toml -- error
//! ```

use mingling::{
    macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
    res::ExitCode,
    setup::ExitCodeSetup,
    this,
};

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(ErrorCommand);
    program.with_setup(ExitCodeSetup::<ThisProgram>::default());
    program.exec();
}

dispatcher!("error", ErrorCommand => ErrorEntry);
pack!(ResultError = ());

#[chain]
fn handle_error_entry(_prev: ErrorEntry) -> NextProcess {
    this::<ThisProgram>().modify_res(|r: &mut ExitCode| r.exit_code = 1);
    return ResultError::default();
}

#[renderer]
fn render_error(_prev: ResultError) {
    r_println!("Error!");
}

gen_program!();
