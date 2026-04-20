# Development Guide

## Getting Started

### Prerequisites

- Rust toolchain (latest stable)
- Just command runner (`cargo install just`)
- Node.js and pnpm (for markdown linting)
- Optional tools:
  - cargo-audit (security audits)
  - actionlint (workflow linting)
  - zizmor (workflow security)
  - semgrep (security pass integration)

### Installation

```bash
# Clone the repository
git clone https://github.com/moutons/skills-validator
cd skills-validator

# Build the project
cargo build --release

# Run tests
cargo test
```

---

## Project Structure

```text
.
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Library exports
│   ├── cli.rs           # CLI commands and argument parsing
│   ├── config.rs        # Config loading (TOML + env overrides)
│   ├── discovery.rs     # Skill discovery via directory walking
│   ├── error.rs         # Error type definitions
│   ├── formatter.rs     # Human and JSON output formatting
│   ├── git.rs           # Git repository detection
│   ├── models.rs        # Data structures (Diagnostic, Severity, Sizeyness)
│   ├── parser.rs        # YAML frontmatter parsing
│   ├── passes/          # Five-pass validation pipeline
│   │   ├── mod.rs       # Pass module exports
│   │   ├── parse.rs     # Pass 1: Parse
│   │   ├── structure.rs # Pass 2: Structure
│   │   ├── content.rs   # Pass 3: Content
│   │   ├── references.rs# Pass 4: References
│   │   └── security.rs  # Pass 5: Security
│   ├── paths.rs         # Path configuration (paths.jsonc)
│   ├── pipeline.rs      # Pipeline orchestration
│   ├── prompt.rs        # XML prompt generation
│   ├── scan.rs          # Scan orchestration
│   └── validator.rs     # Legacy validation (deprecated)
├── tests/               # Integration and unit tests
├── docs/                # Documentation
├── Cargo.toml           # Rust dependencies
├── Justfile             # Task runner recipes
└── README.md            # User documentation
```

---

## Development Workflow

### Required Before Committing

Run all CI checks locally:

```bash
just ensure-ci
```

This runs:

1. `fmt` - Check Rust formatting
2. `clippy` - Run clippy lints
3. `test` - Run all tests
4. `security` - Security audit
5. `markdown` - Markdown linting
6. `workflows` - Workflow validation
7. `build` - Release build

### Individual Checks

```bash
# Formatting
just fmt                  # Check formatting

# Linting
just clippy               # Run clippy with warnings as errors

# Testing
just test                 # Run tests

# Security
just security             # Run cargo audit and gitleaks

# Documentation
just markdown             # Format and lint markdown

# CI Workflows
just workflows            # Lint and validate GitHub Actions

# Building
just build                # Release build
just publish              # Build for publishing (quiet)

# Cleaning
just clean                # Remove build artifacts

# Everything
just full                 # Run all checks

# Install dependencies
just deps                 # Fetch dependencies
```

---

## Code Conventions

### Rust Style

Follow standard Rust conventions:

1. **Formatting**: Use `cargo fmt`
2. **Linting**: Use `cargo clippy -- -D warnings`
3. **Documentation**: Document all public APIs
4. **Error Handling**: Use `thiserror` for error types
5. **Naming**: Use `snake_case` for functions/variables, `PascalCase` for types

### Error Handling Pattern

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Failed to parse SKILL.md: {0}")]
    ParseError(String),
    #[error("Skill validation failed: {0}")]
    ValidationError(String),
}

// Automatic conversions
impl From<std::io::Error> for SkillError {
    fn from(err: std::io::Error) -> Self {
        SkillError::ParseError(err.to_string())
    }
}
```

### Module Organization

```rust
// lib.rs - Re-export public API
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod formatter;
pub mod git;
pub mod models;
pub mod parser;
pub mod passes;
pub mod paths;
pub mod pipeline;
pub mod prompt;
pub mod scan;
pub mod validator;

pub use config::ValidatorConfig;
pub use discovery::{discover_skills, DiscoveredSkill, DiscoveryResult};
pub use formatter::{format_human, format_json};
pub use git::{find_repo_root, GitError};
pub use models::{Diagnostic, Severity};
pub use parser::{find_skill_md, parse_frontmatter, read_properties};
pub use paths::{expand_path, PathsConfig, PathsError};
pub use pipeline::{exit_code, run_pipeline, PipelineResult};
pub use prompt::to_prompt;
pub use scan::{find_duplicates, scan, ScanOptions, ScanResult, SkillValidation};
#[allow(deprecated)]
pub use validator::{validate, ValidationResult};
```

---

## Testing

### Test Organization

```text
tests/
├── helpers.rs              # Shared test utilities
├── fixtures_integration.rs # Tests with fixture files
├── cli_output.rs           # CLI output format tests
├── config.rs               # Config system tests
├── formatter.rs            # Formatter tests
├── models.rs               # Model tests
├── parser.rs               # Parser tests
├── passes_content.rs       # Pass 3 tests
├── passes_parse.rs         # Pass 1 tests
├── passes_references.rs    # Pass 4 tests
├── passes_security.rs      # Pass 5 tests
├── passes_structure.rs     # Pass 2 tests
├── pipeline.rs             # Pipeline integration tests
├── prompt.rs               # Prompt generation tests
└── validator.rs            # Legacy validator tests
```

### Writing Tests

**Unit tests** (in source files):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(result, expected);
    }
}
```

**Integration tests** (in `tests/`):

```rust
use skills_validator::validate;
use std::path::Path;

#[test]
fn test_validate_skill() {
    let result = validate(Path::new("tests/fixtures/valid-skill"));
    assert!(result.errors.is_empty());
}
```

**Using temporary directories:**

```rust
use tempfile::tempdir;
use std::fs;

#[test]
fn test_with_temp_dir() {
    let dir = tempdir().unwrap();
    let skill_md = dir.path().join("SKILL.md");
    fs::write(&skill_md, "---\nname: test\n---\n").unwrap();

    let result = validate(dir.path());
    // assertions...
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Filtered
cargo test validator::
```

---

## Adding New Features

### Adding a Validation Check

1. **Identify the appropriate pass** - Determine which of the five passes should check your rule:
   - Pass 1 (Parse): Frontmatter structure and key presence
   - Pass 2 (Structure): Markdown structure and formatting
   - Pass 3 (Content): Content quality and completeness
   - Pass 4 (References): Cross-references and links
   - Pass 5 (Security): Security scanning and risk detection

2. **Implement the check** in the appropriate pass file (e.g., `src/passes/content.rs`):

```rust
pub fn check_my_rule(props: &SkillProperties) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    // validation logic...
    if some_check_fails {
        diagnostics.push(Diagnostic {
            file: "SKILL.md".to_string(),
            severity: Severity::Warning,
            check_name: "my-rule-name".to_string(),
            message: "Description of the issue".to_string(),
        });
    }
    diagnostics
}
```

1. **Integrate into the pass** - Call your check from the main pass function and collect diagnostics

1. **Add tests** in the corresponding test file (e.g., `tests/passes_content.rs`):

```rust
#[test]
fn test_my_rule() {
    // test cases...
}
```

1. **Document** in `docs/validation-rules.md`

### Adding a New Field

1. **Update model** in `src/models.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProperties {
    // existing fields...
    #[serde(skip_serializing_if = "Option::is_none", rename = "new-field")]
    pub new_field: Option<String>,
}
```

1. **Update validator** in `src/validator.rs`:

```rust
const ALLOWED_FIELDS: &[&str] = &[
    // existing fields...
    "new-field",
];
```

1. **Update parser** in `src/parser.rs`:

```rust
let new_field = get_optional_string(map, "new-field");
```

1. **Add tests** and **update documentation**

---

## Debugging

### Logging

Use the `log` crate for diagnostic output:

```rust
log::debug!("Processing skill: {:?}", skill_dir);
log::info!("Validation complete");
log::warn!("Missing keyword: {}", keyword);
log::error!("Validation failed: {}", error);
```

Run with log level:

```bash
# Debug logging
skills-validator -l debug validate ./my-skill

# JSON output for parsing
skills-validator --json validate ./my-skill
```

### Common Issues

**Clippy warnings:**

```bash
just clippy
```

**Test failures:**

```bash
cargo test -- --nocapture
```

**Build errors:**

```bash
cargo build --verbose
```

---

## Release Process

1. **Update version** in `Cargo.toml`:

```toml
version = "0.2.0"
```

The CLI version is automatically derived from `Cargo.toml` via clap derive macros.

1. **Update README.md** installation instructions

1. **Update documentation** as needed in `docs/`

1. **Run full checks**:

```bash
just full
```

1. **Commit and tag**:

```bash
git add -A
git commit -m "chore(release): bump version to 0.2.0"
git tag v0.2.0
git push origin main --tags
```

1. **GitHub Actions** will build and create release automatically

---

## Contributing

### Before Submitting

- Run `just ensure-ci` locally
- Ensure all tests pass
- Follow existing code style
- Document new features
- Update relevant documentation in `docs/`

### Commit Messages

Follow conventional commits:

```text
feat: add new validation rule for X
fix: correct path handling on Windows
docs: update API reference
test: add integration tests for Y
refactor: simplify validation logic
chore: update dependencies
```

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Clap Documentation](https://docs.rs/clap/)
- [Serde Documentation](https://serde.rs/)
- [Agent Skills Spec](https://agentskills.io/specification)
- [OpenCode Skills](https://opencode.ai/docs/skills/)
- [Claude Code Skills](https://code.claude.com/docs/en/skills)
