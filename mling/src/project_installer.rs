use std::path::PathBuf;

use mingling::{ShellFlag, build::build_comp_script_to};

use crate::{
    namespace_manager::{bin_dir, comp_dir, exe_path, working_dir},
    project_solver::solve,
};

const SCRIPT_LOAD_BASH: &str = include_str!("../tmpl/load.sh");
const SCRIPT_LOAD_FISH: &str = include_str!("../tmpl/load.fish");
const SCRIPT_LOAD_PWSH: &str = include_str!("../tmpl/load.ps1");

#[derive(serde::Deserialize)]
struct CargoToml {
    package: Package,
}

#[derive(serde::Deserialize)]
struct Package {
    name: String,
}

pub fn install_all(clean_before_build: bool) -> Result<(), std::io::Error> {
    let current = std::env::current_dir()?;
    install_this_project(current, clean_before_build)?;
    install_shell_scripts()?;
    Ok(())
}

pub fn install_this_project(
    current: PathBuf,
    clean_before_build: bool,
) -> Result<(), std::io::Error> {
    // Obtain context data
    let solved = solve(current)?;

    let workspace_root = &solved.workspace_root;

    // If clean_before_build, execute cargo clean in workspace_root first
    if clean_before_build {
        let status = std::process::Command::new("cargo")
            .arg("clean")
            .current_dir(workspace_root)
            .status()?;
        if !status.success() {
            return Err(std::io::Error::other("exec `cargo clean` failed"));
        }
    }

    // Execute cargo build --release in workspace_root
    let status = std::process::Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(workspace_root)
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other("cargo build --release failed"));
    }

    // Parse package.name from workspace_root's Cargo.toml as namespace
    let cargo_toml_content = std::fs::read_to_string(workspace_root.join("Cargo.toml"))?;
    let cargo_toml: CargoToml = toml::from_str(&cargo_toml_content).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to parse Cargo.toml: {e}"),
        )
    })?;
    let namespace = cargo_toml.package.name;

    // Ensure destination directories exist
    std::fs::create_dir_all(bin_dir(namespace.clone()))?;
    std::fs::create_dir_all(comp_dir(namespace.clone()))?;

    // Copy binaries to corresponding exe_path
    for bin in &solved.binaries {
        let dst = exe_path(namespace.clone(), bin.name.clone());
        std::fs::copy(&bin.path, &dst)?;
    }

    // Copy all completion scripts containing _comp from target/release to comp_dir
    let target_dir = &solved.target_dir;
    let release_dir = target_dir.join("release");
    if release_dir.exists() {
        for entry in std::fs::read_dir(&release_dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str.contains("_comp") {
                let dest = comp_dir(namespace.clone()).join(file_name.as_os_str());
                std::fs::copy(entry.path(), &dest)?;
            }
        }
    }

    Ok(())
}

pub fn install_shell_scripts() -> Result<(), std::io::Error> {
    // Get the working directory (mingling data dir)
    let wdir = working_dir();
    std::fs::create_dir_all(&wdir)?;

    // Build shell completion scripts for the "mling" command based on the current OS
    let mling_comp = if cfg!(target_os = "windows") {
        vec![ShellFlag::Powershell]
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        vec![ShellFlag::Bash, ShellFlag::Zsh, ShellFlag::Fish]
    } else {
        vec![ShellFlag::Bash]
    };

    for flag in mling_comp {
        build_comp_script_to(
            &flag,
            "mling",
            wdir.join(".comp").display().to_string().as_str(),
        )?;
    }

    // Determine which scripts to write based on platform
    let scripts: Vec<(&str, &str)> = if cfg!(target_os = "windows") {
        vec![("load.ps1", SCRIPT_LOAD_PWSH)]
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        vec![
            ("load.sh", SCRIPT_LOAD_BASH),
            ("load.fish", SCRIPT_LOAD_FISH),
        ]
    } else {
        // Fallback: write bash script
        vec![("load.sh", SCRIPT_LOAD_BASH)]
    };

    for (filename, content) in scripts {
        let dest = wdir.join(filename);
        std::fs::write(&dest, content)?;
        if cfg!(target_os = "linux") {
            let status = std::process::Command::new("chmod")
                .args(["+x", &dest.to_string_lossy()])
                .status()?;
            if !status.success() {
                eprintln!("Failed to chmod {}", filename);
            }
        }
    }

    Ok(())
}
