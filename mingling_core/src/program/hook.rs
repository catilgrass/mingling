use std::{any::Any, fmt::Display};

use crate::{AnyOutput, Program, ProgramCollect, RenderResult};

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
}

#[derive(Default)]
pub struct ProgramAnonymousHook {
    /// Executes when the program starts running
    pub begin: Option<fn()>,

    /// Executes before the program dispatches
    pub pre_dispatch: Option<fn(args: &Vec<String>)>,

    /// Executes after the program dispatches
    pub post_dispatch: Option<fn(entry_name: &str)>,

    /// Executes before the type enters the chain
    pub pre_chain: Option<fn(input_name: &str, raw: &dyn Any)>,

    /// Executes after the chain processing for the type ends
    pub post_chain: Option<fn(output_name: &str)>,

    /// Executes before the type enters the renderer
    pub pre_render: Option<fn(input_name: &str, raw: &dyn Any)>,

    /// Executes after the type enters the renderer
    pub post_render: Option<fn(result: &RenderResult)>,

    /// Executes before the program ends
    pub finish: Option<fn() -> i32>,
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Adds an anonymous hook to the program. The hook will be called at the appropriate
    /// lifecycle events, but receives string representations instead of typed references.
    pub fn with_hook_anonymous(&mut self, hook: ProgramAnonymousHook) {
        self.anonymous_hooks.push(hook);
    }
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C> + Display,
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(begin) = anonymous_hook.begin {
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(pre_dispatch) = anonymous_hook.pre_dispatch {
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(post_dispatch) = anonymous_hook.post_dispatch {
                post_dispatch(entry.to_string().as_str())
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(pre_chain) = anonymous_hook.pre_chain {
                pre_chain(input.to_string().as_str(), raw)
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(post_chain) = anonymous_hook.post_chain {
                post_chain(output.member_id.to_string().as_str())
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(pre_render) = anonymous_hook.pre_render {
                pre_render(input.to_string().as_str(), raw)
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(post_render) = anonymous_hook.post_render {
                post_render(result)
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
        for anonymous_hook in &self.anonymous_hooks {
            if let Some(finish) = anonymous_hook.finish {
                finish();
            }
        }
        exit_code
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
}

impl ProgramAnonymousHook {
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
    pub fn on_post_dispatch(mut self, handler: fn(entry_name: &str)) -> Self {
        let _ = self.post_dispatch.insert(handler);
        self
    }

    /// Sets the handler for the `pre_chain` event.
    pub fn on_pre_chain(mut self, handler: fn(input_name: &str, raw: &dyn Any)) -> Self {
        let _ = self.pre_chain.insert(handler);
        self
    }

    /// Sets the handler for the `post_chain` event.
    pub fn on_post_chain(mut self, handler: fn(output_name: &str)) -> Self {
        let _ = self.post_chain.insert(handler);
        self
    }

    /// Sets the handler for the `pre_render` event.
    pub fn on_pre_render(mut self, handler: fn(input_name: &str, raw: &dyn Any)) -> Self {
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
}
