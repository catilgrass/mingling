//! `Mingling` Example - General Renderer
//!
//! ## Step1 - Enable Feature
//! Enable the `general_renderer` feature for mingling in `Cargo.toml`
//! ```toml
//! [dependencies]
//! mingling = { version = "...", features = ["general_renderer", "parser"] }
//! ```
//!
//! ## Step2 - Add Dependencies
//! Add `serde` dependency to `Cargo.toml` for serialization support
//! ```toml
//! [dependencies]
//! serde = { version = "1", features = ["derive"] }
//! ```
//!
//! ## Step3 - Write Code
//! Write the following content into `main.rs`
//!
//! ## Step4 - Build and Run
//! ```bash
//! cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22
//! cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22 --json
//! cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22 --yaml
//! ```
//!
//! Will print:
//! ```plain
//! Bob is 22 years old
//! {"member_name":"Bob","member_age":22}
//! member_name: Bob
//! member_age: 22
//! ```

use mingling::prelude::*;
use mingling::{parser::Picker, setup::GeneralRendererSetup, Groupped};
use serde::Serialize;

dispatcher!("render", RenderCommand => RenderCommandEntry);

fn main() {
    let mut program = ThisProgram::new();
    // Add `GeneralRendererSetup` to receive user input `--json` `--yaml` parameters
    program.with_setup(GeneralRendererSetup);
    program.with_dispatcher(RenderCommand);
    program.exec();
}

// Manually implement Info struct
#[derive(Serialize, Groupped)]
struct Info {
    #[serde(rename = "member_name")]
    name: String,
    #[serde(rename = "member_age")]
    age: i32,
}

#[chain]
fn parse_render(prev: RenderCommandEntry) -> Next {
    let (name, age) = Picker::new(prev.inner)
        .pick::<String>(())
        .pick::<i32>(())
        .unpack();
    Info { name, age }.to_render()
}

// Implement default renderer for when general_renderer is not specified
#[renderer]
fn render_info(prev: Info) {
    r_println!("{} is {} years old", prev.name, prev.age);
}

gen_program!();
