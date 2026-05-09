use crate::{ProgramCollect, program::Program};

mod basic;
pub use basic::*;

#[doc(hidden)]
pub mod exit_code_control;
pub use exit_code_control::ExitCodeSetup;

#[cfg(feature = "general_renderer")]
mod general_renderer;

#[cfg(feature = "general_renderer")]
pub use general_renderer::*;

pub trait ProgramSetup<C>
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(&mut self, program: &mut Program<C>);
}

impl<C> Program<C>
where
    C: ProgramCollect<Enum = C>,
{
    /// Load and execute init logic
    pub fn with_setup<S: ProgramSetup<C> + 'static>(&mut self, mut setup: S) -> S {
        S::setup(&mut setup, self);
        setup
    }
}
