use std::collections::HashMap;

use serde::Deserialize;
use tools::{eprintln_cargo_style, println_cargo_style, run_cmd};

#[derive(Deserialize)]
struct TestConfig {
    test: HashMap<String, Vec<TestCase>>,
}

#[derive(Deserialize)]
struct TestCase {
    command: String,
    expect: Expect,
}

#[derive(Deserialize)]
struct Expect {
    #[serde(rename = "exit-code")]
    exit_code: i32,
    result: String,
}

fn main() {
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);

    let config = load_config();
    let (passed, total) = run_all_tests(&config);

    println_cargo_style!("Result: {}/{} tests passed", passed, total);

    if passed != total {
        eprintln_cargo_style!("{} test(s) failed", total - passed);
        std::process::exit(1);
    }
}

/// Parse test config from TOML file
fn load_config() -> TestConfig {
    let content = std::fs::read_to_string("examples/test-example-async.toml").unwrap_or_else(|e| {
        eprintln_cargo_style!("Failed to read TOML config file: {}", e);
        std::process::exit(1);
    });

    toml::from_str(&content).unwrap_or_else(|e| {
        eprintln_cargo_style!("Failed to parse TOML config: {}", e);
        std::process::exit(1);
    })
}

/// Run all example test groups, return (passed, total)
fn run_all_tests(config: &TestConfig) -> (usize, usize) {
    let mut total = 0;
    let mut passed = 0;

    for (example_name, test_cases) in &config.test {
        println_cargo_style!("Test: {}", example_name);

        if !build_example(example_name) {
            total += test_cases.len();
            continue;
        }

        for test_case in test_cases {
            total += 1;
            if run_single_test(example_name, test_case) {
                passed += 1;
            }
        }
    }

    (passed, total)
}

/// Build the example binary, return true on success
fn build_example(example_name: &str) -> bool {
    let manifest = format!("examples/{}/Cargo.toml", example_name);
    run_cmd!("cargo build --manifest-path {}", manifest).is_ok()
}

/// Run a single test case, return true on pass
fn run_single_test(example_name: &str, test_case: &TestCase) -> bool {
    let binary_path = format!(".temp/target/debug/{}", get_binary_name(example_name));
    let args: Vec<&str> = test_case.command.split_whitespace().collect();

    let output = match std::process::Command::new(&binary_path)
        .args(&args)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            eprintln_cargo_style!("'{}' - failed to run: {}", test_case.command, e);
            return false;
        }
    };

    let actual_exit_code = output.status.code().unwrap_or(-1);
    let actual_stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let actual_stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let exit_ok = actual_exit_code == test_case.expect.exit_code;
    let result_ok = actual_stdout == test_case.expect.result
        || actual_stdout.contains(&test_case.expect.result);

    if exit_ok && result_ok {
        println_cargo_style!("Passed: '{}'", test_case.command);
        true
    } else {
        eprintln_cargo_style!("'{}'", test_case.command);
        if !exit_ok {
            eprintln_cargo_style!(
                "Expected exit code: {}, actual: {}",
                test_case.expect.exit_code,
                actual_exit_code
            );
        }
        if !result_ok {
            eprintln_cargo_style!("Expected output: {:?}", test_case.expect.result);
            eprintln_cargo_style!("Actual stdout: {:?}", actual_stdout);
            if !actual_stderr.is_empty() {
                eprintln_cargo_style!("Actual stderr: {:?}", actual_stderr);
            }
        }
        false
    }
}

/// Resolve binary filename for the given example
///
/// The binary name matches the package name. On Windows, the `.exe` suffix is required.
fn get_binary_name(example_name: &str) -> String {
    let base = example_name;
    if cfg!(target_os = "windows") {
        format!("{}.exe", base)
    } else {
        base.to_string()
    }
}
