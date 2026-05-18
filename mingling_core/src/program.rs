use crate::error::ProgramPanic;

#[cfg(feature = "comp")]
use crate::{ShellContext, Suggest};

#[cfg(feature = "general_renderer")]
use crate::error::GeneralRendererSerializeError;

#[cfg(not(windows))]
use std::env;

use crate::{
    AnyOutput, ChainProcess, GlobalResources, Groupped, RenderResult,
    asset::dispatcher::Dispatcher,
    error::{ChainProcessError, ProgramExecuteError},
    hook::ProgramHook,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

#[cfg(feature = "async")]
use std::pin::Pin;

#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod exec;
#[doc(hidden)]
pub mod hook;
#[doc(hidden)]
pub mod setup;

mod config;
pub use config::*;

mod flag;
pub use flag::*;

mod string_vec;
pub use string_vec::*;

/// Global static reference to the current program instance
static THIS_PROGRAM: OnceLock<Option<Box<dyn std::any::Any + Send + Sync>>> = OnceLock::new();

/// Returns a reference to the current program instance, panics if not set.
pub fn this<C>() -> &'static Program<C>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    try_get_this_program().expect("Program not initialized")
}

/// Returns a reference to the current program instance, if set.
fn try_get_this_program<C>() -> Option<&'static Program<C>>
where
    C: ProgramCollect<Enum = C> + 'static,
{
    THIS_PROGRAM.get()?.as_ref()?.downcast_ref::<Program<C>>()
}

/// Program, used to define the behavior of the entire command-line program
#[derive(Default)]
pub struct Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    pub(crate) collect: std::marker::PhantomData<C>,

    pub(crate) args: Vec<String>,

    #[cfg(not(feature = "dispatch_tree"))]
    pub(crate) dispatcher: Vec<Box<dyn Dispatcher<C> + Send + Sync>>,

    pub stdout_setting: ProgramStdoutSetting,
    pub user_context: ProgramUserContext,

    #[cfg(feature = "general_renderer")]
    pub general_renderer_name: GeneralRendererSetting,

    pub(crate) hooks: Vec<ProgramHook<C>>,

    pub(crate) resources: GlobalResources,
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Creates a new Program instance, initializing command-line arguments from the environment.
    pub fn new() -> Self {
        #[cfg(not(windows))]
        return Self::new_with_args(env::args().collect::<Vec<String>>());

        #[cfg(windows)]
        return Self::new_with_args({
            std::env::args_os()
                .map(|arg| {
                    use std::os::windows::ffi::OsStrExt;

                    let wide: Vec<u16> = arg.encode_wide().collect();
                    String::from_utf16_lossy(&wide)
                })
                .collect::<Vec<_>>()
        });
    }

    /// Creates a new Program instance with the provided command-line arguments.
    pub fn new_with_args(args: impl Into<StringVec>) -> Self {
        Program {
            collect: std::marker::PhantomData,
            args: args.into().into(),

            #[cfg(not(feature = "dispatch_tree"))]
            dispatcher: Vec::new(),

            stdout_setting: Default::default(),
            user_context: Default::default(),

            #[cfg(feature = "general_renderer")]
            general_renderer_name: GeneralRendererSetting::Disable,

            hooks: Vec::new(),

            resources: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Returns a reference to the current program instance, if set.
    pub fn this_program() -> &'static Program<C>
    where
        C: 'static,
    {
        THIS_PROGRAM
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
            .downcast_ref::<Program<C>>()
            .unwrap()
    }

    /// Get all registered dispatcher names from the program
    pub fn get_nodes(
        &'static self,
    ) -> Vec<(String, &'static (dyn Dispatcher<C> + Send + Sync + 'static))> {
        get_nodes(self)
    }

    /// Dynamically dispatch input arguments to registered entry types
    pub fn dispatch_args_dynamic(
        &'static self,
        args: impl Into<StringVec>,
    ) -> Result<AnyOutput<C>, ChainProcessError> {
        match exec::dispatch_args_dynamic(self, &args.into().into()) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into()),
        }
    }

    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    #[cfg(feature = "dispatch_tree")]
    pub fn dispatch_args_trie(
        &'static self,
        args: impl Into<StringVec>,
    ) -> Result<AnyOutput<C>, ChainProcessError> {
        let string_vec: Vec<String> = args.into().into();
        match C::dispatch_args_trie(&string_vec) {
            Ok(ok) => Ok(ok),
            Err(e) => Err(e.into()),
        }
    }
}

// Async program
#[cfg(feature = "async")]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    async fn exec_wrapper<F, Fut>(self, f: F) -> Result<Fut::Output, ProgramPanic>
    where
        C: 'static + Send + Sync,
        F: FnOnce(&'static Program<C>) -> Fut + Send + Sync,
        Fut: Future + Send,
    {
        THIS_PROGRAM.get_or_init(|| Some(Box::new(self)));
        let program = THIS_PROGRAM
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
            .downcast_ref::<Program<C>>()
            .unwrap();

        #[cfg(not(panic = "abort"))]
        if program.stdout_setting.silence_panic {
            std::panic::set_hook(Box::new(|_| {}));
        }

        #[cfg(panic = "abort")]
        return Ok(f(program));

        #[cfg(not(panic = "abort"))]
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(program))) {
            Ok(fut) => Ok(fut.await),
            Err(panic_info) => {
                let panic_payload = ProgramPanic {
                    payload: panic_info,
                };
                program.run_hook_exec_panic(&panic_payload);
                Err(panic_payload)
            }
        }
    }

    /// Run the command line program
    pub async fn exec_without_render(mut self) -> Result<RenderResult, ProgramExecuteError>
    where
        C: 'static + Send + Sync,
    {
        // Run hooks
        self.run_hook_on_begin();

        self.args = self.args.iter().skip(1).cloned().collect();
        match self
            .exec_wrapper(|p| async { crate::exec::exec(p).await.map_err(|e| e.into()) })
            .await
        {
            Ok(r) => r,
            Err(e) => Err(ProgramExecuteError::Panic(e)),
        }
    }

    /// Run the command line program
    pub async fn exec(self) -> i32
    where
        C: 'static + Send + Sync,
    {
        let stdout_setting = self.stdout_setting.clone();
        let result = match self.exec_without_render().await {
            Ok(r) => r,
            Err(e) => match e {
                ProgramExecuteError::DispatcherNotFound => {
                    eprintln!("Dispatcher not found");
                    return 1;
                }
                ProgramExecuteError::RendererNotFound(renderer_name) => {
                    eprintln!("Renderer `{}` not found", renderer_name);
                    return 1;
                }
                ProgramExecuteError::Other(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
                ProgramExecuteError::Panic(unwinded_error) => {
                    eprintln!("{}", unwinded_error);
                    return 1;
                }
            },
        };

        // Render result
        if stdout_setting.render_output && !result.is_empty() {
            let exit_code = result.exit_code;
            print!("{}", result);

            if let Err(e) = std::io::Write::flush(&mut std::io::stdout())
                && stdout_setting.error_output
            {
                eprintln!("{}", e);
                1
            } else {
                exit_code
            }
        } else {
            0
        }
    }

    /// Run the command line program, then exit
    pub async fn exec_and_exit(self)
    where
        C: 'static + Send + Sync,
    {
        std::process::exit(self.exec().await)
    }
}

// Sync program
#[cfg(not(feature = "async"))]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn exec_wrapper<F, R>(self, f: F) -> Result<R, ProgramPanic>
    where
        C: 'static + Send + Sync,
        F: FnOnce(&'static Program<C>) -> R + Send + Sync,
    {
        THIS_PROGRAM.get_or_init(|| Some(Box::new(self)));
        let program = THIS_PROGRAM
            .get()
            .unwrap()
            .as_ref()
            .unwrap()
            .downcast_ref::<Program<C>>()
            .unwrap();

        #[cfg(not(panic = "abort"))]
        if program.stdout_setting.silence_panic {
            std::panic::set_hook(Box::new(|_| {}));
        }

        #[cfg(panic = "abort")]
        return Ok(f(program));

        #[cfg(not(panic = "abort"))]
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(program))) {
            Ok(result) => Ok(result),
            Err(panic_info) => {
                let panic_payload = ProgramPanic {
                    payload: panic_info,
                };
                program.run_hook_exec_panic(&panic_payload);
                Err(panic_payload)
            }
        }
    }

    /// Run the command line program
    pub fn exec_without_render(mut self) -> Result<RenderResult, ProgramExecuteError>
    where
        C: 'static + Send + Sync,
    {
        // Run hooks
        self.run_hook_on_begin();

        self.args = self.args.iter().skip(1).cloned().collect();
        match self.exec_wrapper(|p| crate::exec::exec(p).map_err(|e| e.into())) {
            Ok(r) => r,
            Err(e) => Err(ProgramExecuteError::Panic(e)),
        }
    }

    /// Run the command line program
    pub fn exec(self) -> i32
    where
        C: 'static + Send + Sync,
    {
        let stdout_setting = self.stdout_setting.clone();
        let result = match self.exec_without_render() {
            Ok(r) => r,
            Err(e) => match e {
                ProgramExecuteError::DispatcherNotFound => {
                    eprintln!("Dispatcher not found");
                    return 1;
                }
                ProgramExecuteError::RendererNotFound(renderer_name) => {
                    eprintln!("Renderer `{}` not found", renderer_name);
                    return 1;
                }
                ProgramExecuteError::Other(e) => {
                    eprintln!("{}", e);
                    return 1;
                }
                ProgramExecuteError::Panic(unwinded_error) => {
                    eprintln!("{}", unwinded_error);
                    return 1;
                }
            },
        };

        // Render result
        if stdout_setting.render_output && !result.is_empty() {
            let exit_code = result.exit_code;
            print!("{}", result);

            if let Err(e) = std::io::Write::flush(&mut std::io::stdout())
                && stdout_setting.error_output
            {
                eprintln!("{}", e);
                1
            } else {
                exit_code
            }
        } else {
            0
        }
    }

    /// Run the command line program, then exit
    pub fn exec_and_exit(self)
    where
        C: 'static + Send + Sync,
    {
        std::process::exit(self.exec())
    }
}

/// Collected program context
///
/// Note: It is recommended to use the `gen_program!()` macro from [mingling_macros](https://crates.io/crates/mingling_macros) to automatically create this type
pub trait ProgramCollect {
    /// Enum type representing internal IDs for the program
    type Enum;
    type DispatcherNotFound: Groupped<Self::Enum>;
    type RendererNotFound: Groupped<Self::Enum>;
    type EmptyResult: Groupped<Self::Enum>;

    /// Use a prefix tree to quickly match arguments and dispatch to an Entry
    #[cfg(feature = "dispatch_tree")]
    fn dispatch_args_trie(
        raw: &Vec<String>,
    ) -> Result<AnyOutput<Self::Enum>, crate::error::ProgramInternalExecuteError>;

    /// Get all registered dispatcher names from the program
    #[cfg(feature = "dispatch_tree")]
    fn get_nodes() -> Vec<(String, &'static (dyn Dispatcher<Self::Enum> + Send + Sync))>;

    /// Build an [AnyOutput](./struct.AnyOutput.html) to indicate that a renderer was not found
    fn build_renderer_not_found(member_id: Self::Enum) -> AnyOutput<Self::Enum>;

    /// Build an [AnyOutput](./struct.AnyOutput.html) to indicate that a dispatcher was not found
    fn build_dispatcher_not_found(args: Vec<String>) -> AnyOutput<Self::Enum>;

    /// Build an [AnyOutput](./struct.AnyOutput.html) to indicate that the chain returned an empty result
    fn build_empty_result() -> AnyOutput<Self::Enum>;

    /// Render the input [AnyOutput](./struct.AnyOutput.html)
    fn render(any: AnyOutput<Self::Enum>, r: &mut RenderResult);

    /// Render help for Entry
    fn render_help(any: AnyOutput<Self::Enum>, r: &mut RenderResult);

    /// Find a matching chain to continue execution based on the input [AnyOutput](./struct.AnyOutput.html), returning a new [AnyOutput](./struct.AnyOutput.html)
    #[cfg(feature = "async")]
    fn do_chain(
        any: AnyOutput<Self::Enum>,
    ) -> Pin<Box<dyn Future<Output = ChainProcess<Self::Enum>> + Send>>;

    /// Find a matching chain to continue execution based on the input [AnyOutput](./struct.AnyOutput.html), returning a new [AnyOutput](./struct.AnyOutput.html)
    #[cfg(not(feature = "async"))]
    fn do_chain(any: AnyOutput<Self::Enum>) -> ChainProcess<Self::Enum>;

    /// Match and execute specific completion logic based on any Entry
    #[cfg(feature = "comp")]
    fn do_comp(any: &AnyOutput<Self::Enum>, ctx: &ShellContext) -> Suggest;

    /// Whether the program has a renderer that can handle the current [AnyOutput](./struct.AnyOutput.html)
    fn has_renderer(any: &AnyOutput<Self::Enum>) -> bool;

    /// Whether the program has a chain that can handle the current [AnyOutput](./struct.AnyOutput.html)
    fn has_chain(any: &AnyOutput<Self::Enum>) -> bool;

    /// Perform general rendering and presentation of any type
    #[cfg(feature = "general_renderer")]
    fn general_render(
        any: AnyOutput<Self::Enum>,
        setting: &GeneralRendererSetting,
    ) -> Result<RenderResult, GeneralRendererSerializeError>;
}

#[macro_export]
#[doc(hidden)]
macro_rules! __dispatch_program_renderers {
    (
        $( $render_ty:ty => $prev_ty:ident, )*
    ) => {
        fn render(any: mingling::AnyOutput<Self::Enum>, r: &mut mingling::RenderResult) {
            match any.member_id {
                $(
                    Self::$prev_ty => {
                        // SAFETY: The `type_id` check ensures that `any` contains a value of type `$prev_ty`,
                        // so downcasting to `$prev_ty` is safe.
                        let value = unsafe { any.downcast::<$prev_ty>().unwrap_unchecked() };
                        <$render_ty as mingling::Renderer>::render(value, r);
                    }
                )*
                _ => (),
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(feature = "async")]
macro_rules! __dispatch_program_chains {
    (
        $( $chain_ty:ty => $chain_prev:ident, )*
    ) => {
        fn do_chain(
            any: mingling::AnyOutput<Self::Enum>,
        ) -> std::pin::Pin<Box<dyn Future<Output = mingling::ChainProcess<Self::Enum>> + Send>> {
            match any.member_id {
                $(
                    Self::$chain_prev => {
                        // SAFETY: The `type_id` check ensures that `any` contains a value of type `$chain_prev`,
                        // so downcasting to `$chain_prev` is safe.
                        let value = unsafe { any.downcast::<$chain_prev>().unwrap_unchecked() };
                        let fut = async { <$chain_ty as mingling::Chain<Self::Enum>>::proc(value).await };
                        Box::pin(fut)
                    }
                )*
                _ => panic!("No chain found for type id: {:?}", any.type_id),
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
#[cfg(not(feature = "async"))]
macro_rules! __dispatch_program_chains {
    (
        $( $chain_ty:ty => $chain_prev:ident, )*
    ) => {
        fn do_chain(
            any: mingling::AnyOutput<Self::Enum>,
        ) -> mingling::ChainProcess<Self::Enum> {
            match any.member_id {
                $(
                    Self::$chain_prev => {
                        // SAFETY: The `type_id` check ensures that `any` contains a value of type `$chain_prev`,
                        // so downcasting to `$chain_prev` is safe.
                        let value = unsafe { any.downcast::<$chain_prev>().unwrap_unchecked() };
                        <$chain_ty as mingling::Chain<Self::Enum>>::proc(value)
                    }
                )*
                _ => panic!("No chain found for type id: {:?}", any.type_id),
            }
        }
    };
}

/// Get all registered dispatcher names from the program
#[allow(unused_variables)]
pub fn get_nodes<C: ProgramCollect<Enum = C>>(
    program: &'static Program<C>,
) -> Vec<(String, &'static (dyn Dispatcher<C> + Send + Sync + 'static))> {
    #[cfg(feature = "dispatch_tree")]
    let r = C::get_nodes();

    #[cfg(feature = "dispatch_tree")]
    {
        #[cfg(feature = "debug")]
        {
            let node_strs: Vec<String> = r.iter().map(|v| v.0.clone()).collect();
            crate::info!("All Nodes: [{}]", node_strs.join(", "));
        }
    }

    #[cfg(not(feature = "dispatch_tree"))]
    let r: Vec<_> = program
        .dispatcher
        .iter()
        .map(|disp| {
            let node_str = disp
                .node()
                .to_string()
                .split('.')
                .collect::<Vec<_>>()
                .join(" ");
            (node_str, &**disp)
        })
        .collect();

    #[cfg(not(feature = "dispatch_tree"))]
    {
        #[cfg(feature = "debug")]
        {
            let node_strs: Vec<String> = r.iter().map(|v| v.0.clone()).collect();
            crate::info!("All Nodes: [{}]", node_strs.join(", "));
        }
    }

    r
}
