use std::path::Path;

use just_fmt::snake_case;
use just_template::{Template, tmpl};

const EXAMPLE_ROOT: &str = "./examples/";
const OUTPUT_PATH: &str = "./mingling/src/example_docs.rs";

const TEMPLATE_CONTENT: &str = include_str!("../../../mingling/src/example_docs.rs.tmpl");

fn main() {
    {
        println!("Refreshing Examples");
        gen_example_doc_module();
    }
}

fn gen_example_doc_module() {
    let mut template = Template::from(TEMPLATE_CONTENT);
    let repo_root = find_git_repo().unwrap();
    let example_root = repo_root.join(EXAMPLE_ROOT);
    let mut examples = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&example_root) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type()
                && file_type.is_dir()
            {
                let example_name = entry.file_name().to_string_lossy().to_string();
                let example_content = ExampleContent::read(&example_name);
                examples.push(example_content);
            }
        }
    }

    for example in examples {
        tmpl!(template += {
            examples {
                (
                    example_header = example.header,
                    example_import = example.cargo_toml,
                    example_code = example.code,
                    example_name = snake_case!(&example.name)
                )
            }
        });
        println!("  Refresh: {}", example.name);
    }

    let template_str = template.to_string();
    let template_str = template_str
        .lines()
        .map(|line| line.trim_end())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(repo_root.join(OUTPUT_PATH), template_str).unwrap();
}

struct ExampleContent {
    name: String,
    header: String,
    code: String,
    cargo_toml: String,
}

impl ExampleContent {
    pub fn read(name: &str) -> Self {
        let repo = find_git_repo().unwrap();
        let cargo_toml = Self::read_cargo_toml(&repo, name);
        let (header, code) = Self::read_header_and_code(&repo, name);

        let cargo_toml = cargo_toml
            .lines()
            .map(|line| format!("/// {}", line))
            .collect::<Vec<_>>()
            .join("\n");

        let header = header
            .lines()
            .map(|line| format!("/// {}", line))
            .collect::<Vec<_>>()
            .join("\n");

        let code = code
            .lines()
            .map(|line| format!("/// {}", line))
            .collect::<Vec<_>>()
            .join("\n");

        ExampleContent {
            name: name.to_string(),
            header,
            code,
            cargo_toml,
        }
    }

    fn read_header_and_code(repo: &Path, name: &str) -> (String, String) {
        let file_path = repo
            .join(EXAMPLE_ROOT)
            .join(name)
            .join("src")
            .join("main.rs");
        let content = std::fs::read_to_string(&file_path).unwrap_or_default();
        let mut lines = content.lines();
        let mut header = String::new();
        let mut code = String::new();

        // Collect header lines (starting with //!)
        for line in lines.by_ref() {
            if line.trim_start().starts_with("//!") {
                let trimmed = line.trim_start_matches("//!");
                header.push_str(trimmed);
                header.push('\n');
            } else {
                // First non-header line found, start collecting code
                code.push_str(line);
                code.push('\n');
                break;
            }
        }

        // Collect remaining code lines
        for line in lines {
            code.push_str(line);
            code.push('\n');
        }

        (header.trim().to_string(), code.trim().to_string())
    }

    fn read_cargo_toml(repo: &Path, name: &str) -> String {
        let file_path = repo.join(EXAMPLE_ROOT).join(name).join("Cargo.toml");

        std::fs::read_to_string(&file_path).unwrap_or_default()
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
