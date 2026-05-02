<h1 align="center">实现回退机制</h1>
<p align="center">
    使用回退机制处理程序的错误情况
</p>

## 书接上文

  在上文中，我们介绍了如何使用 **Mingling** 开发基本的命令行程序：你可以使用 `"greet"` 子命令输出 `"Hello, World!"`，也可以使用 `"greet Alice"` 输出 `"Hello, Alice!"`
  
  而当用户并未输入 `"greet"` 时呢？让我们键入命令尝试一下 ⌨️
  
```bash
~> your-bin hello
~> your-bin hello Alice
```
 
  **它没有任何反应！** 👆
  
  让我来解释为什么：**Mingling** 不自作主张，无论发生什么它都不会输出内容到终端（除了 `unwind` 下的 `panic!`）
  
  这意味着，如果您需要在命令行程序出错时能够主动地做些什么，你就得显式声明。
  
  好在 **Mingling** 提供了较为方便的接口实现该功能：在 `gen_program!` 宏中，会生成两个 `FallBack` 类型
  
|类型|发生时机|发生方式|
|-|-|-|
|RendererNotFound|调度无法找到的渲染器时|作为 `Chain` 调度|
|DispatcherNotFound|输入命令但无法匹配分发器|作为 `Chain` 调度|
  
### `DispatcherNotFound` 类型
  
  首先让我们关注 `DispatcherNotFound` 类型，它的产生方式如下：
  
```rust
// 1. 定义 `greet` 命令
dispatcher!("greet", GreetCommand => GreetEntry);
 
fn main() {
    // ->> 用户输入 "hello Alice"
    let mut program = ThisProgram::new();
 
    // 2. 导入 `greet` 命令
    program.with_dispatcher(GreetCommand);
 
    // 3. 执行程序
    program.exec();
}
 
// ... 
 
// 5. 接收 DispatcherNotFound 调度
#[renderer]
fn dispatcher_not_found(prev: DispatcherNotFound) {
    // 6. 输出
    r_println!(
        "Cannot match any command! Current input: \"{}\"",
        prev.join(" ")
    );
}
 
// 4. 无法匹配到任何名为 `hello` 的分发器
//    将用户参数原样分发到 DispatcherNotFound
gen_program!(); 
```
  
  上述程序的运行效果为：
  
```bash
~> omg hello
Cannot match any command! Current input: "hello"
 
~> omg hello Alice
Cannot match any command! Current input: "hello Alice"
```
 
  现在若用户输入了不匹配的命令，**Mingling** 将会输出对应的内容！
  
## `RendererNotFound` 类型

  `RendererNotFound` 有两种可能产生：
  
  1. 该类型被显式分发到了 `Renderer` (使用 `.to_render()` 函数)，但是该类型未实现渲染器
  2. 该类型被分发到了 `Chain`，但是该类型未实现链，也未实现渲染器
  
  一般来说，`RendererNotFound` **不应该在业务逻辑中产生**：它被调度意味着您的类型需要被渲染但是并不能被渲染。您可以使用该类型来定位哪个类型缺失渲染器实现 ✏️
  
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
 
// 让我们故意去除 `ResultGreetSomeone` 类型的渲染器实现
// #[renderer]
// fn render_greet_someone(prev: ResultGreetSomeone) {
//     r_println!("Hello, {}!", *prev);
// }
 
#[renderer]
fn renderer_not_found(prev: RendererNotFound) {
    if *prev == "DispatcherNotFound" {
        return; // 排除 "DispatcherNotFound" 类型
    }
    
    // 当未找到渲染器时触发 `panic!`
    panic!("Renderer \"{}\" not found!", *prev);
}
 
gen_program!();
 
```
 
  上述程序的运行效果为：
  
```bash
~> your-bin greet Alice
 
thread 'main' (90772) panicked at src/bin/your-bin.rs:30:5:
Renderer "ResultGreetSomeone" not found!
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```
  
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
