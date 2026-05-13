use std::process::exit;

use tools::{cargo_tomls, eprintln_cargo_style, println_cargo_style, run_cmd};

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);
    println!("{}", include_str!("../../../docs/res/ci_banner.txt"));

    let needs_commit_temp = !{ run_cmd!("git diff-index --quiet HEAD --").is_ok() };

    if needs_commit_temp {
        run_cmd!("git add .").unwrap();
        run_cmd!("git commit -m \"CI Temp\"").unwrap();
    }

    if ci().is_ok() {
        println_cargo_style!("Done: All check passed!")
    }

    let is_worktree_clean = run_cmd!("git diff-index --quiet HEAD --").is_ok();
    if !is_worktree_clean {
        eprintln_cargo_style!("Documents needs refresh!");
        if needs_commit_temp {
            run_cmd!("git restore .").unwrap();
            run_cmd!("git reset --soft HEAD~1").unwrap();
        }
        exit(1)
    }

    if needs_commit_temp {
        run_cmd!("git restore .").unwrap();
        run_cmd!("git reset --soft HEAD~1").unwrap();
    }
}

fn ci() -> Result<(), i32> {
    build_all()?;
    clippy_all()?;
    test_all()?;
    test_examples()?;
    docs_refresh()?;

    run_cmd!("git add --renormalize .")?;

    Ok(())
}

fn test_examples() -> Result<(), i32> {
    println_cargo_style!("Testing: examples");
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin test-examples")
}

fn build_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Build: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo check --manifest-path {}",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
}

fn clippy_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Clippy: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo clippy --manifest-path {} -- -D warnings",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
}

fn test_all() -> Result<(), i32> {
    let cargo_tomls = cargo_tomls();
    for cargo_toml in cargo_tomls {
        println_cargo_style!("Testing: {}", cargo_toml.to_string_lossy());
        run_cmd!(
            "cargo test --manifest-path {}",
            cargo_toml.to_string_lossy()
        )?;
    }

    Ok(())
}

fn docs_refresh() -> Result<(), i32> {
    println_cargo_style!("Refresh: document at `./docs/`");

    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin docs-code-box-fix")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin docsify-sidebar-gen")?;
    run_cmd!("cargo run --manifest-path dev_tools/Cargo.toml --bin refresh-docs")?;

    Ok(())
}
