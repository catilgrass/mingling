use std::sync::atomic::{AtomicI32, Ordering};

use crate::{ProgramCollect, hook::ProgramAnonymousHook, setup::ProgramSetup};

static EXIT_CODE: AtomicI32 = AtomicI32::new(0);

/// Provides the ability to control the program's exit code, which is returned when the program ends.
///
/// - Use `mingling::update_exit_code` to update the exit code.
/// - Use `mingling::current_exit_code` to query the current exit code.
pub struct ExitCodeSetup;

impl<C> ProgramSetup<C> for ExitCodeSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(&mut self, program: &mut crate::Program<C>) {
        program
            .with_hook_anonymous(ProgramAnonymousHook::empty().on_finish(current_exit_code));
    }
}

/// Updates the program's exit code.
///
/// This function sets the value that will be returned when the program exits.
/// The new code will take effect immediately and be used when the program finishes.
pub fn update_exit_code(code: i32) {
    EXIT_CODE.store(code, Ordering::SeqCst);
}

/// Returns the current exit code.
///
/// This function queries the value that will be returned when the program exits.
pub fn current_exit_code() -> i32 {
    EXIT_CODE.load(Ordering::SeqCst)
}
