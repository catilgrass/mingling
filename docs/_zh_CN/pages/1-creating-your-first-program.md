<h1 align="center">创建您的第一个程序</h1>
<p align="center">
    了解 <b>Mingling</b>，并使用它创建您的第一个命令行程序
</p>

## 前言

  本章节将介绍如何渐进式地了解 **Mingling**

  在开始之前，我先来讲讲 **Mingling** 能做什么：

  在未开启其他特性时，它本身是一个基于 `proc-macro` 的子命令调度系统：它匹配用户输入的文本，以此查找并创建具体的数据，并将该数据放入调度器中不断转换类型，当该数据被转换到无法转换时，程序会将最终的数据渲染到终端上。

  也就是说，您需要理解一套新的开发范式：**完全基于类型的调度系统**。这可能会让您前期的学习**充满挫败感**，但当您逐渐理解这套范式后，您将可以写出极其方便修改和拓展的命令行程序。



## 创建基本程序

  接下来我将会讲述如何创建一个基本的程序，相信您已经准备好了一个空的 Rust 项目！

#### 1. 添加依赖

  在 `Cargo.toml` 中添加如下依赖 ✏️

```toml
[dependencies]
mingling = "0.1.7"
 
# 如果您要尝鲜，可以试试 Github 上托管的版本
mingling = { git = "https://github.com/catilgrass/mingling", branch = "main" }
```
 
> [!NOTE]
>
> 该版本基于文档编写时的 **Mingling** 版本，您可以前往 [crates.io](https://crates.io/crates/mingling) 查看最新的版本！😄 
>
> **Mingling** 会积极更新文档，以确保文档内容能紧跟最新版本



#### 2. 创建程序

  接下来，在 `src/main.rs` 中创建程序 ✏️

```rust
fn main() {
    // 创建 ThisProgram，并执行
    ThisProgram::new().exec();
}
 
// gen_program! 宏将会收集 *它之前* 的所有组件、类型
// 然后生成程序 `ThisProgram`
mingling::macros::gen_program!();
```
 
> [!TIP]
>
>   `gen_Sprogram!()` 宏展开时，会收集在它之前展开的其他组件、类型的信息，这意味着您需要将 `gen_program!()` 放在整个 crate 中最后被展开的位置
>
> 我推荐放在 `main.rs` 或者 `lib.rs` 的结尾。



#### 3. 创建命令

  当然，现在的程序什么都没有，在运行时不会输出任何消息。所以，让我们创建第一条命令 `greet`，给谁打个招呼吧 ✏️

```rust
fn main() {
    // ...
}
 
// 创建分发器，并将 GreetCommand 绑定在 "greet" 子命令
// 在用户指定该命令时，向调度器发送 GreetEntry
dispatcher!("greet", GreetCommand => GreetEntry);
 
// ...
gen_program!();
```
 
  不要被突然多出来的一个宏和两个类型所吓到！我来逐一解释这个宏干了什么：

##### 关于 `dispatcher!` 宏 💡

1. 宏创建了一个` GreetCommand` 结构体，并实现了 `Dispatcher` trait

​    *这一步告诉框架：现在有了个新的分发器，它将会承接一个子命令的行为。*

2. 宏实现了 `Dispatcher` trait 内部的 `node(&self) -> Node` 函数，并告诉节点为 `"greet"`

​    *这一步告诉框架：该分发器将承接子命令 `"greet"` 的行为*

3. 宏实现了 `Dispatcher` trait 内部的 `begin` 函数，将用户输入的完整参数转换为了第一个类型 `GreetEntry`

​    *这一步告诉框架：该分发器在被匹配到后，将会向调度器发送类型 `GreetEntry`，供后续执行*

  简而言之：**“用户输入 `greet`，我就创建 `GreetEntry`，丢给调度器转换”**



#### 4. 注册命令

  在 `Dispatcher` 创建后，我们得到了两个类型 `GreetCommand` 和 `GreetEntry`，首先将 `GreetCommand` 注册到 `ThisProgram` ✏️

```rust
fn main() {
    let mut program = ThisProgram::new();
    
    // 注册分发器
    program.with_dispatcher(GreetCommand);
    program.exec();
}
```
 
  这样，`ThisProgram` 就认得 `"greet"` 子命令了，但是框架还不知道 `"greet"` 的行为是怎样的。此时我们便需要实现具体的逻辑：



#### 5. 实现渲染行为

  我们期望 `"greet"` 的时候输出 `"Hello, World"`：既然要输出到终端，那么我们可以使用 **Mingling** 的另一个组件 `Renderer`，它负责将数据渲染到终端 ✏️

```rust
// ...
dispatcher!("greet", GreetCommand => GreetEntry);
 
// 声明渲染器 `render_greet`，并表示前一个类型是 `GreetEntry`
#[renderer]
fn render_greet(_prev: GreetEntry) {
    r_println!("Hello, World!");
}
 
// ...
gen_program!(); // 渲染器会被注册到程序
```
 
  对于 `#[renderer]` 属性宏标记的函数，**Mingling** 严格规定只允许使用一种函数签名：

```rust
#[renderer]
fn renderer_name (_prev: PreviousType) {  }
```
 
  宏会读取到第一个参数的类型，并告诉 `gen_program!` 该函数用来渲染该类型。

##### 关于 `r_println!()` 💡

  您可能会注意到，在 `#[renderer]` 中使用的打印宏是 `r_println!` 而非 `println!`，这是因为框架的渲染逻辑并不在该函数内：在 `#[renderer]` 展开后，会向函数注入一个 `r: &mut RenderResult`；而 `r_println!` 将信息追加到 `RenderResult` 内，并在调度器关闭后，将最终的渲染数据交给 `Program::exec` 函数输出。



#### 6. 增加执行逻辑

  我猜您已经很想实现 `greet Alice` 这样的语法来输出 `"Hello, Alice!"` 了，本段正准备干这件事！

  **Mingling** 的核心执行流程是 `Dispatcher -> Chain -> Renderer`，而最关键的就是 `Chain`：它负责将输入的数据类型转换为其他类型，然后让调度器根据结果的类型找到下一个 `Chain` 或者 `Renderer ✏️

```rust
dispatcher!("greet", GreetCommand => GreetEntry);
 
// 包装中间类型 `ResultGreetSomeone`
pack!(ResultGreetSomeone = String);
 
#[chain]
fn handle_greet_entry(prev: GreetEntry) -> NextProcess {
    let args = prev.inner;
    let name = args
    	.first()
    	.cloned()
    	.unwrap_or_else(|| "World".to_string());
 
    // 包装为中间类型
    ResultGreetSomeone::new(name)
}
 
#[renderer]
fn render_greet_someone(prev: ResultGreetSomeone) {
    // 解引用 prev 拿到原始类型
    r_println!("Hello, {}!", *prev); 
}
```
 
  像 `#[renderer]` 一样，我们创建了一个 `#[chain]`，它处理类型 `GreetEntr`，输出 `ResultGreetSomeone`

  这样我们就在原本的 `Dispatcher` 和 `Renderer` 中间插入了一个 `Chain`：它可以将用户输入的参数提取出来（或回退到默认值 "World"），再交由渲染器打印到终端。

##### 关于 `NextProcess` 💡

  `NextProcess` 是由 `gen_program!()` 生成的占位符，在 `#[chain]` 展开后，它将被替换为调度器能识别的类型擦除类型 `ChainProcess<ThisProgram>`，用于减少代码量

> [!NOTE]
>
> `NextProcess` 方案为临时替代，下一次更新需要等待 Rust 的 `Impl In Type Aliases` 特性稳定后。
>
> **不过，您不用担心**：下一次 `NextProcess` 的更新不会引入 **破坏性变更！**

##### 关于 `pack!` 💡

  `pack!` 是 **Mingling** 开发过程中使用频率 **极高** 的宏：它负责将任意类型包装成另一个类型，并自动为其派生框架所需的特征。

  它的语法如您所见，极为简单：

```rust
pack!(PackedType = RawType);
```
 
  不过请注意：`pack!` 宏不支持带有生命周期的类型包装，因为类型在调度器之间的流转方式永远都是 `move` 而非 `borrow`。



#### 7. 编译并运行

  好的，至此我们完成了一个基本的命令行程序，以下是完整代码，您可以直接粘贴运行：

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
 
  运行结果：

```bash
~> your-bin greet
Hello, World!
~> your-bin greet Alice
Hello, Alice!
```
 
<p align="center" style="font-size: 0.85em; color: gray;">
    Written by @Weicao-CatilGrass
</p>
