use crate::{ProgramCollect, program::Program};

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
