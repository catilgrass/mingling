# Contribution Guide

First of all, thank you for your interest in Mingling! 🎉 Whether it's fixing bugs, improving documentation, adding new features, or making suggestions, we welcome all contributions.

Before contributing, we recommend reading [README](README.md) to get an overview of the project.



## 1. Project Structure

| Category | Path/Name | Description |
|----------|-----------|-------------|
| **Entry crate** | `mingling/` | Project entry point |
| **Core library** | `mingling_core/` | Imported as an external dependency |
| **Macro library** | `mingling_macros/` | Imported as an external dependency |
| **Examples** | `examples/` | To add expected output tests, modify `examples/test-examples.toml` |
| **Documentation, resources** | `docs/` | All documentation and resource files |
| **Development tools** | `dev_tools/` | Contains scripts and Rust tools |
| Scripts | `dev_tools/scripts/` | Helper `.sh`/`.ps1`/`.py` scripts, executed via `./run-tools.sh` or `.\run-tools.ps1` |
| Rust tools | `dev_tools/src/bin/` | Same as above |
| CI check entry | `dev_tools/src/bin/ci.rs` | Can be invoked directly via `cargo ci` |
| **Scaffolding tool** | `mling/` | Scaffolding tool `mingling-cli` |
| **Temporary files** | `.temp/` | Ignored by `.gitignore` |



## 2. Submission Guide

1. **Pull Request**

   - Submit a GitHub Pull Request and @Reviewer **[Weicao-CatilGrass](https://github.com/Weicao-CatilGrass)** for review
   - Or send patches to **catil_grass@qq.com**

2. **Commit Messages**
   - Clearly and concisely describe the changes, no stringent requirements
   - Provide more detail for complex changes, keep it brief for simple changes

3. **CHANGELOG**
   - If the submission includes functional changes or fixes, **the PR must include modifications to CHANGELOG.md** to describe the changes
   - For minor changes like typo fixes, **CHANGELOG.md modification is not required**, and we will merge faster

4. **Multi-commit PR**

   - A PR can contain multiple commits
   - However, at least one commit must modify CHANGELOG.md

5. **Review**
   - After submission, please notify [Weicao-CatilGrass](https://github.com/Weicao-CatilGrass) for review — this is the most efficient way to get feedback

6. **Binary Resources**
   - For binary resource files (images, etc.), please be cautious about adding them to avoid repository bloat



## 3. Documentation Contribution

### Documentation Location

- English documentation: `docs/pages/`
- Chinese documentation: `docs/_zh_CN/pages/`

### Documentation Build

After editing documentation, refresh relevant files:

```bash
# Refresh sidebar and README sync
./run-tools.sh docsify-sidebar-gen
./run-tools.sh refresh-docs

# Fix code block blank line issues
./run-tools.sh docs-code-box-fix
```

These steps are included in `cargo ci`; running `cargo ci` will execute them automatically.



## 4. Regarding AI Agent Usage

- You are free to use AI agents to assist development — no restrictions
- **Humans are the final decision-makers**, everything is subject to human judgment
- Please **DO NOT** leave AI instruction files like `CLAUDE.md` in the repository root. Mingling currently has no plans to introduce **Harness Engineering**



## 5. License

Mingling uses the **MIT + Apache 2.0** dual license. For details, please see:

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)
