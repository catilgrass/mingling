//! `Mingling` Example - Async
//!
//! After enabling the `async` feature:
//! 1. The `chain!` macro will support using **async** functions,
//! 2. The `exec` function of `Program` will return a `Future` for you to use with an async runtime
//!
//! ## Enable Feature
//! Enable the `async` feature for mingling in `Cargo.toml`
//! ```toml
//! [dependencies]
//! mingling = { version = "...", features = ["async"] }
//! ```
//!
//! # How to Run
//! ```bash
//! cargo run --manifest-path ./examples/example-async/Cargo.toml -- hello World
//! ```

use mingling::prelude::*;

dispatcher!("hello", HelloCommand => HelloEntry);

// Use Tokio async runtime
#[tokio::main]
async fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(HelloCommand);

    // Run program
    program.exec().await;
}

pack!(Hello = String);

// You can freely use async / non-async functions to declare your Chain

#[chain]
// fn parse_name(prev: HelloEntry) -> NextProcess {
async fn parse_name(prev: HelloEntry) -> NextProcess {
    let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
    Hello::new(name).to_render()
}

// For renderers, you can still only use synchronous functions
#[renderer]
fn render_hello_who(prev: Hello) {
    r_println!("Hello, {}!", *prev);
}

gen_program!();
