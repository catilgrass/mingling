<h1 align="center">Parsing Complex Args</h1>
<p align="center">
    Use Mingling Picker to parse complex user input
</p>

## Intro

  In the prev. example, we built a CLI app with a `"greet"` subcommand that outputs the user's first arg.

  You may have noticed the approach used was almost direct string manipulation—not very semantic, and hard to maintain long-term.

```rust
let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
```
 
  This chapter introduces a new **Mingling** feature: `Picker`. It provides a lightweight parsing solution that meshes well with **Mingling**'s typed routing.

  To enable `Picker`, edit `Cargo.toml` ✏️

```toml
[dependencies]
mingling = { 
    version = "...", 
    features = ["parser"] 
}
```
 
  Enough talk, let's get coding and rewrite the parsing logic from the prev. section ✏️

```rust
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    // Prev. approach:
    // let args = prev.inner;
    // let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
 
    // New approach with Picker
    let name = prev.pick_or((), "World").unpack();
 
    ResultGreetSomeone::new(name)
}
```
 
  `Picker` implements `pick`, `pick_or`, and `pick_or_route` for anything `Into<Vec<String>>`. These functions let you semantically **pick** args from a string list and convert them into structured data.

  In the code above:

```rust
prev.pick_or((), "World").unpack();
```
 
  Its meaning:

```rust
   prev.pick_or((), "World").unpack();
// ~~~~ ~~~~~~~ ~~  ~~~~~~~  ~~~~~~~~
// |    |       |   |        |_ unpack to String
// |    |       |   |__________ default value is "World"
// |    |       |______________ pick the first positional arg (no flag)
// |    |______________________ pick or use default
// |___________________________ from the prev. input                
```
 
## Parsing Flag Args

  If your app needs to parse flag args (e.g., `greet --name Alice`), do:

```rust
prev.pick_or(["--name", "-n"], "World").unpack();
```
 
  Its meaning:

```rust
   prev.pick_or(["--name", "-n"], "World").unpack();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~  ~~~~~~~  ~~~~~~~~
// |    |       |                 |        |_ unpack to String
// |    |       |                 |__________ default value is "World"
// |    |       |____________________________ pick the value after "--name" or "-n"
// |    |____________________________________ pick or use default
// |_________________________________________ from the prev. input                
```
 
## About `.unpack()` 💡

  You may have noticed `Picker` calls `.unpack()` at the end of parsing. It converts the parsed result into structured info.

  For a single pick, `.unpack()` returns a single value. For multiple picks, `Picker` returns a tuple:

```rust
let name_single: String = prev.clone().pick_or((), "World").unpack();
let (name, age, id) = prev
    .pick::<String>(["--name", "-n"])
    .pick::<u8>(["--age", "-a"])
    .pick::<u32>(["--id", "-I"])
    .unpack();
 
// Parses: --name Alice --age 21 --id 0711251
```
 
> [!IMPORTANT]
> `Picker` is very order-sensitive, esp. with positional args: it parses sequentially.
>
> If you need to parse positional args, make sure to pick & consume all **flag args** first.

## Using `pick_or_route` for Edge Cases

  Ha, as the old saying goes: "Never trust your users." Missing required args, type mismatches, enabling mutually exclusive options—these are all headache-inducing edge cases.

  `pick_or_route` handles these by routing the chain to a dedicated error-handling type, giving you fine-grained error control.

  Let's write a simple example showing basic usage:

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
pack!(ResultGreetSomeone = String);
pack!(ErrorGreetNoNameProvided = ());
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    // Use `pick_or_route` to extract the `--name` arg
    // If missing or parse fails, route to ErrorGreetNoNameProvided
    let pick_result = prev
        .pick_or_route(
            ["--name", "-n"],
            ErrorGreetNoNameProvided::default().to_render(),
        )
        // After using any routable method, `unpack` returns `Result<Value, Route>`
        .unpack();
 
    // Use the `route!` macro to expand `pick_result`,
    // If it's `Err`, the chain returns here, routing to the specified type
    let name = route!(pick_result);
    ResultGreetSomeone::new(name).to_chain()
}
 
// Handles rendering for `ErrorGreetNoNameProvided`
#[renderer]
fn render_err_greet_no_name_provided(_prev: ErrorGreetNoNameProvided) {
    r_println!("Error: No name provided.")
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    r_println!("Hello, {}!", *prev);
}
```
 
  Using `pick_or_route` makes the code a bit more complex: `.unpack()` no longer returns the value directly, but `Result<Value, Route>`.

  However, **Mingling** provides the `route!` macro to simplify expansion. It's not complex—just cuts some boilerplate:

```rust
let name = route!(pick_result);
 
// Expands to
let name = match pick_result {
    Ok(r) => r,
    Err(e) => return e,
};
```
 
## Post-Processing Extracted Values

  After using `pick` to extract user input, you can use `after` or `after_or_route` to process the arg immediately ✏️

```rust
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    let name = prev
        .pick_or(["--name", "-n"], "World")
        // After extracting `--name`, format it immediately
        .after(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unpack();
 
    ResultGreetSomeone::new(name) // name is now formatted
}
```
 
  Similarly, use `after_or_route` to handle format errors in input args ✏️

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
pack!(ResultGreetSomeone = String);
pack!(ErrorGreetNameTooLong = usize);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    let pick_result = prev
        .pick_or(["--name", "-n"], "World")
        // Unlike `after`, this borrows &String
        .after_or_route(|name: &String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string();
 
            // Check name length, route to error type if too long
            let len = name.len();
            if len < 32 {
                Ok(name.clone())
            } else {
                Err(ErrorGreetNameTooLong::new(len).to_render())
            }
        })
        .unpack();
    let name = route!(pick_result);
 
    ResultGreetSomeone::new(name).to_chain()
}
 
#[renderer]
fn render_error_greet_name_too_long(prev: ErrorGreetNameTooLong) {
    let len = *prev;
    r_println!("Error: name too long (length: {} > 32)", len);
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    r_println!("Hello, {}!", *prev);
}
```
 
## Parsing Booleans

  `Picker` can parse **bool** types too, but with both explicit and implicit modes:

  |Mode|Format|
  |-|-|
  |Explicit|`--confirm true` or `--confirm yes`|
  |Implicit|`--confirmed`|

  - Using `.pick` on `bool` uses implicit parsing: flag present → `true`
  - Using `.pick` on `mingling::parser::Yes` or `mingling::parser::True` uses explicit parsing; the value must be `true` / `yes` to be recognized as `true`

  Generally, implicit parsing is enough, but for positional args or important confirmations, explicit logic might be more semantic.

```rust
#[chain]
fn handle_some_entry(prev: SomeEntry) -> Next {
    let confirmed: bool = prev.pick::<Yes>(()).unpack().is_yes();
    let confirm: bool = prev.pick::<bool>(["--confirm", "-C"]).unpack();
 
    // other logic
}
```
 
## Special Use: `usize` Parsing

  **Mingling** has a special use for `usize`: parsing strings like `25G`, `32mb`, etc. ✏️

```rust
#[test]
fn parse_size() {
    let vec = vec!["--size".to_string(), "25mib".to_string()];
    let size: usize = vec.pick(["--size", "-S"]).unpack();
    assert_eq!(size, 25 * 1024 * 1024);
}
```
 
## Custom Parsable Types

  Use the `Pickable` trait to make your types parsable by `Picker`. This is where `Picker`'s extensibility comes from ✏️

```rust
// Must implement Default: parse failures record the default directly
#[derive(Default)]
pub struct Address {
    ip: String,
    port: u16,
}
 
impl Pickable for Address {
    type Output = Self;
    fn pick(args: &mut Argument, flag: Flag) -> Option<Self::Output> {
        // Extract raw string from Argument using Flag
        let raw = args.pick_argument(flag)?;
 
        // Parse raw string into structured data
        let parts: Vec<&str> = raw.split(':').collect();
        let ip = parts.first()?.to_string();
        let port: u16 = parts.get(1)?.parse().ok()?;
 
        Some(Address { ip, port })
    }
}
```
 
  With `Pickable` implemented for `Address`, we can now use `ip:port` format for input ✏️

```rust
dispatcher!("connect", ConnectCommand => ConnectEntry);
 
pack!(ResultConnected = Address);
 
#[chain]
fn handle_connect_entry(prev: ConnectEntry) -> Next {
    let address: Address = prev.pick("--addr").unpack();
    ResultConnected::new(address)
}
 
#[renderer]
fn render_connected(prev: ResultConnected) {
    let addr = prev.inner;
    r_println!("Connected: IP: {} PORT: {}", addr.ip, addr.port);
}
```
 
  Running it:

```bash
~> your-bin connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## Auto-Implementing Pickable for Enums

  No need to manually implement `Pickable` for enums: `Picker` auto-implements it for any type that implements `PickableEnum`, as long as it also implements `EnumTag` ✏️

```rust
// Debug  : for rendering
// Default: for Picker parsing
// EnumTag: for implementing PickableEnum
#[derive(Debug, Default, EnumTag)]
pub enum Fruits {
    #[default]
    Apple,
    Banana,
    Orange,
}
 
// Implement PickableEnum for Fruits
impl PickableEnum for Fruits {}
```
 
  Now you can directly use `Picker` to parse this type ✏️

```rust
pack!(ResultFruit = Fruits);
 
#[chain]
fn handle_eat_fruit_entry(prev: EatFruitEntry) -> Next {
    let fruit: Fruits = prev.pick("--fruit").unpack();
    ResultFruit::new(fruit)
}
 
#[renderer]
fn render_ate_fruit(prev: ResultFruit) {
    r_println!("Picked fruit: {:?}", *prev);
}
```
 
  That's all for `Picker`'s usage. In the next chapter, I'll introduce how to implement help docs for commands in **Mingling**.

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
