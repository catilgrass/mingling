use mingling_core::{Program, ProgramCollect, setup::ProgramSetup};

/// Performs basic program initialization:
///
/// - Collects `--quiet` flag to control message rendering
/// - Collects `--help` flag to enable help mode
/// - Collects `--confirm` flag to skip user confirmation
pub struct BasicProgramSetup;

impl<C> ProgramSetup<C> for BasicProgramSetup
where
    C: ProgramCollect<Enum = C>,
{
    fn setup(&mut self, program: &mut Program<C>) {
        program.global_flag(["--quiet", "-q"], |p| {
            p.stdout_setting.render_output = false;
            p.stdout_setting.error_output = false;
        });

        program.global_flag(["--help", "-h"], |p| {
            p.user_context.help = true;
        });

        program.global_flag(["--confirm", "-C"], |p| {
            p.user_context.confirm = true;
        });
    }
}
