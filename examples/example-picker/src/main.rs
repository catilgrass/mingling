//! `Mingling` Example - Picker
//!
//! ## Step1 - Enable Feature
//! Enable the `parser` feature for mingling in `Cargo.toml`
//! ```toml
//! [dependencies]
//! mingling = { version = "...", features = ["parser"] }
//! ```
//!
//! ## Step2 - Write Code
//! Write the following content into `main.rs`
//!
//! ## Step3 - Build and Run
//! ```bash
//! cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick Bob
//! cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick Bob --age -15
//! cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick --age 99
//! ```

use mingling::{
    macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
    parser::Picker,
};

dispatcher!("pick", PickCommand => PickEntry);

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(PickCommand);
    program.exec();
}

pack!(NoNameProvided = ());
pack!(ParsedPickInput = (i32, String));

#[chain]
fn parse(prev: PickEntry) -> NextProcess {
    // Extract arguments from `PickEntry`'s inner and create a `Picker`
    let picker = Picker::new(prev.inner);
    let picked = picker
        // First extract the named argument
        .pick_or("--age", 20)
        .after(|n: i32| n.clamp(0, 100))
        // Then sequentially extract the remaining arguments
        .pick_or_route((), NoNameProvided::default().to_render())
        .unpack();

    match picked {
        Ok(value) => ParsedPickInput::new(value).to_render(),
        Err(e) => e,
    }
}

#[renderer]
fn render_parsed_pick_input(prev: ParsedPickInput) {
    let (age, name) = prev.inner;
    r_println!("Picked: name = {}, age = {}", name, age);
}

#[renderer]
fn render_no_name_input(_prev: NoNameProvided) {
    r_println!("No name provided.");
}

gen_program!();
