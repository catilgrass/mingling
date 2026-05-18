<h1 align="center">解析复杂参数</h1>
<p align="center">
    使用 Mingling Picker 解析复杂的用户输入
</p>

## 引言

  在前文的示例中，我们成功创建了带有 `"greet"` 子命令的命令行程序，可以输出用户输入的第一个参数。
  
  您也注意到了，示例当中使用的方式近乎直接操作字符串，不够语义化，这不利于长期维护。

```rust
let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
```
 
  而本章节将会引入新的 **Mingling** 特性：`Picker`，它提供轻量且和 **Mingling** 类型路由高度契合的命令解析方案。

  要启用 `Picker`，您需要修改 `Cargo.toml` ✏️

```toml
[dependencies]
mingling = { 
    version = "...", 
    features = ["parser"] 
}
```
 
  好了，多的不说，让我们上手编辑代码，重写前文的解析代码 ✏️
  
```rust
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    // 前文中使用的方式：
    // let args = prev.inner;
    // let name = args.first().cloned().unwrap_or_else(|| "World".to_string());
  
    // 引入 Picker 后使用的方式
    let name = prev.pick_or((), "World").unpack();
  
    ResultGreetSomeone::new(name)
}
```
 
  `Picker` 为所有 `Into<Vec<String>>` 实现了 `pick` `pick_or` `pick_or_route` 函数：它们可以语义化地从字符串列表中 **拾取 (Pick)** 参数，并转换为结构化数据。
  
  对于上述示例中的代码：
  
```rust
prev.pick_or((), "World").unpack();
```
 
  它的语义为：
  
```rust
   prev.pick_or((), "World").unpack();
// ~~~~ ~~~~~~~ ~~  ~~~~~~~  ~~~~~~~~
// |    |       |   |        |_ 解包为 String
// |    |       |   |__________ 默认值为 "World"
// |    |       |______________ 取出第一个位置参数（不指定标志）
// |    |______________________ 拾取或使用默认
// |___________________________ 从前一个输入中                
```
 
## 解析标志参数

  若您的程序设计需要解析标志参数 (例如：`greet --name Alice`)，可以使用如下方式：

```rust
prev.pick_or(["--name", "-n"], "World").unpack();
```
 
  同理，它的语义为：
  
```rust
   prev.pick_or(["--name", "-n"], "World").unpack();
// ~~~~ ~~~~~~~ ~~~~~~~~~~~~~~~~  ~~~~~~~  ~~~~~~~~
// |    |       |                 |        |_ 解包为 String
// |    |       |                 |__________ 默认值为 "World"
// |    |       |____________________________ 取出 "--name" 或 "-n" 后面的参数
// |    |____________________________________ 拾取或使用默认
// |_________________________________________ 从前一个输入中                
```
 
## 关于 `.unpack()` 💡

  您可能注意到了，`Picker` 在命令解析的最后，会执行一个 `.unpack()` 函数，它的作用是将前面解析出来的结果，转换为结构化信息。
  
  对于只拾取了一次的数据来说，`.unpack()` 会返回单个数据，而对于多次拾取，`Picker` 则会返回元组：
  
```rust
let name_single: String = prev.clone().pick_or((), "World").unpack();
let (name, age, id) = prev
    .pick::<String>(["--name", "-n"])
    .pick::<u8>(["--age", "-a"])
    .pick::<u32>(["--id", "-I"])
    .unpack();
 
// 可解析参数 --name Alice --age 21 --id 0711251
```
 
> [!IMPORTANT]
> `Picker` 对解析顺序极其敏感，特别是位置参数：因为它是顺序解析的
>
> 若您需要解析位置参数，请确保解析前已拾取并消费所有 **标志参数**

## 使用 `pick_or_route` 处理边界情况

  哈哈，就像那句老话：“永远不要相信你的用户”，为了应对错误情况：必要参数缺失、输入类型不匹配、同时启用了互斥选项，这些都是令人头疼的边界情况。
  
  `pick_or_route` 便用于上述问题发生时，能将执行链路由到专门的错误处理类型上，以提供精细的错误处理逻辑。
  
  让我们先编写一个简单的示例来展示基本的用法：
  
```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
pack!(ResultGreetSomeone = String);
pack!(ErrorGreetNoNameProvided = ());
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    // 使用 `pick_or_route` 提取 `--name` 参数
    // 如果不存在或解析失败，则路由到 ErrorGreetNoNameProvided
    let pick_result = prev
        .pick_or_route(
            ["--name", "-n"],
            ErrorGreetNoNameProvided::default().to_render(),
        )
        // 在使用了任何可路由的函数后，`unpack` 将会返回 `Result<Value, Route>`
        .unpack();
 
    // 使用 route! 宏展开 `pick_result`，
    // 若内部为 Err，该链在此处返回，并路由到指定类型
    let name = route!(pick_result);
    ResultGreetSomeone::new(name).to_chain()
}
 
// 承接 `ErrorGreetNoNameProvided` 的渲染
#[renderer]
fn render_err_greet_no_name_provided(_prev: ErrorGreetNoNameProvided) {
    r_println!("Error: No name provided.")
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    r_println!("Hello, {}!", *prev);
}
```
 
  若使用 `pick_or_route`，写法会变得相对复杂：因为 `.unpack()` 不再直接返回参数，而是 `Result<Value, Route>`
  
  不过 **Mingling** 提供了简化展开的宏 `route!`，它不复杂，只是省略了一部分样板代码：
  
```rust
let name = route!(pick_result);
 
// 展开为
let name = match pick_result {
    Ok(r) => r,
    Err(e) => return e,
};
```
 
## 提取值的后处理

  在您使用 `pick` 提取了用户输入后，可以使用 `after` 或 `after_or_route` 立刻处理该参数 ✏️
  
```rust
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    let name = prev
        .pick_or(["--name", "-n"], "World")
        // 在提取出 `--name` 后，立刻对其进行格式化
        .after(|name: String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string()
        })
        .unpack();
 
    ResultGreetSomeone::new(name) // 此处传入的 name 已被格式化处理
}
```
 
  同样，您可以使用 `after_or_route` 来处理输入参数的格式错误 ✏️

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
pack!(ResultGreetSomeone = String);
pack!(ErrorGreetNameTooLong = usize);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> Next {
    let pick_result = prev
        .pick_or(["--name", "-n"], "World")
        // 和 `after` 不同，此处传入的是 &String
        .after_or_route(|name: &String| {
            name.replace(['-', '_', '.'], " ")
                .to_lowercase()
                .trim()
                .to_string();
 
            // 判断名字长度，若过长则路由到错误类型
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
 
## 布尔值解析

  `Picker` 当然也可以解析 **布尔类型**，但是布尔类型分为显式和隐式模式，
  
  |模式|格式|
  |-|-|
  |显式|`--confirm true` 或 `--confirm yes`|
  |隐式|`--confirmed`|

  - 若您使用 `.pick` 解析 `bool` 类型的时候，它将使用隐式解析：只要标志存在则为 `true`
  - 若您使用 `.pick` 解析 `mingling::parser::Yes` 或 `mingling::parser::True` 时，它将使用显式解析，此处必须填写为 `true` / `yes` 时，才能识别为 `true`

  一般来说：使用隐式解析足以，但是在处理位置参数或重要确认行为时，使用显式逻辑可能更符合语义。

```rust
#[chain]
fn handle_some_entry(prev: SomeEntry) -> Next {
    let confirmed: bool = prev.pick::<Yes>(()).unpack().is_yes();
    let confirm: bool = prev.pick::<bool>(["--confirm", "-C"]).unpack();
 
    // 其他逻辑
}
```
 
## 特殊用法：`usize` 解析

  **Mingling** 为 `usize` 提供了一个特殊的用法：解析类似 `25G`、`32mb` 等字样 ✏️
  
```rust
#[test]
fn parse_size() {
    let vec = vec!["--size".to_string(), "25mib".to_string()];
    let size: usize = vec.pick(["--size", "-S"]).unpack();
    assert_eq!(size, 25 * 1024 * 1024);
}
```
 
## 自定义可解析类型

  您可以使用 `Pickable` trait 使您的类型支持被 `Picker` 解析，这也是 `Picker` 拓展性的来源 ✏️
  
```rust
// 必须实现 Default：当解析失败时，内部会直接记录默认值
#[derive(Default)]
pub struct Address {
    ip: String,
    port: u16,
}
 
impl Pickable for Address {
    type Output = Self;
    fn pick(args: &mut Argument, flag: Flag) -> Option<Self::Output> {
        // 直接从 Argument 中使用 Flag 提取原始字符
        let raw = args.pick_argument(flag)?;
 
        // 解析原始字符，转换为结构化数据
        let parts: Vec<&str> = raw.split(':').collect();
        let ip = parts.first()?.to_string();
        let port: u16 = parts.get(1)?.parse().ok()?;
 
        Some(Address { ip, port })
    }
}
```
 
  我们为 `Address` 实现 `Pickable`：接下来我们便可以使用 `ip:port` 的方式来输入参数了 ✏️

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
 
  执行效果如下：
  
```bash
~> your-bin connect --addr 127.0.0.1:8080
Connected: IP: 127.0.0.1 PORT: 8080
```
 
## 自动为枚举实现 Pickable

  要为枚举类型实现 `Pickable` trait，无需手动实现：`Picker` 会为所有实现了 `PickableEnum` 的类型实现 `Pickable`，只需要该枚举类型实现了 `EnumTag` ✏️
  
```rust
// Debug  : 用于渲染
// Default: 用于 Picker 解析
// EnumTag: 用于实现 PickableEnum
#[derive(Debug, Default, EnumTag)]
pub enum Fruits {
    #[default]
    Apple,
    Banana,
    Orange,
}
 
// 为 Fruits 实现 PickableEnum
impl PickableEnum for Fruits {}
```
 
  接下来您便可以直接使用 `Picker` 解析该类型 ✏️
  
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
 
  以上便是 `Picker` 的所有用法，在下一章节，我会介绍如何在 **Mingling** 内为命令实现帮助文档。

<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
