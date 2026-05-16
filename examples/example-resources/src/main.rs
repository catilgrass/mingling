//! `Mingling` Example - Global Resource Injection
//!
//! This example demonstrates how to use global resource injection in `#[chain]` functions.
//! You can inject both immutable (`&T`) and mutable (`&mut T`) references to global resources.
//!
//! # How to Run
//! ```bash
//! cargo run --manifest-path ./examples/example-resources/Cargo.toml -- setup
//! ```

use mingling::prelude::*;
use std::{env::current_dir, path::PathBuf};

// Define a resource for storing global state
#[derive(Default, Clone)]
pub struct MyResource {
    current_dir: PathBuf,
}

fn main() {
    let mut program = ThisProgram::new();

    // Add the resource to the program
    program.with_resource(MyResource::default());

    program.with_dispatcher(SetupCommand);
    program.exec_and_exit();
}

dispatcher!("setup", SetupCommand => SetupEntry);
pack!(StateRead = ());
pack!(ResultCurrentDir = PathBuf);

#[chain]
fn setup(
    _prev: SetupEntry,
    resource: &mut MyResource, // Import the resource into `setup`
) -> NextProcess {
    // Set the global resource
    resource.current_dir = current_dir().unwrap();

    StateRead::default()
}

#[chain]
fn read(_prev: StateRead, resource: &MyResource) -> NextProcess {
    // Read the global resource
    let current_dir = resource.current_dir.clone();
    ResultCurrentDir::new(current_dir).to_render()
}

#[renderer]
fn render_current_dir(dir: ResultCurrentDir) {
    r_println!("Current dir: {}", dir.to_string_lossy())
}

gen_program!();
