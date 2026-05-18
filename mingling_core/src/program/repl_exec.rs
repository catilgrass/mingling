use std::io::Write;

mod splitter;

use crate::error::{ProgramInternalExecuteError, ProgramPanic};
use crate::program::repl_exec::splitter::split_input_string;
use crate::{Program, ProgramCollect, RenderResult};

#[cfg(not(feature = "async"))]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    pub fn exec_repl(self) {
        self.run_hook_repl_on_begin();

        self.exec_wrapper(|p| -> ! {
            loop {
                let args = split_input_string(readline_or_empty());
                match exec_once(p, args) {
                    Ok(r) => {
                        p.run_hook_repl_on_receive_result(&r);
                    }
                    Err(ProgramInternalExecuteError::REPLPanic(panic)) => {
                        p.run_hook_repl_on_panic(&panic);
                    }
                    _ => {}
                }
            }
        });
    }
}

#[cfg(feature = "async")]
impl<C> Program<C>
where
    C: ProgramCollect<Enum = C> + Send + Sync,
{
    pub async fn exec_repl(self) {
        self.run_hook_repl_on_begin();

        self.exec_wrapper(|p| -> ! {
            loop {
                let args = split_input_string(readline_or_empty());
                match exec_once(p, args) {
                    Ok(r) => {
                        p.run_hook_repl_on_receive_result(&r);
                    }
                    Err(ProgramInternalExecuteError::REPLPanic(panic)) => {
                        p.run_hook_repl_on_panic(&panic);
                    }
                    _ => {}
                }
            }
        })
        .await;
    }
}

fn readline() -> Result<String, std::io::Error> {
    let mut input = String::new();
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn readline_or_empty() -> String {
    readline().unwrap_or("".to_string())
}

fn exec_once<C>(
    p: &'static Program<C>,
    args: Vec<String>,
) -> Result<RenderResult, ProgramInternalExecuteError>
where
    C: ProgramCollect<Enum = C> + Send + Sync + 'static,
{
    #[cfg(panic = "abort")]
    let exec_result = super::exec::exec_with_args(p, args);

    #[cfg(not(panic = "abort"))]
    let exec_result = {
        let exec_unwind_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            super::exec::exec_with_args(p, args)
        }));

        match exec_unwind_result {
            Err(panic_info) => {
                let panic_payload = ProgramPanic {
                    payload: panic_info,
                };
                let program = crate::program::THIS_PROGRAM
                    .get()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .downcast_ref::<Program<C>>()
                    .unwrap();
                program.run_hook_repl_on_panic(&panic_payload);
                Err(ProgramInternalExecuteError::REPLPanic(panic_payload))
            }
            Ok(r) => r,
        }
    };

    exec_result
}
