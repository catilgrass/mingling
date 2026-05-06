<h1 align="center">Creating your first Program</h1>
<p align="center">
    Learn <b>Mingling</b> and use it to create your first command-line program
</p>

## Intro

  This chapter will guide you through **Mingling** step by step.

  Before we start, let me explain what **Mingling** can do:

  Without extra features, it is a sub-command dispatch system based on `proc-macro`: it matches user input, finds & creates the corresponding data, then pushes that data into a dispatcher that continually transforms its type. When the data can no longer be transformed, the program renders the final result to the terminal.

  In other words, you need to understand a new dev paradigm: **a fully type-based dispatch system**. This may feel **frustrating** at first, but once you get the hang of it, you'll be able to write CLI apps that are super easy to modify and extend.



## Creating a Basic Program

  Next I'll walk you through creating a basic program—I assume you already have an empty Rust project ready!

#### 1. Add Dependencies

  Add the following deps to `Cargo.toml` ✏️

```toml
[dependencies]
mingling = "0.1.7"
 
# If you want the latest, try the version hosted on Github
mingling = { git = "https://github.com/catilgrass/mingling", branch = "main" }
```

> [!NOTE]
>
> This version matches the **Mingling** version used when writing this doc. Check [crates.io](https://crates.io/crates/mingling) for the latest release! 😄 
>
> **Mingling** docs are actively updated to keep pace with the latest version.



#### 2. Create the Program

  Now, create the program in `src/main.rs` ✏️

```rust
fn main() {
    // Create ThisProgram and run it
    ThisProgram::new().exec();
}
 
// The gen_program! macro collects *all preceding* components & types
// then generates the `ThisProgram` struct
mingling::macros::gen_program!();
```

> [!TIP]
>
> When `gen_program!()` expands, it gathers info from other components & types that were expanded before it. This means you must place `gen_program!()` at the very last expansion point in the crate.
>
> I recommend putting it at the end of `main.rs` or `lib.rs`.



#### 3. Create a Command

  Of course, the program currently does nothing—it won't output anything at runtime. So let's create our first command `greet` and say hi to someone ✏️

```rust
fn main() {
    // ...
}
 
// Create a dispatcher, binding GreetCommand to the "greet" sub-command
// When the user specifies this command, send GreetEntry to the dispatcher
dispatcher!("greet", GreetCommand => GreetEntry);
 
// ...
gen_program!();
```

  Don't be scared by the sudden macro and two new types! Let me explain what this macro does:

##### About the `dispatcher!` macro 💡

1. It creates a `GreetCommand` struct and implements the `Dispatcher` trait

​    *This tells the framework: there's a new dispatcher that will handle a sub-command's behavior.*

2. It implements the `Dispatcher` trait's `node(&self) -> Node` function, setting the node to `"greet"`

​    *This tells the framework: this dispatcher handles the `"greet"` sub-command.*

3. It implements the `Dispatcher` trait's `begin` function, converting the user's full input into the first type `GreetEntry`

​    *This tells the framework: when this dispatcher is matched, it sends a `GreetEntry` type to the dispatcher for further processing.*

  In short: **"When user types `greet`, I create a `GreetEntry` and throw it into the dispatcher for conversion."**



#### 4. Register the Command

  After creating the `Dispatcher`, we have two types: `GreetCommand` and `GreetEntry`. First, register `GreetCommand` with `ThisProgram` ✏️

```rust
fn main() {
    let mut program = ThisProgram::new();
 
    // Register the dispatcher
    program.with_dispatcher(GreetCommand);
    program.exec();
}
```

  Now `ThisProgram` recognizes the `"greet"` sub-command, but the framework still doesn't know what `"greet"` should do. That's where we implement the actual logic:



#### 5. Implement Rendering Behavior

  We want `"greet"` to output `"Hello, World"`: since we're outputting to the screen, we can use another **Mingling** component, `Renderer`, which handles rendering data to the terminal ✏️

```rust
// ...
dispatcher!("greet", GreetCommand => GreetEntry);
 
// Declare a renderer `render_greet`, specifying the previous type as `GreetEntry`
#[renderer]
fn render_greet(_prev: GreetEntry) {
    r_println!("Hello, World!");
}
 
// ...
gen_program!(); // The renderer will be registered with the program
```

  For functions marked with `#[renderer]`, **Mingling** strictly enforces only one function signature:

```rust
#[renderer]
fn renderer_name (_prev: PreviousType) {  }
```

  The macro reads the type of the first param and tells `gen_program!` that this function renders that type.

##### About `r_println!()` 💡

  You might notice that the print macro used inside `#[renderer]` is `r_println!` instead of `println!`. This is because the framework's rendering logic doesn't happen inside that function: after `#[renderer]` expands, it injects a `r: &mut RenderResult` into the function; `r_println!` appends the message to the `RenderResult`, and after the dispatcher closes, the final rendered data is handed to `Program::exec` for output.



#### 6. Add Execution Logic

  I bet you're already itching to implement something like `greet Alice` to output `"Hello, Alice!"`—and this section is about to do just that!

  **Mingling**'s core execution flow is `Dispatcher -> Chain -> Renderer`, and the key part is `Chain`: it converts the input data type into another type, then lets the dispatcher find the next `Chain` or `Renderer` based on the result type ✏️

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
// Wrap the intermediate type `ResultGreetSomeone`
pack!(ResultGreetSomeone = String);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> NextProcess {
    let args = prev.inner;
    let name = args
     .first()
     .cloned()
     .unwrap_or_else(|| "World".to_string());
 
    // Wrap into intermediate type
    ResultGreetSomeone::new(name)
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    // Deref prev to get the raw type
    r_println!("Hello, {}!", *prev); 
}
```

  Just like `#[renderer]`, we created a `#[chain]` that processes type `GreetEntry` and outputs `ResultGreetSomeone`.

  This inserts a `Chain` between the original `Dispatcher` and `Renderer`: it extracts the user's input params (or falls back to "World"), then passes them to the renderer to print to the terminal.

##### About `NextProcess` 💡

  `NextProcess` is a placeholder generated by `gen_program!()`. After `#[chain]` expands, it's replaced by a type-erased type `ChainProcess<ThisProgram>` that the dispatcher can recognize, helping reduce boilerplate code.

> [!NOTE]
>
> `NextProcess` is a temporary solution; the next update will wait until Rust's `Impl In Type Aliases` feature is stable.
>
> **But don't worry**: the next `NextProcess` update won't introduce **breaking changes!**

##### About `pack!` 💡

  `pack!` is an **extremely** frequently used macro in **Mingling** development: it wraps any type into another type and auto-derives the traits the framework needs.

  Its syntax is as simple as you see:

```rust
pack!(PackedType = RawType);
```

  Note: `pack!` doesn't support types with lifetimes, because types are always moved (not borrowed) between dispatchers.



#### 7. Compile & Run

  Alright, we've completed a basic CLI app. Here's the full code—you can paste it and run it directly:

```rust
use mingling::macros::{chain, dispatcher, gen_program, pack, r_println, renderer};
 
fn main() {
    let mut program = ThisProgram::new();
    program.with_dispatcher(GreetCommand);
    program.exec();
}
 
dispatcher!("greet", GreetCommand => GreetEntry);
 
pack!(ResultGreetSomeone = String);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> NextProcess {
    let args = prev.inner;
    let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
 
    ResultGreetSomeone::new(name)
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    r_println!("Hello, {}!", *prev);
}
 
gen_program!();
```

  Output:

```bash
~> your-bin greet
Hello, World!
~> your-bin greet Alice
Hello, Alice!
```

  At this point, you have successfully created a basic **Mingling** command-line program. The next chapter will explain how to implement a fallback mechanism for your command-line program to handle cases where a command or renderer does not exist.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
