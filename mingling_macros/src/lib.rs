//! Mingling Macros Crate
//!
//! This crate provides procedural macros for the Mingling framework.
//! Macros are implemented in separate modules and re-exported here.
//!
//! # Architecture Overview
//!
//! The Mingling macros crate provides the following categories of macros:
//!
//! - **Command definition**: `dispatcher!`, `dispatcher_clap!`, `node!`, `pack!`
//! - **Chain processing**: `#[chain]`, `gen_program!`, `route!`
//! - **Rendering**: `#[renderer]`, `r_print!`, `r_println!`
//! - **Help system**: `#[help]`, `register_help!`
//! - **Derive macros**: `#[derive(Groupped)]`, `#[derive(EnumTag)]`, `#[derive(GrouppedSerialize)]`
//! - **Program setup**: `#[program_setup]`
//! - **Completion (comp feature)**: `#[completion]`, `suggest!`, `suggest_enum!`
//! - **Internal registration**: `register_type!`, `register_chain!`, `register_renderer!`,
//!   `program_fallback_gen!`, `program_final_gen!`, `program_comp_gen!`

use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::collections::BTreeSet;
use std::sync::Mutex;
use syn::parse_macro_input;

mod chain;
#[cfg(feature = "comp")]
mod completion;
#[cfg(feature = "dispatch_tree")]
mod dispatch_tree_gen;
mod dispatcher;
#[cfg(feature = "clap")]
mod dispatcher_clap;
mod enum_tag;
mod groupped;
mod help;
mod node;
mod pack;
mod program_setup;
mod render;
mod renderer;
#[cfg(feature = "comp")]
mod suggest;

pub(crate) const DEFAULT_PROGRAM_NAME: &str = "ThisProgram";

#[allow(dead_code)]
pub(crate) fn default_program_ident() -> Ident {
    Ident::new(DEFAULT_PROGRAM_NAME, proc_macro2::Span::call_site())
}

pub(crate) fn default_program_path() -> proc_macro2::TokenStream {
    quote::quote! { crate::ThisProgram }
}

// Global variables
#[cfg(feature = "general_renderer")]
pub(crate) static GENERAL_RENDERERS: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));
#[cfg(feature = "comp")]
pub(crate) static COMPLETIONS: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));

#[cfg(feature = "dispatch_tree")]
pub(crate) static COMPILE_TIME_DISPATCHERS: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));

pub(crate) static PACKED_TYPES: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));
pub(crate) static CHAINS: Lazy<Mutex<BTreeSet<String>>> = Lazy::new(|| Mutex::new(BTreeSet::new()));
pub(crate) static RENDERERS: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));
pub(crate) static CHAINS_EXIST: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));
pub(crate) static RENDERERS_EXIST: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));
pub(crate) static HELP_REQUESTS: Lazy<Mutex<BTreeSet<String>>> =
    Lazy::new(|| Mutex::new(BTreeSet::new()));

/// Checks that a TypePath is a simple single-segment identifier (no `::` in the path).
///
/// This is used by `#[renderer]`, `#[help]`, `#[chain]`, and `#[completion]` attribute macros
/// to ensure that the type in the function signature is a bare identifier like `Empty`,
/// not a qualified path like `other::Empty`.
///
/// Returns `None` if the type is valid, or a `compile_error!` token stream if it contains `::`.
pub(crate) fn check_single_segment_type(
    type_path: &syn::TypePath,
    attr_name: &str,
) -> Option<proc_macro2::TokenStream> {
    if type_path.path.segments.len() > 1 {
        let type_str = quote! { #type_path };
        Some(quote! {
            compile_error!(concat!(
                "The type `",
                #type_str,
                "` in ",
                #attr_name,
                " function must be a simple single-segment type, ",
                "e.g. `Empty` instead of `other::Empty`. ",
                "Qualified paths with `::` are not allowed here."
            ));
        })
    } else {
        None
    }
}

/// Creates a `Node` from a dot-separated path string.
///
/// Each segment is converted to kebab-case (unless it starts with `_`).
/// Segments are joined via `.join()` calls, building a path hierarchy for
/// command matching.
///
/// # Syntax
///
/// ```rust,ignore
/// node!("subcommand")
/// node!("sub.subsub")
/// node!("")           // empty → Node::default()
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::node;
///
/// // Creates a single-level node for "hello"
/// let n = node!("hello");
///
/// // Creates a two-level node for "remote control"
/// let n = node!("remote.control");
/// ```
///
/// # Internals
///
/// The generated code is equivalent to:
/// ```rust,ignore
/// Node::default().join("hello")
/// Node::default().join("remote").join("control")
/// ```
///
/// This macro is typically used internally by `dispatcher!` and should rarely
/// need to be called directly.
#[proc_macro]
pub fn node(input: TokenStream) -> TokenStream {
    node::node(input)
}

/// Creates a type-safe wrapper struct around an inner type, with automatic
/// trait implementations for use in the Mingling chain/render pipeline.
///
/// The generated struct implements: `From`/`Into`, `AsRef`/`AsMut`, `Deref`/`DerefMut`,
/// `Default` (conditional on inner type), and conversion into `AnyOutput` /
/// `ChainProcess` for routing.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program name (uses `ThisProgram`):
/// pack!(TypeName = InnerType);
///
/// // Explicit program name:
/// pack!(MyProgram, TypeName = InnerType);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::pack;
///
/// // Creates `Hello` wrapping `String`, registered under `ThisProgram`:
/// pack!(Hello = String);
///
/// // Creates `Greeting` wrapping `String`, registered under `MyApp`:
/// pack!(MyApp, Greeting = String);
/// ```
///
/// After expansion, `Hello` has:
/// - `Hello::new(String)` — constructor
/// - `Hello::to_chain()` — routes to the next chain processor
/// - `Hello::to_render()` — routes to a renderer
/// - `From<String> for Hello`, `From<Hello> for String`
/// - `Deref<Target = String>`, `DerefMut`
/// - `AsRef<String>`, `AsMut<String>`
/// - `Default` if `String: Default`
/// - `Into<AnyOutput<ThisProgram>>`, `Into<ChainProcess<ThisProgram>>`
/// - Implements `Groupped<ThisProgram>` with `member_id()` returning the enum variant
///
/// The struct is also registered via `register_type!` so that `gen_program!`
/// can include it in the program enum.
///
/// When the `general_renderer` feature is enabled, the struct also gets
/// `#[derive(serde::Serialize)]`.
#[proc_macro]
pub fn pack(input: TokenStream) -> TokenStream {
    pack::pack(input)
}

/// Early-returns an error from a `Result`, converting the `Ok` branch to a
/// `ChainProcess`.
///
/// This macro is equivalent to:
/// ```rust,ignore
/// match expr {
///     Ok(r) => r,
///     Err(e) => return e,
/// }
/// ```
///
/// It is useful inside chain functions where you have a `Result<ChainProcess<G>, ChainProcess<G>>`
/// and want to propagate the error case as an early return.
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{chain, route};
///
/// #[chain]
/// fn process(prev: SomeEntry) -> ChainProcess<ThisProgram> {
///     let value = route!(try_something().ok_or(ErrorEntry::new("failed".into()).to_render()));
///     // value is the Ok(ChainProcess) from try_something()
///     value
/// }
/// ```
#[proc_macro]
pub fn route(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as syn::Expr);
    let expanded = quote! {
        match #expr {
            Ok(r) => r,
            Err(e) => return e,
        }
    };
    TokenStream::from(expanded)
}

/// Creates a `Dispatcher` implementation for a subcommand.
///
/// This is the primary way to define command-line subcommands in Mingling.
/// It generates a dispatcher struct that, when matched against user input,
/// converts the arguments into a `ChainProcess` via the specified entry type.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program name (uses `ThisProgram`):
/// dispatcher!("command.path", CommandStruct => EntryStruct);
///
/// // Explicit program name:
/// dispatcher!(MyProgram, "command.path", CommandStruct => EntryStruct);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::dispatcher;
///
/// // "hello" subcommand → HelloCommand → HelloEntry
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// // Nested: "remote control" → RemoteControlCommand → RemoteControlEntry
/// dispatcher!("remote.control", RemoteControlCommand => RemoteControlEntry);
///
/// // With explicit program:
/// dispatcher!(MyApp, "status", StatusCommand => StatusEntry);
/// ```
///
/// The generated `HelloCommand` implements `Dispatcher<ThisProgram>`:
/// - `node()` returns the `Node` hierarchy for "hello"
/// - `begin(args)` wraps `args` into `HelloEntry` and routes to chain
/// - `clone_dispatcher()` returns a boxed clone
///
/// The `HelloEntry` struct is a wrapper around `Vec<String>` created via
/// an implicit `pack!` call with the program name.
///
/// When the `comp` feature is enabled, the entry type also implements
/// `CompletionEntry` for providing shell completion suggestions.
#[proc_macro]
pub fn dispatcher(input: TokenStream) -> TokenStream {
    dispatcher::dispatcher(input)
}

/// Prints formatted text to the current `RenderResult` buffer within a
/// `#[renderer]`(macro.renderer.html) function.
///
/// This macro requires a mutable reference to a `RenderResult` named `r`
/// to be in scope, which is automatically provided inside `#[renderer]`
/// functions.
///
/// # Syntax
///
/// Same as `format!` / `print!`:
///
/// ```rust,ignore
/// r_print!("Hello, {}!", name);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{renderer, r_print};
///
/// #[renderer]
/// fn show_greeting(prev: Greeting) {
///     r_print!("Hello, {}!", *prev);
/// }
/// ```
///
/// # Difference from `r_println!`
///
/// `r_print!` does **not** append a newline. Use `r_println!` for newline-terminated output.
#[proc_macro]
pub fn r_print(input: TokenStream) -> TokenStream {
    render::r_print(input)
}

/// Prints formatted text followed by a newline to the current `RenderResult`
/// buffer within a `#[renderer]`(macro.renderer.html) function.
///
/// This macro requires a mutable reference to a `RenderResult` named `r`
/// to be in scope, which is automatically provided inside `#[renderer]`
/// functions.
///
/// # Syntax
///
/// Same as `println!`:
///
/// ```rust,ignore
/// r_println!("Hello, {}!", name);
/// r_println!();  // just a newline
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{renderer, r_println};
///
/// #[renderer]
/// fn show_greeting(prev: Greeting) {
///     r_println!("Hello, {}!", *prev);
/// }
/// ```
#[proc_macro]
pub fn r_println(input: TokenStream) -> TokenStream {
    render::r_println(input)
}

/// Declares a chain processing step that transforms one type into another
/// within a Mingling pipeline.
///
/// The `#[chain]` attribute converts an ordinary function (or async function
/// with the `async` feature) into a chain step by:
/// 1. Generating a hidden struct implementing the `Chain` trait.
/// 2. Registering the chain mapping in the global chain registry.
/// 3. Keeping the original function for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program (ThisProgram):
/// #[chain]
/// fn my_step(prev: InputType) -> Next {
///     // transform `prev`...
///     OutputType::new(result)
/// }
///
/// // Explicit program name:
/// #[chain(MyProgram)]
/// fn my_step(prev: InputType) -> Next {
///     // ...
/// }
/// ```
///
/// # Resource Injection
///
/// The `#[chain]` macro supports automatic injection of global resources
/// via the 2nd to Nth parameters. You can read resources immutably with
/// `&T` or mutate them with `&mut T`.
///
/// ## Immutable Resource (`&T`)
///
/// When you write `&SomeResource` as a parameter, the macro automatically
/// resolves it from the global resource store:
///
/// ```rust,ignore
/// #[chain]
/// fn process(prev: HelloEntry, age: &Age, name: &Name) -> Next {
///     // `age` and `name` are automatically injected
///     println!("Age: {}, Name: {}", age, name);
///     NextStep::default()
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// let __age_binding = ::mingling::this::<ThisProgram>().res_or_default::<Age>();
/// let age: &Age = __age_binding.as_ref();
/// let __name_binding = ::mingling::this::<ThisProgram>().res_or_default::<Name>();
/// let name: &Name = __name_binding.as_ref();
/// ```
///
/// ## Mutable Resource (`&mut T`)
///
/// When you write `&mut SomeResource` as a parameter, the macro wraps the
/// function body in nested `__modify_res_and_return_any` calls:
///
/// ```rust,ignore
/// #[chain]
/// fn process(prev: HelloEntry, count: &mut InvocationCount, name: &Name) -> Next {
///     count.0 += 1;
///     println!("Invocation #{} for {}", count.0, name);
///     NextStep::default()
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// let __name_binding = ::mingling::this::<ThisProgram>().res_or_default::<Name>();
/// let name: &Name = __name_binding.as_ref();
///
/// ::mingling::this::<ThisProgram>().__modify_res_and_return_any(|count: &mut InvocationCount| {
///     count.0 += 1;
///     println!("Invocation #{} for {}", count.0, name);
///     NextStep::default()
/// }).into()
/// ```
///
/// Multiple `&mut` parameters are supported with proper nesting.
///
/// ## Restrictions
///
/// - The first parameter (previous type) must be taken **by move**, not by reference.
/// - Resource injection parameters **must** be references (`&T` or `&mut T`),
///   owned values are not allowed.
/// - When the `async` feature is enabled, `&mut T` cannot be used in async
///   chain functions (only `&T` is supported for async).
///
/// # Sync Example
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     MyOutput::new(name)
/// }
/// ```
///
/// # Sync Example with Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program, r_println};
///
/// #[derive(Default, Clone)]
/// struct UserName(String);
///
/// pack!(Greeting = String);
/// pack!(DisplayCount = ());
///
/// #[chain]
/// fn greet(prev: HelloEntry, user_name: &UserName, count: &mut u64) -> Next {
///     r_println!("User: {:?}", user_name);
///     *count += 1;
///     Greeting::new(format!("Hello, {}!", user_name.0))
/// }
/// ```
///
/// # Async Example (with `async` feature)
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput::new(name)
/// }
/// ```
///
/// # Async Example with Immutable Resource Injection
///
/// ```rust,ignore
/// use mingling::macros::{chain, pack, gen_program};
///
/// pack!(MyOutput = String);
///
/// #[chain]
/// async fn greet(prev: HelloEntry, prefix: &Prefix) -> Next {
///     let name = prev.first().cloned().unwrap_or_else(|| "World".to_string());
///     some_async_fn(&name).await;
///     MyOutput::new(format!("{}{}", prefix.0, name))
/// }
/// ```
///
/// # Requirements
///
/// - The function must have at least **one** parameter (the previous type in the chain).
/// - The first parameter must be taken **by move**.
/// - The function must return `Next` (the type alias generated by `gen_program!`, which equals `ChainProcess<ProgramName>`).
/// - With the `async` feature, async functions are supported; without it, async functions are rejected.
#[proc_macro_attribute]
pub fn chain(attr: TokenStream, item: TokenStream) -> TokenStream {
    chain::chain_attr(attr, item)
}

/// Declares a renderer step that renders the output of a chain to the terminal.
///
/// The `#[renderer]` attribute converts a function into a renderer by:
/// 1. Generating a hidden struct implementing the `Renderer` trait.
/// 2. Registering the renderer mapping in the global renderer registry.
/// 3. Keeping the original function for direct calls. When called directly,
///    a new `RenderResult` is created and the renderer function writes its
///    output directly to the current terminal output buffer.
///
/// Inside a `#[renderer]` function, you can use `r_print!` and `r_println!`
/// to write output to the `RenderResult` buffer.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program (ThisProgram):
/// #[renderer]
/// fn render_my_type(prev: MyType) {
///     r_println!("Output: {:?}", *prev);
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{renderer, r_println, pack, gen_program};
///
/// pack!(Greeting = String);
///
/// #[renderer]
/// fn render_greeting(prev: Greeting) {
///     r_println!("Hello, {}!", *prev);
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly **one** parameter (the type to render).
/// - The function must return `()` (unit).
/// - The function **cannot** be async.
///
/// # Fallback Renderers
///
/// The macros `gen_program!` automatically generates two fallback types that
/// you can provide renderers for:
/// - `RendererNotFound` — triggered when no matching renderer is found
/// - `DispatcherNotFound` — triggered when no matching dispatcher is found
///
/// ```rust,ignore
/// #[renderer]
/// fn fallback_dispatcher_not_found(prev: DispatcherNotFound) {
///     r_println!("Unknown command: {}", prev.join(", "));
/// }
///
/// #[renderer]
/// fn fallback_renderer_not_found(prev: RendererNotFound) {
///     r_println!("No renderer for `{}`", *prev);
/// }
/// ```
#[proc_macro_attribute]
pub fn renderer(_attr: TokenStream, item: TokenStream) -> TokenStream {
    renderer::renderer_attr(item)
}

/// Declares a completion suggestion provider for a command entry type.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `#[completion]` attribute converts a function into a completion provider by:
/// 1. Generating a hidden struct implementing the `Completion` trait.
/// 2. Registering the completion mapping for the specified entry type.
/// 3. Keeping the original function for direct calls.
///
/// # Syntax
///
/// ```rust,ignore
/// #[completion(EntryType)]
/// fn complete_my_entry(ctx: &ShellContext) -> Suggest {
///     // Return suggestions based on current input state...
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest, suggest_enum};
/// use mingling::{ShellContext, Suggest};
///
/// #[completion(MyEntry)]
/// fn complete_my_command(ctx: &ShellContext) -> Suggest {
///     if ctx.filling_argument_first("--name") {
///         return suggest!();
///     }
///     if ctx.filling_argument_first("--type") {
///         return suggest_enum!(MyEnum);
///     }
///     if ctx.typing_argument() {
///         return suggest! {
///             "--name": "Provide a name",
///             "--type": "Select a type"
///         }.strip_typed_argument(ctx);
///     }
///     suggest!()
/// }
/// ```
///
/// # Requirements
///
/// - The `comp` feature must be enabled.
/// - The function must have exactly one parameter of type `&ShellContext`.
/// - The function must return `Suggest`.
/// - The function cannot be async.
#[cfg(feature = "comp")]
#[proc_macro_attribute]
pub fn completion(attr: TokenStream, item: TokenStream) -> TokenStream {
    completion::completion_attr(attr, item)
}

/// Declares a program setup function that initializes the program instance
/// before execution.
///
/// The `#[program_setup]` attribute converts a function into a setup step by:
/// 1. Generating a struct implementing the `ProgramSetup` trait.
/// 2. The setup function receives a mutable reference to `&mut Program<G>`.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program (ThisProgram):
/// #[program_setup]
/// fn setup_my_program(program: &mut Program<ThisProgram>) {
///     program.stdout_setting.render_output = false;
/// }
///
/// // Explicit program name:
/// #[program_setup(MyProgram)]
/// fn setup_my_program(program: &mut Program<MyProgram>) {
///     // ...
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::program_setup;
/// use mingling::Program;
///
/// #[program_setup]
/// fn configure(program: &mut Program<ThisProgram>) {
///     program.with_setup(GeneralRendererSetup);
///     program.user_context.some_flag = true;
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly one parameter of type `&mut Program<G>`.
/// - The function must return `()`.
/// - The function cannot be async.
#[proc_macro_attribute]
pub fn program_setup(attr: TokenStream, item: TokenStream) -> TokenStream {
    program_setup::setup_attr(attr, item)
}

/// Declares a `Dispatcher` that uses `clap::Parser` for argument parsing.
///
/// **This macro is only available with the `clap` feature.**
///
/// The `#[dispatcher_clap]` attribute:
/// 1. Keeps the original struct definition (typically with `#[derive(clap::Parser)]`).
/// 2. Generates a dispatcher struct that parses arguments using clap and routes
///    to the chain pipeline.
/// 3. Optionally generates a `#[help]` block for displaying clap-generated help.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program (ThisProgram):
/// #[derive(clap::Parser)]
/// #[dispatcher_clap("command.name", DispatcherStruct)]
/// struct MyEntry { /* ... */ }
///
/// // With explicit error type and help:
/// #[derive(clap::Parser)]
/// #[dispatcher_clap("cmd", Disp, error = ParseError, help = true)]
/// struct CmdEntry { /* ... */ }
///
/// // With explicit program name:
/// #[derive(clap::Parser)]
/// #[dispatcher_clap(MyProgram, "cmd", Disp)]
/// struct CmdEntry { /* ... */ }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use clap::Parser;
/// use mingling::macros::dispatcher_clap;
///
/// #[derive(Parser)]
/// #[dispatcher_clap("greet", GreetDispatcher, error = GreetParseError, help = true)]
/// struct GreetArgs {
///     #[arg(short, long)]
///     name: String,
/// }
/// ```
///
/// # Options
///
/// - `error = ErrorType` — Specifies an error wrapper type for clap parse failures.
///   The error message is captured and routed to the renderer.
/// - `help = true` — Generates a `#[help]` block that displays clap's help output
///   when `--help` is passed.
#[cfg(feature = "clap")]
#[proc_macro_attribute]
pub fn dispatcher_clap(attr: TokenStream, item: TokenStream) -> TokenStream {
    dispatcher_clap::dispatcher_clap_attr(attr, item)
}

/// Registers a help request mapping between an entry type and a help struct.
///
/// This macro is used internally by the `#[help]`(macro.help.html) attribute
/// and is also available for manual registration if needed.
///
/// # Syntax
///
/// ```rust,ignore
/// register_help!(EntryType, HelpStruct);
/// ```
///
/// This adds an entry to the global `HELP_REQUESTS` registry, mapping the
/// enum variant for `EntryType` to the help rendering logic in `HelpStruct`.
#[proc_macro]
pub fn register_help(input: TokenStream) -> TokenStream {
    help::register_help(input)
}

/// Registers a dispatcher at compile time for the `dispatch_tree` feature.
///
/// This macro is called internally by `dispatcher!`(macro.dispatcher.html) when
/// the `dispatch_tree` feature is enabled. It stores the node name into the global
/// `COMPILE_TIME_DISPATCHERS` registry and generates a static variable for the
/// dispatcher instance, which is later used by `gen_program!` to generate the
/// dispatch tree routing logic.
///
/// # Syntax
///
/// ```rust,ignore
/// register_dispatcher!("node.name", DispatcherType, EntryName);
/// ```
#[proc_macro]
pub fn register_dispatcher(input: TokenStream) -> TokenStream {
    dispatcher::register_dispatcher(input)
}

/// Declares a help rendering function for an entry type.
///
/// The `#[help]` attribute converts a function into a help provider by:
/// 1. Generating a hidden struct implementing the `HelpRequest` trait.
/// 2. Registering the help mapping in the global `HELP_REQUESTS` registry.
/// 3. Keeping the original function for direct calls (with a dummy `RenderResult`).
///
/// Inside a `#[help]` function, you can use `r_print!` and `r_println!`
/// to write help text to the `RenderResult` buffer.
///
/// # Syntax
///
/// ```rust,ignore
/// #[help]
/// fn help_my_entry(prev: MyEntry) {
///     r_println!("Usage: myapp myentry [options]");
///     r_println!("  Does something useful.");
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{help, r_println, pack, gen_program};
///
/// pack!(MyEntry = Vec<String>);
///
/// #[help]
/// fn help_my_entry(_prev: MyEntry) {
///     r_println!("Usage: myapp greet [name]");
///     r_println!("Greets the user.");
/// }
/// ```
///
/// # Requirements
///
/// - The function must have exactly one parameter (the entry type to provide help for).
/// - The function must return `()`.
/// - The function cannot be async.
#[proc_macro_attribute]
pub fn help(_attr: TokenStream, item: TokenStream) -> TokenStream {
    help::help_attr(item)
}

/// Derive macro for automatically implementing the `Groupped` trait on a struct.
///
/// The `#[derive(Groupped)]` macro:
/// 1. Implements `Groupped<Group>` where the group is specified via `#[group(GroupName)]`.
/// 2. Registers the type via `register_type!` so it's included in the program enum.
/// 3. Generates `Into<AnyOutput<Group>>` and `Into<ChainProcess<Group>>` conversions.
/// 4. Adds `to_chain()` and `to_render()` methods to the struct.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(Groupped)]
/// #[group(MyProgram)]   // optional; defaults to `ThisProgram`
/// struct MyStruct {
///     field: String,
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::{Groupped, macros::{chain, gen_program, renderer, r_println}};
///
/// #[derive(Groupped)]
/// #[group(ThisProgram)]
/// struct Greeting {
///     name: String,
/// }
/// ```
///
/// This is equivalent to using `pack!` but works with custom structs that
/// have named fields. For simple wrappers, prefer `pack!`.
#[proc_macro_derive(Groupped, attributes(group))]
pub fn derive_groupped(input: TokenStream) -> TokenStream {
    groupped::derive_groupped(input)
}

/// Derive macro for automatically implementing the `EnumTag` trait on an enum
/// with unit-only variants.
///
/// The `#[derive(EnumTag)]` macro generates:
/// - `enum_info(&self) -> (&'static str, &'static str)` — returns (name, description)
///   for the current variant.
/// - `build_enum(name: String) -> Option<Self>` — constructs a variant from its
///   display name (or `#[enum_rename]` value).
/// - `enums() -> &'static [(&'static str, &'static str)]` — returns all (name, description)
///   pairs.
///
/// # Attributes
///
/// - `#[enum_desc("description text")]` — Provides a description for the variant.
/// - `#[enum_rename("display name")]` — Changes the display/build name of the variant.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(EnumTag)]
/// enum Fruit {
///     #[enum_desc("A sweet red fruit")]
///     #[enum_rename("apple")]
///     Apple,
///
///     #[enum_desc("A yellow tropical fruit")]
///     #[enum_rename("banana")]
///     Banana,
/// }
/// ```
///
/// # Requirements
///
/// - Can only be derived for **enums** (not structs or unions).
/// - All variants must be **unit variants** (no fields).
/// - Each variant is optional; variants without attributes get their Rust name as display name
///   and an empty description.
#[proc_macro_derive(EnumTag, attributes(enum_desc, enum_rename))]
pub fn derive_enum_tag(input: TokenStream) -> TokenStream {
    enum_tag::derive_enum_tag(input)
}

/// Derive macro for implementing both `Groupped` and `serde::Serialize` on a struct.
///
/// **This macro is only available with the `general_renderer` feature.**
///
/// This is identical to `#[derive(Groupped)]` but also adds `#[derive(serde::Serialize)]`
/// to the struct, which is required for the general renderer to serialize output
/// to formats like JSON, YAML, TOML, or RON.
///
/// # Syntax
///
/// ```rust,ignore
/// #[derive(GrouppedSerialize)]
/// #[group(MyProgram)]
/// struct Info {
///     name: String,
///     age: i32,
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::GrouppedSerialize;
/// use serde::Serialize;
///
/// #[derive(GrouppedSerialize)]
/// struct Info {
///     name: String,
///     age: i32,
/// }
/// ```
#[cfg(feature = "general_renderer")]
#[proc_macro_derive(GrouppedSerialize, attributes(group))]
pub fn derive_groupped_serialize(input: TokenStream) -> TokenStream {
    groupped::derive_groupped_serialize(input)
}

/// Generates the program enum and all collected types, chains, and renderers.
///
/// This macro **must** be called at the end of your program module to collect
/// all registered types, chains, renderers, and help requests into a single
/// program enum that implements `ProgramCollect`.
///
/// # Syntax
///
/// ```rust,ignore
/// // Default program name (uses `ThisProgram`):
/// gen_program!();
///
/// // Explicit program name:
/// gen_program!(MyProgram);
/// ```
///
/// # What it generates
///
/// The macro expands to:
/// 1. **`pub type Next = ChainProcess<ProgramName>`** — A convenience type alias
///    for use in chain function return types.
/// 2. **`program_comp_gen!(...)`** (with `comp` feature) — Generates completion infrastructure.
/// 3. **`program_fallback_gen!(...)`** — Generates `RendererNotFound` and `DispatcherNotFound` types.
/// 4. **`program_final_gen!(...)`** — Generates the program enum with:
///    - An enum with all packed types as variants
///    - `Display` implementation for the enum
///    - `ProgramCollect` implementation dispatching to all registered renderers and chains
///    - A `new()` constructor returning `Program<ProgramName>`
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{dispatcher, chain, renderer, gen_program};
///
/// dispatcher!("hello", HelloCommand => HelloEntry);
///
/// #[chain]
/// fn process(prev: HelloEntry) -> Next {
///     // ...
/// }
///
/// #[renderer]
/// fn render(prev: /* ... */) {
///     r_println!("Done!");
/// }
///
/// // Collect everything:
/// gen_program!();
/// ```
#[proc_macro]
pub fn gen_program(input: TokenStream) -> TokenStream {
    let name = read_name(&input);

    #[cfg(feature = "comp")]
    let comp_gen = quote! {
        ::mingling::macros::program_comp_gen!(#name);
    };

    #[cfg(not(feature = "comp"))]
    let comp_gen = quote! {};

    TokenStream::from(quote! {
        // Shit, this feature is unstable
        // TODO :: This logic will be implemented when Rust's Impl In Type Alias feature becomes stable
        // pub type Next = impl Into<::mingling::ChainProcess<#name>>;
        pub type Next = ::mingling::ChainProcess<#name>;

        #comp_gen
        ::mingling::macros::program_fallback_gen!(#name);
        ::mingling::macros::program_final_gen!(#name);
    })
}

/// Internal macro used by `gen_program!` to generate completion infrastructure.
///
/// **This macro is only available with the `comp` feature.**
///
/// This is an internal macro and should not be called directly by user code.
/// It generates a completion dispatcher, the `CompletionContext` type, and
/// the execution/render logic for shell completion.
///
/// The generated module `__completion_gen` contains:
/// - A `__comp` dispatcher that routes completion requests
/// - A `__exec_completion` chain that processes `CompletionContext` into `CompletionSuggest`
/// - A `__render_completion` renderer that outputs completion suggestions
#[proc_macro]
#[cfg(feature = "comp")]
pub fn program_comp_gen(input: TokenStream) -> TokenStream {
    let name = read_name(&input);

    #[cfg(feature = "async")]
    let fn_exec_comp = quote! {
        #[::mingling::macros::chain(#name)]
        pub async fn __exec_completion(prev: CompletionContext) -> Next {
            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<#name>(&ctx);
                    CompletionSuggest::new((ctx, suggest)).to_render()
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    #[cfg(not(feature = "async"))]
    let fn_exec_comp = quote! {
        #[::mingling::macros::chain(#name)]
        pub fn __exec_completion(prev: CompletionContext) -> Next {
            let read_ctx = ::mingling::ShellContext::try_from(prev.inner);
            match read_ctx {
                Ok(ctx) => {
                    let suggest = ::mingling::CompletionHelper::exec_completion::<#name>(&ctx);
                    CompletionSuggest::new((ctx, suggest)).to_render()
                }
                Err(_) => std::process::exit(1),
            }
        }
    };

    let comp_dispatcher = quote! {
        ::mingling::macros::dispatcher!(#name, "__comp", CompletionDispatcher => CompletionContext);
        ::mingling::macros::pack!(
            #name,
            CompletionSuggest = (::mingling::ShellContext, ::mingling::Suggest)
        );

        #fn_exec_comp

        ::mingling::macros::register_type!(CompletionContext);

        #[allow(unused)]
        #[::mingling::macros::renderer(#name)]
        pub fn __render_completion(prev: CompletionSuggest) {
            let (ctx, suggest) = prev.inner;
            ::mingling::CompletionHelper::render_suggest::<#name>(ctx, suggest);
        }
    };

    TokenStream::from(comp_dispatcher)
}

/// Registers a type into the global packed types registry for inclusion in
/// the program enum generated by `gen_program!`.
///
/// This macro is called internally by `pack!` and `#[derive(Groupped)]`(macro.derive_groupped.html)
/// and is generally not needed in user code. However, it can be used for manual
/// registration if you are implementing custom type registration outside of
/// the standard macros.
///
/// # Syntax
///
/// ```rust,ignore
/// register_type!(MyType);
/// ```
///
/// Each call inserts the type's name into the `PACKED_TYPES` global set, which
/// is later consumed by `program_final_gen!` to generate enum variants.
#[proc_macro]
pub fn register_type(input: TokenStream) -> TokenStream {
    let type_ident = parse_macro_input!(input as syn::Ident);
    let entry_str = type_ident.to_string();

    PACKED_TYPES.lock().unwrap().insert(entry_str);

    TokenStream::new()
}

/// Registers a chain mapping from a previous type to a chain struct.
///
/// This macro is called internally by `#[chain]`(macro.chain.html) and is
/// generally not needed in user code. It inserts entries into the global
/// `CHAINS` and `CHAINS_EXIST` registries.
///
/// # Syntax
///
/// ```rust,ignore
/// register_chain!(PreviousType, ChainStruct);
/// ```
///
/// The `PreviousType` is the input type of the chain step, and `ChainStruct`
/// is the generated struct that implements the `Chain` trait.
#[proc_macro]
pub fn register_chain(input: TokenStream) -> TokenStream {
    chain::register_chain(input)
}

/// Registers a renderer mapping from a type to a renderer struct.
///
/// This macro is called internally by `#[renderer]`(macro.renderer.html) and is
/// generally not needed in user code. It inserts entries into the global
/// `RENDERERS`, `RENDERERS_EXIST` and (with `general_renderer` feature)
/// `GENERAL_RENDERERS` registries.
///
/// # Syntax
///
/// ```rust,ignore
/// register_renderer!(PreviousType, RendererStruct);
/// ```
///
/// The `PreviousType` is the input type of the renderer, and `RendererStruct`
/// is the generated struct that implements the `Renderer` trait.
#[proc_macro]
pub fn register_renderer(input: TokenStream) -> TokenStream {
    renderer::register_renderer(input)
}

/// Internal macro used by `gen_program!` to generate fallback types.
///
/// This macro generates two fallback wrapper types that are essential
/// for error handling in the Mingling pipeline:
///
/// - **`RendererNotFound`** — Wraps a `String` (the name of the missing renderer).
///   Used when no matching renderer is found for a given output type.
/// - **`DispatcherNotFound`** — Wraps `Vec<String>` (the unrecognized command args).
///   Used when no matching dispatcher is found for user input.
///
/// Users can (and should) write `#[renderer]` functions for these types
/// to provide meaningful error messages.
///
/// This macro is called automatically by `gen_program!` and should not
/// be called directly by user code.
///
/// # Syntax
///
/// ```rust,ignore
/// // Called internally by gen_program!:
/// program_fallback_gen!(ThisProgram);
/// program_fallback_gen!(MyProgram);
/// ```
///
/// # Generated code equivalent
///
/// ```rust,ignore
/// pack!(ProgramName, RendererNotFound = String);
/// pack!(ProgramName, DispatcherNotFound = Vec<String>);
/// ```
#[proc_macro]
pub fn program_fallback_gen(input: TokenStream) -> TokenStream {
    let name = read_name(&input);

    let expanded = quote! {
        ::mingling::macros::pack!(#name, RendererNotFound = String);
        ::mingling::macros::pack!(#name, DispatcherNotFound = Vec<String>);
    };
    TokenStream::from(expanded)
}

/// Internal macro used by `gen_program!` to generate the final program enum
/// and its `ProgramCollect` implementation.
///
/// This is the core code generation macro that:
/// 1. Collects all registered types (from `pack!`, `#[derive(Groupped)]`, etc.) and
///    creates an enum with each type as a variant.
/// 2. Generates the `Display` implementation for the enum.
/// 3. Generates the `ProgramCollect` implementation that dispatches to all
///    registered renderers, chains, help handlers, completions, and general renderers.
/// 4. Adds a `new()` constructor on the enum returning `Program<EnumName>`.
///
/// The generated enum's representation type (`#[repr(u8)]`, `#[repr(u16)]`, etc.)
/// is automatically chosen based on the number of variants.
///
/// This macro is called automatically by `gen_program!` and should not
/// be called directly by user code.
///
/// # Syntax
///
/// ```rust,ignore
/// program_final_gen!(ThisProgram);
/// program_final_gen!(MyProgram);
/// ```
///
/// # Generated code structure
///
/// ```rust,ignore
/// #[repr(u8)]
/// pub enum MyProgram {
///     TypeA,
///     TypeB,
///     // ...
/// }
///
/// impl ProgramCollect for MyProgram {
///     type Enum = MyProgram;
///     fn render(any, r) { /* dispatches to all registered renderers */ }
///     fn do_chain(any) -> ChainProcess { /* dispatches to all registered chain steps */ }
///     fn render_help(any, r) { /* dispatches to all registered help handlers */ }
///     fn has_renderer(any) -> bool { /* checks renderer registry */ }
///     fn has_chain(any) -> bool { /* checks chain registry */ }
///     // (with comp feature) fn do_comp(...)
///     // (with general_renderer feature) fn general_render(...)
/// }
///
/// impl MyProgram {
///     pub fn new() -> Program<MyProgram> { Program::new() }
/// }
/// ```
#[proc_macro]
pub fn program_final_gen(input: TokenStream) -> TokenStream {
    let name = read_name(&input);

    let packed_types = PACKED_TYPES.lock().unwrap().clone();

    let renderers = RENDERERS.lock().unwrap().clone();
    let chains = CHAINS.lock().unwrap().clone();
    let renderer_exist = RENDERERS_EXIST.lock().unwrap().clone();
    let chain_exist = CHAINS_EXIST.lock().unwrap().clone();

    #[cfg(feature = "general_renderer")]
    let general_renderers = GENERAL_RENDERERS.lock().unwrap().clone();

    #[cfg(feature = "comp")]
    let completions = COMPLETIONS.lock().unwrap().clone();

    let packed_types: Vec<proc_macro2::TokenStream> = packed_types
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_tokens: Vec<proc_macro2::TokenStream> = renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_tokens: Vec<proc_macro2::TokenStream> = chains
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let renderer_exist_tokens: Vec<proc_macro2::TokenStream> = renderer_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let chain_exist_tokens: Vec<proc_macro2::TokenStream> = chain_exist
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "general_renderer")]
    let general_renderer_tokens: Vec<proc_macro2::TokenStream> = general_renderers
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "general_renderer")]
    let general_render = quote! {
        fn general_render(
            any: ::mingling::AnyOutput<Self::Enum>,
            setting: &::mingling::GeneralRendererSetting,
        ) -> Result<::mingling::RenderResult, ::mingling::error::GeneralRendererSerializeError> {
            match any.member_id {
                #(#general_renderer_tokens)*
                _ => Ok(::mingling::RenderResult::default()),
            }
        }
    };

    #[cfg(not(feature = "general_renderer"))]
    let general_render = quote! {};

    #[cfg(feature = "dispatch_tree")]
    let compile_time_dispatchers: Vec<String> = COMPILE_TIME_DISPATCHERS
        .lock()
        .unwrap()
        .clone()
        .iter()
        .cloned()
        .collect();

    #[cfg(feature = "dispatch_tree")]
    let dispatch_tree_nodes = {
        let entries: Vec<(String, String, String)> = compile_time_dispatchers
            .iter()
            .filter_map(|entry| {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].to_string(),
                        parts[1].to_string(),
                        parts[2].to_string(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        let get_nodes_fn = dispatch_tree_gen::gen_get_nodes(&entries);
        let dispatch_trie_fn = dispatch_tree_gen::gen_dispatch_args_trie(&entries);

        quote! {
            #get_nodes_fn
            #dispatch_trie_fn
        }
    };

    #[cfg(not(feature = "dispatch_tree"))]
    let dispatch_tree_nodes = quote! {};

    #[cfg(feature = "comp")]
    let completion_tokens: Vec<proc_macro2::TokenStream> = completions
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    #[cfg(feature = "comp")]
    let comp = quote! {
        fn do_comp(any: &::mingling::AnyOutput<Self::Enum>, ctx: &::mingling::ShellContext) -> ::mingling::Suggest {
            match any.member_id {
                #(#completion_tokens)*
                _ => ::mingling::Suggest::FileCompletion,
            }
        }
    };

    #[cfg(not(feature = "comp"))]
    let comp = quote! {};

    let help_tokens: Vec<proc_macro2::TokenStream> = HELP_REQUESTS
        .lock()
        .unwrap()
        .clone()
        .iter()
        .map(|s| syn::parse_str::<proc_macro2::TokenStream>(s).unwrap())
        .collect();

    let num_variants = packed_types.len();
    let repr_type = if num_variants <= u8::MAX as usize {
        quote! { u8 }
    } else if num_variants <= u16::MAX as usize {
        quote! { u16 }
    } else if num_variants <= u32::MAX as usize {
        quote! { u32 }
    } else {
        quote! { u128 }
    };

    let expanded = quote! {
        #[derive(Debug, PartialEq, Eq, Clone)]
        #[repr(#repr_type)]
        pub enum #name {
            #(#packed_types),*
        }

        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(#name::#packed_types => write!(f, stringify!(#packed_types)),)*
                }
            }
        }

        impl ::mingling::ProgramCollect for #name {
            type Enum = #name;
            type DispatcherNotFound = DispatcherNotFound;
            type RendererNotFound = RendererNotFound;
            fn build_renderer_not_found(member_id: Self::Enum) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(RendererNotFound::new(member_id.to_string()))
            }
            fn build_dispatcher_not_found(args: Vec<String>) -> ::mingling::AnyOutput<Self::Enum> {
                ::mingling::AnyOutput::new(DispatcherNotFound::new(args))
            }
            ::mingling::__dispatch_program_renderers!(
                #(#renderer_tokens)*
            );
            ::mingling::__dispatch_program_chains!(
                #(#chain_tokens)*
            );
            fn render_help(any: ::mingling::AnyOutput<Self::Enum>, r: &mut ::mingling::RenderResult) {
                match any.member_id {
                    #(#help_tokens)*
                    _ => (),
                }
            }
            fn has_renderer(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#renderer_exist_tokens)*
                    _ => false
                }
            }
            fn has_chain(any: &::mingling::AnyOutput<Self::Enum>) -> bool {
                match any.member_id {
                    #(#chain_exist_tokens)*
                    _ => false
                }
            }
            #dispatch_tree_nodes
            #general_render
            #comp
        }

        impl #name {
            /// Creates a new `Program<#name>` instance with default configuration.
            pub fn new() -> ::mingling::Program<#name> {
                ::mingling::Program::new()
            }

            /// Returns a static reference to the global `Program<#name>` singleton.
            pub fn this() -> &'static ::mingling::Program<#name> {
                &::mingling::this::<#name>()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Builds a `Suggest` instance with inline suggestion items.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `suggest!` macro provides a concise syntax for creating shell completion
/// suggestions. Each item can be either a simple flag or a flag with a description.
///
/// # Syntax
///
/// ```rust,ignore
/// // Empty suggestions:
/// suggest!()
///
/// // Simple flags (no description):
/// suggest! { "--flag1", "--flag2" }
///
/// // Flags with descriptions:
/// suggest! {
///     "--name": "User's name",
///     "--age":  "User's age"
/// }
///
/// // Mixed:
/// suggest! {
///     "--name": "User's name",
///     "--verbose"
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest};
/// use mingling::{ShellContext, Suggest};
///
/// #[completion(MyEntry)]
/// fn complete(ctx: &ShellContext) -> Suggest {
///     if ctx.typing_argument() {
///         return suggest! {
///             "--name": "Provide a name",
///             "--type": "Select a type"
///         }.strip_typed_argument(ctx);
///     }
///     suggest!()
/// }
/// ```
///
/// # Related
///
/// - `suggest_enum!`(macro.suggest_enum.html) — Build suggestions from an `EnumTag` enum.
#[cfg(feature = "comp")]
#[proc_macro]
pub fn suggest(input: TokenStream) -> TokenStream {
    suggest::suggest(input)
}

/// Builds a `Suggest` instance from an `EnumTag` enum's variants.
///
/// **This macro is only available with the `comp` feature.**
///
/// The `suggest_enum!` macro iterates over all variants of an `EnumTag`-derived
/// enum and creates suggestion items using each variant's display name
/// (from `#[enum_rename]`) and description (from `#[enum_desc]`).
///
/// # Syntax
///
/// ```rust,ignore
/// suggest_enum!(MyEnumType);
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use mingling::macros::{completion, suggest_enum};
/// use mingling::{ShellContext, Suggest, EnumTag};
///
/// #[derive(EnumTag)]
/// enum Fruit {
///     #[enum_desc("A sweet red fruit")]
///     #[enum_rename("apple")]
///     Apple,
///     #[enum_desc("A yellow tropical fruit")]
///     #[enum_rename("banana")]
///     Banana,
/// }
///
/// #[completion(MyEntry)]
/// fn complete(ctx: &ShellContext) -> Suggest {
///     if ctx.filling_argument_first("--fruit") {
///         return suggest_enum!(Fruit);
///     }
///     suggest!()
/// }
/// ```
///
/// # Generated code equivalent
///
/// ```rust,ignore
/// {
///     let mut enum_suggest = Suggest::new();
///     for (name, desc) in <Fruit>::enums() {
///         if desc.is_empty() {
///             enum_suggest.insert(SuggestItem::new(name.to_string()));
///         } else {
///             enum_suggest.insert(SuggestItem::new_with_desc(name.to_string(), desc.to_string()));
///         }
///     }
///     enum_suggest
/// }
/// ```
///
/// # Related
///
/// - `suggest!`(macro.suggest.html) — Build suggestions with inline syntax.
/// - `EnumTag`(derive.EnumTag.html) — The derive macro required for the enum type.
#[cfg(feature = "comp")]
#[proc_macro]
pub fn suggest_enum(input: TokenStream) -> TokenStream {
    suggest::suggest_enum(input)
}

fn read_name(input: &TokenStream) -> Ident {
    if input.is_empty() {
        Ident::new(DEFAULT_PROGRAM_NAME, proc_macro2::Span::call_site())
    } else {
        syn::parse(input.clone()).unwrap()
    }
}
