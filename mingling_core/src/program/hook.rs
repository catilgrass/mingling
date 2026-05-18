use std::any::Any;

use crate::{AnyOutput, Program, ProgramCollect, RenderResult, error::ProgramPanic};

#[derive(Default)]
pub struct ProgramHook<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Executes when the program starts running
    pub begin: Option<fn()>,

    /// Executes before the program dispatches
    pub pre_dispatch: Option<fn(args: &Vec<String>)>,

    /// Executes after the program dispatches
    pub post_dispatch: Option<fn(entry: &C)>,

    /// Executes before the type enters the chain
    pub pre_chain: Option<fn(input: &C, raw: &dyn Any)>,

    /// Executes after the chain processing for the type ends
    pub post_chain: Option<fn(output: &AnyOutput<C>)>,

    /// Executes before the type enters the renderer
    pub pre_render: Option<fn(input: &C, raw: &dyn Any)>,

    /// Executes after the type enters the renderer
    pub post_render: Option<fn(result: &RenderResult)>,

    /// Executes before the program ends
    pub finish: Option<fn() -> i32>,

    /// Executes when the program panics
    pub exec_panic: Option<fn(&ProgramPanic)>,

    /// Executes when the REPL starts (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_on_begin: Option<fn()>,

    /// Executes when the REPL receives a render result (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_on_receive_result: Option<fn(&RenderResult)>,

    /// Executes when the REPL panics (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub repl_on_panic: Option<fn(&ProgramPanic)>,
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Adds a typed hook to the program. The hook will be called at the appropriate
    /// lifecycle events.
    pub fn with_hook(&mut self, hook: ProgramHook<C>) {
        self.hooks.push(hook);
    }

    pub(crate) fn run_hook_on_begin(&self) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(begin) = hook.begin {
                begin()
            }
        }
    }

    pub(crate) fn run_hook_pre_dispatch(&self, args: &Vec<String>) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(pre_dispatch) = hook.pre_dispatch {
                pre_dispatch(args)
            }
        }
    }

    pub(crate) fn run_hook_post_dispatch(&self, entry: &C) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(post_dispatch) = hook.post_dispatch {
                post_dispatch(entry)
            }
        }
    }

    pub(crate) fn run_hook_pre_chain(&self, input: &C, raw: &dyn Any) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(pre_chain) = hook.pre_chain {
                pre_chain(input, raw)
            }
        }
    }

    pub(crate) fn run_hook_post_chain(&self, output: &AnyOutput<C>) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(post_chain) = hook.post_chain {
                post_chain(output)
            }
        }
    }

    pub(crate) fn run_hook_pre_render(&self, input: &C, raw: &dyn Any) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(pre_render) = hook.pre_render {
                pre_render(input, raw)
            }
        }
    }

    pub(crate) fn run_hook_post_render(&self, result: &RenderResult) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(post_render) = hook.post_render {
                post_render(result)
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn run_hook_exec_panic(&self, panic_info: &ProgramPanic) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(exec_panic) = hook.exec_panic {
                exec_panic(panic_info)
            }
        }
    }

    pub(crate) fn run_hook_finish(&self) -> i32 {
        if !self.user_context.run_hook {
            return 0;
        }

        let mut exit_code = 0;
        for hook in &self.hooks {
            if let Some(finish) = hook.finish {
                exit_code = finish();
                if exit_code != 0 {
                    return exit_code;
                }
            }
        }
        exit_code
    }

    /// Runs the REPL begin hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_on_begin(&self) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(repl_on_begin) = hook.repl_on_begin {
                repl_on_begin()
            }
        }
    }

    /// Runs the REPL receive result hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_on_receive_result(&self, result: &RenderResult) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(repl_on_receive_result) = hook.repl_on_receive_result {
                repl_on_receive_result(result)
            }
        }
    }

    /// Runs the REPL panic hooks (only available with `repl` feature)
    #[cfg(feature = "repl")]
    pub(crate) fn run_hook_repl_on_panic(&self, panic_info: &ProgramPanic) {
        if !self.user_context.run_hook {
            return;
        }

        for hook in &self.hooks {
            if let Some(repl_on_panic) = hook.repl_on_panic {
                repl_on_panic(panic_info)
            }
        }
    }
}

impl<C> ProgramHook<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Creates a new empty hook set with no handlers.
    pub fn empty() -> Self {
        Self {
            begin: None,
            pre_dispatch: None,
            post_dispatch: None,
            pre_chain: None,
            post_chain: None,
            pre_render: None,
            post_render: None,
            finish: None,
            exec_panic: None,
            #[cfg(feature = "repl")]
            repl_on_begin: None,
            #[cfg(feature = "repl")]
            repl_on_receive_result: None,
            #[cfg(feature = "repl")]
            repl_on_panic: None,
        }
    }

    /// Sets the handler for the `begin` event.
    pub fn on_begin(mut self, handler: fn()) -> Self {
        let _ = self.begin.insert(handler);
        self
    }

    /// Sets the handler for the `pre_dispatch` event.
    pub fn on_pre_dispatch(mut self, handler: fn(args: &Vec<String>)) -> Self {
        let _ = self.pre_dispatch.insert(handler);
        self
    }

    /// Sets the handler for the `post_dispatch` event.
    pub fn on_post_dispatch(mut self, handler: fn(entry: &C)) -> Self {
        let _ = self.post_dispatch.insert(handler);
        self
    }

    /// Sets the handler for the `pre_chain` event.
    pub fn on_pre_chain(mut self, handler: fn(input: &C, raw: &dyn Any)) -> Self {
        let _ = self.pre_chain.insert(handler);
        self
    }

    /// Sets the handler for the `post_chain` event.
    pub fn on_post_chain(mut self, handler: fn(output: &AnyOutput<C>)) -> Self {
        let _ = self.post_chain.insert(handler);
        self
    }

    /// Sets the handler for the `pre_render` event.
    pub fn on_pre_render(mut self, handler: fn(input: &C, raw: &dyn Any)) -> Self {
        let _ = self.pre_render.insert(handler);
        self
    }

    /// Sets the handler for the `post_render` event.
    pub fn on_post_render(mut self, handler: fn(result: &RenderResult)) -> Self {
        let _ = self.post_render.insert(handler);
        self
    }

    /// Sets the handler for the `finish` event.
    pub fn on_finish(mut self, handler: fn() -> i32) -> Self {
        let _ = self.finish.insert(handler);
        self
    }

    /// Sets the handler for the `exec_panic` event.
    pub fn on_exec_panic(mut self, handler: fn(&ProgramPanic)) -> Self {
        let _ = self.exec_panic.insert(handler);
        self
    }

    /// Sets the handler for the REPL begin event (only available with `repl` feature).
    #[cfg(feature = "repl")]
    pub fn on_repl_begin(mut self, handler: fn()) -> Self {
        let _ = self.repl_on_begin.insert(handler);
        self
    }

    /// Sets the handler for the REPL receive result event (only available with `repl` feature).
    #[cfg(feature = "repl")]
    pub fn on_repl_receive_result(mut self, handler: fn(result: &RenderResult)) -> Self {
        let _ = self.repl_on_receive_result.insert(handler);
        self
    }

    /// Sets the handler for the REPL panic event (only available with `repl` feature).
    #[cfg(feature = "repl")]
    pub fn on_repl_panic(mut self, handler: fn(panic: &ProgramPanic)) -> Self {
        let _ = self.repl_on_panic.insert(handler);
        self
    }
}
