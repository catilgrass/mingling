// Auto generated

/// `Mingling` Example - Async
///
///  After enabling the `async` feature:
///  1. The `chain!` macro will support using **async** functions,
///  2. The `exec` function of `Program` will return a `Future` for you to use with an async runtime
///
///  ## Enable Feature
///  Enable the `async` feature for mingling in `Cargo.toml`
///  ```toml
///  [dependencies]
///  mingling = { version = "...", features = ["async"] }
///  ```
///
///  # How to Run
///  ```bash
///  cargo run --manifest-path ./examples/example-async/Cargo.toml -- hello World
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-async"
/// version = "0.0.1"
/// edition = "2024"
///
/// [dependencies]
/// tokio = { version = "1", features = ["full"] }
/// mingling = { path = "../../mingling", features = ["async"] }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::macros::{chain, dispatcher, gen_program, pack, r_println, renderer};
///
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// // Use Tokio async runtime
/// #[tokio::main]
/// async fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(HelloCommand);
///
///     // Run program
///     program.exec().await;
/// }
///
/// pack!(Hello = String);
///
/// // You can freely use async / non-async functions to declare your Chain
///
/// #[chain]
/// // fn parse_name(prev: HelloEntry) -> NextProcess {
/// async fn parse_name(prev: HelloEntry) -> NextProcess {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     Hello::new(name).to_render()
/// }
///
/// // For renderers, you can still only use synchronous functions
/// #[renderer]
/// fn render_hello_who(prev: Hello) {
///     r_println!("Hello, {}!", *prev);
/// }
///
/// gen_program!();
/// ```
pub mod example_async {}
/// `Mingling` Example - Basic
///
///  # How to Run
///  ```bash
///  cargo run --manifest-path ./examples/example-basic/Cargo.toml -- hello World
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-basic"
/// version = "0.0.1"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::macros::{chain, dispatcher, gen_program, pack, r_println, renderer};
///
/// // Define dispatcher `HelloCommand`, directing subcommand "hello" to `HelloEntry`
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// fn main() {
///     // Create program
///     let mut program = ThisProgram::new();
///
///     // Add dispatcher `HelloCommand`
///     program.with_dispatcher(HelloCommand);
///
///     // Run program
///     program.exec();
/// }
///
/// // Register wrapper type `Hello`, setting inner to `String`
/// pack!(Hello = String);
///
/// // Register chain to `ThisProgram`, handling logic from `HelloEntry`
/// #[chain]
/// fn parse_name(prev: HelloEntry) -> NextProcess {
///     // Extract string from `HelloEntry` as argument
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///
///     // Build `Hello` type and route to renderer
///     Hello::new(name).to_render()
/// }
///
/// // Register renderer to `ThisProgram`, handling rendering of `Hello`
/// #[renderer]
/// fn render_hello_who(prev: Hello) {
///     // Print message
///     r_println!("Hello, {}!", *prev);
///
///     // Program ends here
/// }
///
/// // Generate program, default is `ThisProgram`
/// gen_program!();
/// ```
pub mod example_basic {}
/// `Mingling` Example - Completion
///
///  # How to Deploy
///  1. Enable the `comp` feature
///  ```toml
///  [dependencies]
///  mingling = { version = "...", features = [
///      "comp",  // Enable this feature
///      "parser"
///  ] }
///  ```
///
///  2. Add `mingling` as a build dependency, enabling the `builds` and `comp` features
///  ```toml
///  [build-dependencies]
///  mingling = { version = "...", features = [
///      "builds", // Enable this feature for build scripts
///      "comp"
///  ] }
///  ```
///
///  3. Write `build.rs` to generate completion scripts at compile time
///  ```ignore
///  use mingling::build::{build_comp_scripts, build_comp_scripts_with_bin_name};
///  fn main() {
///      // Generate completion scripts for the current program, using the Cargo package name as the binary filename
///      build_comp_scripts(env!("CARGO_PKG_NAME")).unwrap();
///
///      // Or, explicitly specify the binary filename
///      // build_comp_scripts("your_bin").unwrap();
///  }
///  ```
///
///  4. Write `main.rs`, adding completion logic for your command entry point
///  5. Execute `cargo install --path ./`, then run the corresponding completion script in your shell
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-completion"
/// version = "0.0.1"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = ["comp", "parser"] }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::{
///     EnumTag, Groupped, ShellContext, Suggest,
///     macros::{
///         chain, completion, dispatcher, gen_program, r_println, renderer, suggest, suggest_enum,
///     },
///     parser::{PickableEnum, Picker},
/// };
///
/// // Define dispatcher `FruitCommand`, directing subcommand "fruit" to `FruitEntry`
/// dispatcher!("fruit", FruitCommand => FruitEntry);
///
/// #[completion(FruitEntry)]
/// fn comp_fruit_command(ctx: &ShellContext) -> Suggest {
///     if ctx.filling_argument_first("--name") {
///         return suggest!();
///     }
///     if ctx.filling_argument_first("--type") {
///         return suggest_enum!(FruitType);
///     }
///     if ctx.typing_argument() {
///         return suggest! {
///             "--name": "Fruit name",
///             "--type": "Fruit type"
///         }
///         .strip_typed_argument(ctx);
///     }
///     return suggest!();
/// }
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(CompletionDispatcher);
///     program.with_dispatcher(FruitCommand);
///     program.exec();
/// }
///
/// #[derive(Groupped)]
/// struct FruitInfo {
///     name: String,
///     fruit_type: FruitType,
/// }
///
/// #[derive(Default, Debug, EnumTag)]
/// enum FruitType {
///     #[enum_desc("It's Apple")]
///     #[enum_rename("apple")]
///     FruitApple,
///
///     #[enum_desc("It's Banana")]
///     #[enum_rename("banana")]
///     FruitBanana,
///
///     #[enum_desc("It's Cherry")]
///     #[enum_rename("cherry")]
///     FruitCherry,
///
///     #[enum_desc("It's Date")]
///     #[enum_rename("date")]
///     FruitDate,
///
///     #[enum_desc("It's Elderberry")]
///     #[enum_rename("elderberry")]
///     FruitElderberry,
///
///     #[default]
///     #[enum_rename("unknown")]
///     Unknown,
/// }
///
/// impl PickableEnum for FruitType {}
///
/// #[chain]
/// fn parse_fruit_info(prev: FruitEntry) -> NextProcess {
///     let picker = Picker::from(prev.inner);
///     let (fruit_name, fruit_type) = picker.pick("--name").pick("--type").unpack();
///     let info = FruitInfo {
///         name: fruit_name,
///         fruit_type,
///     };
///     info.to_render()
/// }
///
/// #[renderer]
/// fn render_fruit(prev: FruitInfo) {
///     match (prev.name.is_empty(), prev.fruit_type) {
///         (true, FruitType::Unknown) => {
///             r_println!("Fruit name is empty and type is unknown");
///         }
///         (true, fruit_type) => {
///             r_println!("Fruit name is empty, Type: {:?}", fruit_type);
///         }
///         (false, FruitType::Unknown) => {
///             r_println!("Fruit name: {}, Type is unknown", prev.name);
///         }
///         (false, fruit_type) => {
///             r_println!("Fruit name: {}, Type: {:?}", prev.name, fruit_type);
///         }
///     }
/// }
///
/// gen_program!();
/// ```
pub mod example_completion {}
/// `Mingling` Example - Dispatch Tree
///
///  # How to Deploy
///  1. Enable the `dispatch_tree` feature (`comp` is optional)
///  ```toml
///  mingling = { version = "...", features = [
///      "dispatch_tree",  // Enable this feature
///      "comp" // optional
///  ] }
///  ```
///
///  2. Using `cargo expand`:
///
///  ```bash
///  cargo expand --manifest-path examples/example-dispatch-tree/Cargo.toml > expanded.rs
///  cat expanded.rs | grep dispatch_args_trie -A 264
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-dispatch-tree"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = ["dispatch_tree", "comp"] }
/// ```
///
/// main.rs
/// ```ignore
/// #![allow(unused_mut)]
///
/// use mingling::macros::{dispatcher, gen_program};
///
/// fn main() {
///     let mut program = ThisProgram::new();
///
///     // After enabling `dispatch_tree`, this method will no longer exist
///     // program.with_dispatcher(CommandGreet);
///     //
///     // The `CompletionDispatcher` automatically generated by `comp` will also be imported
///     // automatically
///     // program.with_dispatcher(CompletionDispatcher);
///
///     program.exec();
/// }
///
/// dispatcher!("greet", CommandGreet => EntryGreet);
/// dispatcher!("help", CommandHelp => EntryHelp);
/// dispatcher!("quit", CommandQuit => EntryQuit);
/// dispatcher!("list", CommandList => EntryList);
/// dispatcher!("status", CommandStatus => EntryStatus);
/// dispatcher!("save", CommandSave => EntrySave);
/// dispatcher!("load", CommandLoad => EntryLoad);
/// dispatcher!("config", CommandConfig => EntryConfig);
/// dispatcher!("run", CommandRun => EntryRun);
/// dispatcher!("debug", CommandDebug => EntryDebug);
/// dispatcher!("version", CommandVersion => EntryVersion);
///
/// gen_program!();
/// ```
pub mod example_dispatch_tree {}
/// `Mingling` Example - Exit Code
///
///  This example demonstrates how to modify the program's exit code using `ExitCodeSetup`.
///  By default, the program exits with code 0. This example shows:
///  1. Using `dispatcher!` to define an error command,
///  2. Using `chain!` to handle errors and set a custom exit code via `ProgramExitCode`,
///  3. Using `renderer!` to print an error message.
///
///  # How to Run
///  ```bash
///  cargo run --manifest-path ./examples/example-exit-code/Cargo.toml -- error
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-exit-code"
/// version = "0.1.0"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling" }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::{
///     macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
///     res::{exit_code, update_exit_code},
///     setup::ExitCodeSetup,
/// };
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(ErrorCommand);
///     program.with_setup(ExitCodeSetup::<ThisProgram>::default());
///     program.exec_and_exit();
/// }
///
/// dispatcher!("error", ErrorCommand => ErrorEntry);
/// pack!(ResultError = ());
///
/// #[chain]
/// fn handle_error_entry(_prev: ErrorEntry) -> NextProcess {
///     update_exit_code::<ThisProgram>(1);
///     return ResultError::default();
/// }
///
/// #[renderer]
/// fn render_error(_prev: ResultError) {
///     let exit_code = exit_code::<ThisProgram>();
///     r_println!("Exit with exit code: {}", exit_code);
/// }
///
/// gen_program!();
/// ```
pub mod example_exit_code {}
/// `Mingling` Example - General Renderer
///
///  ## Step1 - Enable Feature
///  Enable the `general_renderer` feature for mingling in `Cargo.toml`
///  ```toml
///  [dependencies]
///  mingling = { version = "...", features = ["general_renderer", "parser"] }
///  ```
///
///  ## Step2 - Add Dependencies
///  Add `serde` dependency to `Cargo.toml` for serialization support
///  ```toml
///  [dependencies]
///  serde = { version = "1", features = ["derive"] }
///  ```
///
///  ## Step3 - Write Code
///  Write the following content into `main.rs`
///
///  ## Step4 - Build and Run
///  ```bash
///  cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22
///  cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22 --json
///  cargo run --manifest-path ./examples/example-general-renderer/Cargo.toml -- render Bob 22 --yaml
///  ```
///
///  Will print:
///  ```plain
///  Bob is 22 years old
///  {"member_name":"Bob","member_age":22}
///  member_name: Bob
///  member_age: 22
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-general-renderer"
/// version = "0.0.1"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = [
///     "parser",
///     "general_renderer",
/// ] }
/// serde = { version = "1", features = ["derive"] }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::{
///     Groupped,
///     macros::{chain, dispatcher, gen_program, r_println, renderer},
///     parser::Picker,
///     setup::GeneralRendererSetup,
/// };
/// use serde::Serialize;
///
/// dispatcher!("render", RenderCommand => RenderCommandEntry);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     // Add `GeneralRendererSetup` to receive user input `--json` `--yaml` parameters
///     program.with_setup(GeneralRendererSetup);
///     program.with_dispatcher(RenderCommand);
///     program.exec();
/// }
///
/// // Manually implement Info struct
/// #[derive(Serialize, Groupped)]
/// struct Info {
///     #[serde(rename = "member_name")]
///     name: String,
///     #[serde(rename = "member_age")]
///     age: i32,
/// }
///
/// #[chain]
/// fn parse_render(prev: RenderCommandEntry) -> NextProcess {
///     let (name, age) = Picker::new(prev.inner)
///         .pick::<String>(())
///         .pick::<i32>(())
///         .unpack();
///     Info { name, age }.to_render()
/// }
///
/// // Implement default renderer for when general_renderer is not specified
/// #[renderer]
/// fn render_info(prev: Info) {
///     r_println!("{} is {} years old", prev.name, prev.age);
/// }
///
/// gen_program!();
/// ```
pub mod example_general_renderer {}
/// `Mingling` Example - Picker
///
///  ## Step1 - Enable Feature
///  Enable the `parser` feature for mingling in `Cargo.toml`
///  ```toml
///  [dependencies]
///  mingling = { version = "...", features = ["parser"] }
///  ```
///
///  ## Step2 - Write Code
///  Write the following content into `main.rs`
///
///  ## Step3 - Build and Run
///  ```bash
///  cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick Bob
///  cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick Bob --age -15
///  cargo run --manifest-path ./examples/example-picker/Cargo.toml -- pick --age 99
///  ```
///
/// Cargo.toml
/// ```ignore
/// [package]
/// name = "example-picker"
/// version = "0.0.1"
/// edition = "2024"
///
/// [dependencies]
/// mingling = { path = "../../mingling", features = ["parser"] }
/// tokio = { version = "1", features = ["rt", "rt-multi-thread", "macros"] }
/// ```
///
/// main.rs
/// ```ignore
/// use mingling::{
///     macros::{chain, dispatcher, gen_program, pack, r_println, renderer},
///     parser::Picker,
/// };
///
/// dispatcher!("pick", PickCommand => PickEntry);
///
/// fn main() {
///     let mut program = ThisProgram::new();
///     program.with_dispatcher(PickCommand);
///     program.exec();
/// }
///
/// pack!(NoNameProvided = ());
/// pack!(ParsedPickInput = (i32, String));
///
/// #[chain]
/// fn parse(prev: PickEntry) -> NextProcess {
///     // Extract arguments from `PickEntry`'s inner and create a `Picker`
///     let picker = Picker::new(prev.inner);
///     let picked = picker
///         // First extract the named argument
///         .pick_or("--age", 20)
///         .after(|n: i32| n.clamp(0, 100))
///         // Then sequentially extract the remaining arguments
///         .pick_or_route((), NoNameProvided::default().to_render())
///         .unpack();
///
///     match picked {
///         Ok(value) => ParsedPickInput::new(value).to_render(),
///         Err(e) => e,
///     }
/// }
///
/// #[renderer]
/// fn render_parsed_pick_input(prev: ParsedPickInput) {
///     let (age, name) = prev.inner;
///     r_println!("Picked: name = {}, age = {}", name, age);
/// }
///
/// #[renderer]
/// fn render_no_name_input(_prev: NoNameProvided) {
///     r_println!("No name provided.");
/// }
///
/// gen_program!();
/// ```
pub mod example_picker {}
