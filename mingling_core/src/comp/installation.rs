use crate::{build::build_comp_script_to_file, ShellFlag};

pub fn install_comp_script(
    flag: ShellFlag,
    bin_name: impl AsRef<str>,
) -> Result<(), std::io::Error> {
    match flag {
        // ~/.local/share/bash-completion/completions/
        ShellFlag::Bash => {
            let Some(data_dir) = dirs::data_dir() else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Data directory not found!",
                ));
            };

            let bin_name = bin_name.as_ref();

            let comp_script_path = data_dir
                .join("bash-completion")
                .join("completions")
                .join(format!("{}.sh", bin_name));

            build_comp_script_to_file(&ShellFlag::Bash, bin_name, comp_script_path)?;
            Ok(())
        }

        // ~/.zsh/completions/
        ShellFlag::Zsh => {
            let Some(home_dir) = dirs::home_dir() else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Home directory not found!",
                ));
            };

            let bin_name = bin_name.as_ref();

            let comp_script_path = home_dir
                .join(".zsh")
                .join("completions")
                .join(format!("{}.zsh", bin_name));

            build_comp_script_to_file(&ShellFlag::Zsh, bin_name, comp_script_path)?;
            Ok(())
        }

        // ~/.config/fish/completions/
        ShellFlag::Fish => {
            let Some(config_dir) = dirs::config_dir() else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Config directory not found!",
                ));
            };

            let bin_name = bin_name.as_ref();

            let comp_script_path = config_dir
                .join("fish")
                .join("completions")
                .join(format!("{}.fish", bin_name));

            build_comp_script_to_file(&ShellFlag::Fish, bin_name, comp_script_path)?;
            Ok(())
        }
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "unsupported shell flag",
        )),
    }
}
