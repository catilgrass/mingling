<h1 align="center">Implementing Fallbacks</h1>
<p align="center">
    Handling error cases in your program using a fallback mechanism
</p>

## Recap

   In the last post, we introduced how to develop a basic CLI program using **Mingling**: you can use the `"greet"` subcommand to output `"Hello, World!"`, or use `"greet Alice"` to output `"Hello, Alice!"`

   But what happens when the user does not enter `"greet"`? Let's type a command and find out ⌨️

```bash
~> your-bin hello
~> your-bin hello Alice
```
 
   **It does nothing!** 👆

   Let me explain why: **Mingling** doesn't presume to act; it will not output anything to the terminal no matter what happens (except for `panic!` under `unwind`)

   This means that if you need to actively do something when your CLI program encounters an error, you have to state it explicitly.

   Fortunately, **Mingling** provides a convenient interface for this functionality: inside the `gen_program!` macro, two `FallBack` types are generated

|Type|When it occurs|How it occurs|
|-|-|-|
|RendererNotFound|When a renderer cannot be found for scheduling|Scheduled as a `Chain`|
|DispatcherNotFound|When a command is entered but no dispatcher matches|Scheduled as a `Chain`|

### The `DispatcherNotFound` Type

   Let's first focus on the `DispatcherNotFound` type. It is produced as follows:

```rust
// 1. Define the `greet` command
dispatcher!("greet", GreetCommand => GreetEntry);
 
fn main() {
    // ->> User enters "hello Alice"
    let mut program = ThisProgram::new();
 
    // 2. Import the `greet` command
    program.with_dispatcher(GreetCommand);
 
    // 3. Execute the program
    program.exec();
}
 
// ... 
 
// 5. Receive the DispatcherNotFound dispatch
#[renderer]
fn dispatcher_not_found(prev: DispatcherNotFound) {
    // 6. Output
    r_println!(
        "Cannot match any command! Current input: \"{}\"",
        prev.join(" ")
    );
}
 
// 4. Cannot match any dispatcher named `hello`
//    Forward the user's arguments as-is to DispatcherNotFound
gen_program!(); 
```
 
   The output of the above program is:

```bash
~> omg hello
Cannot match any command! Current input: "hello"
 
~> omg hello Alice
Cannot match any command! Current input: "hello Alice"
```
 
   Now, if the user enters a command that doesn't match, **Mingling** will output the appropriate message!

## The `RendererNotFound` Type

   `RendererNotFound` can be produced in two ways:

   1. The type was explicitly dispatched to a `Renderer` (using the `.to_render()` function), but the type does not have a renderer implementation
   2. The type was dispatched to a `Chain`, but the type has neither a chain nor a renderer implementation

   Generally, `RendererNotFound` **should not occur in business logic**: its dispatch means your type needs to be rendered but can't be. You can use this type to pinpoint which type is missing a renderer implementation ✏️

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
fn main() {
    let mut program = ThisProgram::new();
 
    program.with_dispatcher(GreetCommand);
    program.exec();
}
 
pack!(ResultGreetSomeone = String);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> NextProcess {
    let args = prev.inner;
    let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
 
    ResultGreetSomeone::new(name)
}
 
// Let's intentionally remove the renderer implementation for `ResultGreetSomeone`
// #[renderer]
// fn render_greet_someone(prev: ResultGreetSomeone) {
//     r_println!("Hello, {}!", *prev);
// }
 
#[renderer]
fn renderer_not_found(prev: RendererNotFound) {
    if *prev == "DispatcherNotFound" {
        return; // Exclude the "DispatcherNotFound" type
    }
 
    // Trigger `panic!` when a renderer is not found
    panic!("Renderer \"{}\" not found!", *prev);
}
 
gen_program!();
 
```
 
   The output of the above program is:

```bash
~> your-bin greet Alice
 
thread 'main' (90772) panicked at src/bin/your-bin.rs:30:5:
Renderer "ResultGreetSomeone" not found!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
