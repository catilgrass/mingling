//! `Mingling` Example - Completion
//!
//! # How to Deploy
//! 1. Enable the `comp` feature
//! ```toml
//! [dependencies]
//! mingling = { version = "...", features = [
//!     "comp",  // Enable this feature
//!     "parser"
//! ] }
//! ```
//!
//! 2. Add `mingling` as a build dependency, enabling the `builds` and `comp` features
//! ```toml
//! [build-dependencies]
//! mingling = { version = "...", features = [
//!     "builds", // Enable this feature for build scripts
//!     "comp"
//! ] }
//! ```
//!
//! 3. Write `build.rs` to generate completion scripts at compile time
//! ```ignore
//! use mingling::build::{build_comp_scripts, build_comp_scripts_with_bin_name};
//! fn main() {
//!     // Generate completion scripts for the current program, using the Cargo package name as the binary filename
//!     build_comp_scripts(env!("CARGO_PKG_NAME")).unwrap();
//!
//!     // Or, explicitly specify the binary filename
//!     // build_comp_scripts("your_bin").unwrap();
//! }
//! ```
//!
//! 4. Write `main.rs`, adding completion logic for your command entry point
//! 5. Execute `cargo install --path ./`, then run the corresponding completion script in your shell

use mingling::{
    EnumTag, Groupped, ShellContext, Suggest,
    macros::{
        chain, completion, dispatcher, gen_program, r_println, renderer, suggest, suggest_enum,
    },
    parser::{PickableEnum, Picker},
};

// Define dispatcher `FruitCommand`, directing subcommand "fruit" to `FruitEntry`
dispatcher!("fruit", FruitCommand => FruitEntry);

#[completion(FruitEntry)]
fn comp_fruit_command(ctx: &ShellContext) -> Suggest {
    if ctx.filling_argument_first("--name") {
        return suggest!();
    }
    if ctx.filling_argument_first("--type") {
        return suggest_enum!(FruitType);
    }
    if ctx.typing_argument() {
        return suggest! {
            "--name": "Fruit name",
            "--type": "Fruit type"
        }
        .strip_typed_argument(ctx);
    }
    return suggest!();
}

fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(CompletionDispatcher);
    program.with_dispatcher(FruitCommand);
    program.exec();
}

#[derive(Groupped)]
struct FruitInfo {
    name: String,
    fruit_type: FruitType,
}

#[derive(Default, Debug, EnumTag)]
enum FruitType {
    #[enum_desc("It's Apple")]
    #[enum_rename("apple")]
    FruitApple,

    #[enum_desc("It's Banana")]
    #[enum_rename("banana")]
    FruitBanana,

    #[enum_desc("It's Cherry")]
    #[enum_rename("cherry")]
    FruitCherry,

    #[enum_desc("It's Date")]
    #[enum_rename("date")]
    FruitDate,

    #[enum_desc("It's Elderberry")]
    #[enum_rename("elderberry")]
    FruitElderberry,

    #[default]
    #[enum_rename("unknown")]
    Unknown,
}

impl PickableEnum for FruitType {}

#[chain]
fn parse_fruit_info(prev: FruitEntry) -> NextProcess {
    let picker = Picker::from(prev.inner);
    let (fruit_name, fruit_type) = picker.pick("--name").pick("--type").unpack();
    let info = FruitInfo {
        name: fruit_name,
        fruit_type,
    };
    info.to_render()
}

#[renderer]
fn render_fruit(prev: FruitInfo) {
    match (prev.name.is_empty(), prev.fruit_type) {
        (true, FruitType::Unknown) => {
            r_println!("Fruit name is empty and type is unknown");
        }
        (true, fruit_type) => {
            r_println!("Fruit name is empty, Type: {:?}", fruit_type);
        }
        (false, FruitType::Unknown) => {
            r_println!("Fruit name: {}, Type is unknown", prev.name);
        }
        (false, fruit_type) => {
            r_println!("Fruit name: {}, Type: {:?}", prev.name, fruit_type);
        }
    }
}

gen_program!();
