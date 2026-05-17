//! `Mingling` Example - Basic
//!
//! # How to Run
//! ```bash
//! cargo run --manifest-path ./examples/example-basic/Cargo.toml -- hello World
//! ```

use mingling::prelude::*;

// Define dispatcher `HelloCommand`, directing subcommand "hello" to `HelloEntry`
dispatcher!("hello", HelloCommand => HelloEntry);

fn main() {
    // Create program
    let mut program = ThisProgram::new();

    // Add dispatcher `HelloCommand`
    program.with_dispatcher(HelloCommand);

    // Run program
    program.exec();
}

// Register wrapper type `Hello`, setting inner to `String`
pack!(Hello = String);

// Register chain to `ThisProgram`, handling logic from `HelloEntry`
#[chain]
fn parse_name(prev: HelloEntry) -> Next {
    // Extract string from `HelloEntry` as argument
    let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());

    // Build `Hello` type and route to renderer
    Hello::new(name).to_render()
}

// Register renderer to `ThisProgram`, handling rendering of `Hello`
#[renderer]
fn render_hello_who(prev: Hello) {
    // Print message
    r_println!("Hello, {}!", *prev);

    // Program ends here
}

// Generate program, default is `ThisProgram`
gen_program!();
