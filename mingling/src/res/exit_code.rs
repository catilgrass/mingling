use mingling_core::{ProgramCollect, this};

/// Represents a program exit code.
#[derive(Debug, Default, Clone, Copy)]
pub struct ExitCode {
    /// The numeric exit code value.
    pub exit_code: i32,
}

/// Updates the globally stored exit code for the given `ProgramCollect` type.
pub fn update_exit_code<C>(exit_code: i32)
where
    C: ProgramCollect<Enum = C> + 'static,
{
    this::<C>().modify_res(|e: &mut ExitCode| e.exit_code = exit_code);
}

/// Retrieves the globally stored exit code for the given `ProgramCollect` type.
/// Returns `0` if no exit code has been set.
pub fn exit_code<C>() -> i32
where
    C: ProgramCollect<Enum = C> + 'static,
{
    match this::<C>().res::<ExitCode>() {
        Some(e) => e.exit_code,
        None => 0,
    }
}
