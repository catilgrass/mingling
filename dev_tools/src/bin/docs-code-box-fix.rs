use std::fs;
use std::path::Path;

use tools::println_cargo_style;

/// Docsify code blocks require that blank lines before and after code blocks are not completely empty,
/// but must contain at least one space, otherwise code block rendering will have issues.
///
/// This tool scans all `.md` files in the docs directory,
/// and replaces completely empty lines before and after code blocks with blank lines containing a single space.
const DOCS_DIR: &str = "./docs";

fn main() {
    println_cargo_style!("Fixing: code box empty lines in docs/**/*.md ...");
    let repo_root = find_git_repo().expect("Cannot find git repo root");
    let docs_dir = repo_root.join(DOCS_DIR);

    let mut fixed_count = 0;
    let mut file_count = 0;

    collect_md_files(&docs_dir, &mut |path| {
        if let Some(name) = path.file_name() {
            let name = name.to_string_lossy();
            if name.to_lowercase() == "_sidebar.md" {
                return;
            }
        }

        let content = fs::read_to_string(path).unwrap_or_default();
        if content.is_empty() {
            return;
        }

        let new_content = fix_code_box_empty_lines(&content);
        if new_content != content {
            fs::write(path, &new_content).unwrap();
            println_cargo_style!("Fixed: {}", path.display());
            fixed_count += 1;
        }
        file_count += 1;
    });

    println_cargo_style!(
        "Done: Scanned {} files, fixed {} files.",
        file_count,
        fixed_count
    );
}

fn fix_code_box_empty_lines(content: &str) -> String {
    let mut result = String::new();
    let lines: Vec<&str> = content.lines().collect();
    let len = lines.len();

    let mut i = 0;
    while i < len {
        let line = lines[i];

        // detect beginning of code block: beginning with ```
        if line.trim_start().starts_with("```") {
            // record the beginning line of the code block
            result.push_str(line);
            result.push('\n');
            i += 1;

            // find the end of the code block
            let mut found_end = false;
            let code_start = i; // record starting position of code content
            let mut code_end = len; // index of code block end line

            while i < len {
                let cline = lines[i];
                if cline.trim_start().starts_with("```") && cline.trim() != "" {
                    // this is the closing marker
                    code_end = i;
                    found_end = true;
                    break;
                }
                i += 1;
            }

            // check the blank line before the code block
            // if result ends with \n\n, add a space to turn it into \n \n
            ensure_space_before_code_block(&mut result);

            // output code content
            for code_line in lines.iter().take(code_end).skip(code_start) {
                if code_line.is_empty() {
                    result.push(' ');
                } else {
                    result.push_str(code_line);
                }
                result.push('\n');
            }

            if found_end {
                result.push_str(lines[code_end]);
                result.push('\n');
                i += 1;

                // check the blank line after the code block
                // if the next line is blank, change it to one with a space
                if i < len && lines[i].trim().is_empty() && lines[i].is_empty() {
                    // skip the original blank line, write " \n"
                    result.push(' ');
                    result.push('\n');
                    i += 1;
                }
            }
        } else {
            result.push_str(line);
            result.push('\n');
            i += 1;
        }
    }

    // remove trailing newlines
    while result.ends_with('\n') {
        result.pop();
    }
    result.push('\n');

    result
}

/// ensure there is a blank line with a space before the code block
fn ensure_space_before_code_block(result: &mut String) {
    // if result ends with \n\n,
    // turn it into \n \n
    let len = result.len();
    if len >= 2 && result[len - 2..] == *"\n\n" {
        // insert a space before the last \n
        result.insert(len - 1, ' ');
    }
}

/// recursively collect all .md files in the docs directory
fn collect_md_files(dir: &Path, callback: &mut dyn FnMut(&Path)) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_md_files(&path, callback);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                callback(&path);
            }
        }
    }
}

fn find_git_repo() -> Option<std::path::PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;

    loop {
        let git_dir = current_dir.join(".git");
        if git_dir.exists() && git_dir.is_dir() {
            return Some(current_dir);
        }

        if !current_dir.pop() {
            break;
        }
    }

    None
}
