# Changelogs

### Release 0.1.8

#### Fixes:
None

#### Optimizings:
None

#### Features:

1. **\[macros\]** The `gen_program!()` macro now generates `pub fn this() -> &'static Program<#name>` for the generated program type, providing convenient static accessors.

#### **BREAKING CHANGES**:
None

--- 

### Release 0.1.7

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

### Release 0.1.6 **\[YANKED\]**

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
