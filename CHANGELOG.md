# Changelogs

### Release 0.1.8

#### Fixes:
None

#### Optimizings:
None

#### Features:

1. **\[macros\]** The `gen_program!()` macro now generates `pub fn this() -> &'static Program<#name>` for the generated program type, providing convenient static accessors.
2. **\[macros\]** The `#[chain]` macro now supports resource injection parameters (2nd to Nth). When you write:

```rust
#[chain]
fn process(prev: HelloEntry, age: &Age, name: &Name) -> NextProcess {
    // age and name are automatically injected from global resources
}
```

Will expand: 

```rust
fn proc(prev: HelloEntry) -> ChainProcess<ThisProgram> {
    let age: &Age = ::mingling::this::<ThisProgram>()
        .res_or_default::<Age>()
        .as_ref();
    let name: &Name = ::mingling::this::<ThisProgram>()
        .res_or_default::<Name>()
        .as_ref();
    // original function body inlined here
}
```

3. **\[mingling\]** Added the `dispatch_tree` feature. When enabled, it will automatically generate a prefix tree, improving the command lookup efficiency from O(n) to O(len)

4. **\[mingling\]** Added `mingling::feature` module for runtime feature detection. You can now check which features are enabled at compile time:

```rust
// Example: Check if a feature is enabled
if mingling::feature::MINGLING_ASYNC {
    // async feature is enabled
}
```

5. **\[core\]** Added `with_hook` functions to embed callback events into the program lifecycle
6. **\[core\]** Added `user_context.run_hook` configuration item to control whether the program runs hooks
7. **\[core\]** Added `exec_and_exit`, which will return an `i32` exit code after the program ends
8. **\[core\]** Added `ExitCodeSetup`, you can control the program's exit code by modifying the `mingling::res::ExitCode` resource

```rust
#[chain]
fn your_chain(_prev: Prev) -> NextProcess {
    // Use `modify_res` to modify the value of `ExitCode`
    this::<ThisProgram>().modify_res(|r: &mut ExitCode| r.exit_code = 1);

    // Or use:
    mingling::res::update_exit_code::<ThisProgram>(1);
    // ...
}
```

9. **\[core\]** `RenderResult` now carries new data `exit_code`

10. **\[core\]** Added `modify` function to `ResourceMarker` for modifying a program's global resources

```rust
// Example
ExitCode::modify::<ThisProgram>(|code| {
    code.exit_code = 1;
});

// Equivalent to:
this::<ThisProgram>().modify_res::<ExitCode>(|code| {
    code.exit_code = 1;
});
```

11. **\[core\]** Added panic catch for program execution. 

#### **BREAKING CHANGES**:

1. **\[core\]** The signature of `exec` has been changed to `exec(self) -> i32` (previously was `exec(self)`)

--- 

### Release 0.1.7 (2026-05-04)

#### Fixes:

1. Fixed a build failure on **Windows** caused by `mingling_core/src/program.rs`
2. **\[picker\]** Fixed an issue where the `Pickable` trait for `Yes` and `True` types could not correctly parse explicit boolean `--value true`

#### Optimizings:

1. **\[macros\]** Optimized the memory usage of the `gen_program!()` macro: the internal generated enum now uses the smallest possible integer representation (`u8`, `u16`, `u32`, or `u128`) based on the number of packed types, instead of always using `u32`.

#### Features:

1. **\[mingling\]** Added the scaffolding tool `mling`, which can quickly deploy and test your command-line programs
2. **\[macros\]** Completed the `clap` feature: **Mingling** now supports parsing input using `clap::Parser`

```rust
#[derive(Groupped, clap::Parser)]
#[dispatcher_clap("your_cmd", YourClapCommand)]
// #[dispatcher_clap("...", ..., error = YourCommandParseError)] // Dispatch when parse failed
// #[dispatcher_clap("...", ..., help = true)] // Enable clap help
struct YourCommandEntry {
    #[arg(long, short)]
    str_param: String,
    
    #[arg(long, short)]
    path_param: PathBuf,
}
```

3. **\[clap\]** Added the `stdout_setting.clap_help_print_behaviour` configuration item to `Program`, used to control the behavior of Clap Help
4. **\[core\]** Added function `new_with_args` to `Program`
5. **\[core\]** Added function `dispatch_args_dynamic` to `Program`
6. **\[core\]** Impl `std::io::Write` trait for `RenderResult`
7. **\[core\]** Added Help system, which allows binding an event for `--help` to an `Entry` via the `help!` macro

```rust
#[help]
fn your_command_help(_prev: YourEntry) {
    r_println!("Your help docs");
}
```

8. **\[core\]** Added the function `build_comp_script_to` to the `mingling::build` module: supports outputting completion scripts precisely to a specified directory
9. **\[macros\]** Added the `route!` macro, which allows quick error routing within the `chain!` function. Usage is as follows:

```rust
// Before
#[chain]
fn parse(prev: PickEntry) -> mingling::ChainProcess<ThisProgram> {
    let picker = Picker::new(prev.inner);
    let pick_result = picker
        .pick_or_route((), NoNameProvided::default().to_render())
        .unpack();

    match pick_result {
        Ok(name) => {
            // use name here
        }
        Err(e) => {
            // handle error route here
            e
        }
    }
}

// After
#[chain]
fn parse(prev: PickEntry) -> mingling::ChainProcess<ThisProgram> {
    let picker = Picker::new(prev.inner);
    let name: String = route! {
        picker
            .pick_or_route((), NoNameProvided::default().to_render())
            .unpack()
    };

    // use name here
}
```

10. Added a resource system to `Program` for managing global resources [Details](docs/res/changlog_examples/feat_program_res.rs)

```rust
// Define global resource
#[derive(Debug, Default, Clone)]
struct Global {
    name: String,
    age: i32,
}

// Add global resource
program.with_resource(Global::default());

// Read the global resource
let global = this::<ThisProgram>().res_or_default::<Global>();

// Modify the global resource
this::<ThisProgram>().modify_res(|r: &mut Global| {
    r.name = name;
    r.age = age
});
```

11. **\[picker\]** For any type that can `Into<Vec<String>>`, `.pick()`, `.pick_or()`, and `.pick_or_route()` functions are now supported

```rust
// Before
let name: String = Picker::new(prev.inner).pick("--name").unpack();

// Now
let name: String = prev.pick("--name").unpack();
```

#### **BREAKING CHANGES**:

1. **\[macros\]** Removed macro `dispatcher_render!` from `mingling_macros`
2. **\[core\]** The `<..., Group>` in `Program<Collect, Group>` no longer requires `std::fmt::Display`
3. **\[core\]** Changed `Program<Collect, Group>` to `Program<Collect>` (merged the Group and Collect types)
4. **\[picker\]** When performing `unpack` or `unpack_directly` on the result of the first `pick` of `Picker`, it no longer returns a tuple

```rust
// Before
#[chain]
fn parse_sth(prev: SomeEntry) -> NextProcess {
    let str: String = Picker::<()>::new(prev.inner)
        .pick_or((), "None")
        .unpack_directly().0;
    let parsed = Something::new(ok);
    parsed
}

// Now
#[chain]
fn parse_sth(prev: SomeEntry) -> NextProcess {
    let str: String = Picker::<()>::new(prev.inner)
        .pick_or((), "None")
        .unpack_directly(); // Directly return the type instead of a tuple
    let parsed = Something::new(ok);
    parsed
}
```

5. **\[core\]** Removed `mingling::marker::NextProcess` and moved its creation process to `gen_program!()`

```rust
use mingling::marker::NextProcess; // Remove this 

// NextProcess generated here
gen_program!();
```

6. **\[picker\]** Simplified `Picker` logic:

     - `Picker` no longer requires the generic parameter `<G>` by default; it only needs it when using `pick_or_route` or `after_or_route`

     - Additionally, if no `or_route` operations are used, the `unpack_directly` function is no longer available; `unpack` will directly extract the inner value

```rust
// Before
let (name, age) = Picker::<()>::new(prev.inner) // had to specify an arbitrary type even for routers Picker without routes
    .pick::<String>(())
    .pick::<i32>(())
    .unpack_directly(); // had to use `unpack_directly` to get the inner value

// After
let (name, age) = Picker::new(prev.inner) // no longer need to specify an unused route type
    .pick::<String>(())
    .pick::<i32>(())
    .unpack(); // no longer need to use `unpack_directly` 

// But ...
let (name, age) = Picker::new(prev.inner)
    .pick::<String>(())
    .pick_or_route::<i32>((), NoNumberProvided::default().to_render()) // if a route type is specified
    .unpack(); // will return Result<Value, Route>
```

7. **\[macros\]** The enum generated by `gen_program!()` no longer has a default variant (`__FallBack`), and the `#[default]` attribute has been removed accordingly.

8. **\[macros\]** Removed `#[derive(Debug)]` from generated pack types to remove unnecessary trait bounds.

9. **\[macros\]** **\[core\]** **\[mingling\]** Removed the `full` feature from all crates.

---

### Release 0.1.6 (2026-04-20) **\[YANKED 26.4.24\]**

`Mingling` 0.1.6 primarily focuses on optimizing the writing experience and code completion.

> [!CAUTION]
>
> This version cannot be built correctly on **Windows**, please do not use this version.

> [!warning]
>
> To align with the `mingling` version, `mingling_core` and `mingling_macros` will skip version `0.1.5` and be released directly as `0.1.6`.

#### Fixes:

1. **\[core\]** Fixed an issue where the `Powershell` completion script could not be used.

#### Features:

1. **\[core\]** Added support for completion descriptions in `Powershell`.
2. **\[core\]** Added more context-based completion functions, such as `filling_argument` and `typing_argument`. For details, see [Docs.rs](https://docs.rs/mingling/0.1.6/mingling/)

#### **BREAKING CHANGES**:

1. **\[macros\]** The `chain!` macro no longer requires explicit type conversion when routing a type to `Chain`.
```rust
// Before
#[chain]
fn proc(_prev: SomeType) -> NextProcess {
    let result = SomeResult::new(());
    result.to_chain()
}

// Now
#[chain]
fn proc(_prev: SomeType) -> NextProcess {
    let result = SomeResult::new(());
    result // No need for `to_chain()`
}
```

2. **\[macros\]** Moved type registration from the `chain!` and `renderer!` macros forward to the `pack!` and `derive Groupped` macros

3. **\[core\]** **\[macros\]** Added an `async` feature, which is disabled by default. `Mingling` no longer forces a dependency on an Async Runtime.

4. **\[picker\]** Changed the signature of `pick_or` from `(..., or: TNext)` to `(..., or: impl Into<TNext>)`

---

> [!NOTE]
>
> Versions 0.1.0 through 0.1.5 were released before this CHANGELOG file existed (which was introduced in 0.1.6). The entries above have been retroactively reconstructed from git history and may not be fully comprehensive.

---

### Release 0.1.5 (2026-04-12)

#### Fixes:
None

#### Features:

1. **\[completion\]** Added the completion system, including `ShellContext`, shell suggestion generation, and completion script build support (`build_comp_script_to`)
2. **\[completion\]** Added `YesOrNo` and `TrueOrFalse` pickable boolean types for completion
3. **\[core\]** Implemented `mingling::this` function for accessing the current program instance
4. **\[workspace\]** Added workspace configuration and example projects
5. **\[docs\]** Added architecture diagram, project branding, and README structure improvements

#### BREAKING CHANGES:

1. **\[macros\]** Renamed `DefaultProgram` to `ThisProgram` and removed `ThisProgram` marker type

---

### Release 0.1.4 (2026-04-06)

#### Fixes:
None

#### Features:

1. **\[picker\]** Added vector pickers for collecting multiple values
2. **\[picker\]** Added error routing to `Picker` with generic route type
3. **\[picker\]** Added `after` method for post-processing picked values
4. **\[macros\]** Added `Groupped` derive macro for automatic trait implementation
5. **\[macros\]** Added `general_renderer` support with serialization formats (behind feature flag)
6. **\[macros\]** Simplified attribute parsing in macros

#### BREAKING CHANGES:
None

---

### Release 0.1.3 (2026-04-01)

#### Fixes:

1. **\[core\]** Added early exit for renderer not found in execution loop
2. **\[core\]** Added default error handling methods to `ProgramCollect` trait

#### Features:

1. **\[core\]** Replaced typeid-based dispatch with enum-based dispatch for better performance
2. **\[macros\]** Renamed `chain_struct` macro to `pack`
3. **\[docs\]** Added documentation for `mingling_core` and public items in parser modules

#### BREAKING CHANGES:

1. **\[macros\]** The `chain_struct!` macro has been renamed to `pack!`

---

### Release 0.1.2 (2026-03-31)

#### Fixes:
None

#### Features:

1. **\[parser\]** Added argument parser module with `Picker` API
2. **\[parser\]** Added `Argument` type to picker builtins and exposed `Picker` publicly
3. **\[core\]** Added `From<()>` implementation for `Flag`

#### BREAKING CHANGES:
None

---

### Release 0.1.1 (2026-03-29)

#### Fixes:
None

#### Features:

1. **\[core\]** Replaced `ChainProcess` type alias with an enum for better type safety
2. **\[core\]** Added `general_renderer` and `full` features
3. **\[core\]** Removed `ProgramEnd` and `NoChainFound` hint markers
4. **\[mingling\]** Created the `mingling` umbrella crate with core re-exports and documentation

#### BREAKING CHANGES:

1. **\[core\]** `ChainProcess` changed from a type alias to an enum; conversion code may need updating

---

### Release 0.1.0 (2026-03-29)

Initial release of the Mingling framework.

#### Features:

1. **\[core\]** Basic chain processing pipeline with `#[chain]` and `#[renderer]` macros
2. **\[macros\]** `gen_program!` for program generation, `pack!` for wrapper types, `dispatcher!` for command routing
3. **\[core\]** `Program` struct with dispatcher registration and execution
4. **\[core\]** `RenderResult` for terminal output buffering
5. **\[docs\]** README and license files

---
