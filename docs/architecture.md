# Architecture Documentation

## System Architecture

The skills-validator follows a modular, pipeline-oriented architecture with clear separation of concerns. The five-pass validation pipeline is the core execution path; the legacy validator is a deprecated wrapper around it.

```text
┌─────────────────────────────────────────────────────────────────────┐
│                           CLI Layer                                 │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────┐             │
│  │ validate │  │read-properties│  │    to-prompt     │             │
│  └────┬─────┘  └──────┬────────┘  └────────┬─────────┘             │
│       │               │                    │                       │
│       ▼               ▼                    ▼                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                          scan                               │   │
│  │        (discovers skills, feeds pipeline, aggregates)       │   │
│  └───────────────────────────┬─────────────────────────────────┘   │
└──────────────────────────────┼─────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Pipeline Layer                               │
│  ┌────────────┐  ┌─────────────┐  ┌────────────┐  ┌────────────┐   │
│  │  Pass 1    │  │   Pass 2    │  │  Pass 3    │  │  Pass 4    │   │
│  │  Parse     │  │  Structure  │  │  Content   │  │ References │   │
│  │  (AST)     │─▶│ (inventory) │─▶│(frontmatter│─▶│  (chains,  │   │
│  │            │  │             │  │  quality)  │  │  orphans)  │   │
│  └────────────┘  └─────────────┘  └────────────┘  └─────┬──────┘   │
│                                                          │          │
│  ┌────────────┐                                          │          │
│  │  Pass 5    │◀─────────────────────────────────────────┘          │
│  │  Security  │                                                     │
│  │(semgrep,   │                                                     │
│  │ remote exec│                                                     │
│  └────────────┘                                                     │
│                                                                     │
│  Each pass produces Vec<Diagnostic> (severity: info/suggestion/     │
│  warning/error). Pipeline merges and formats the full result.       │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Library Layer                                │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────────────┐   │
│  │   models     │  │  parser  │  │          prompt              │   │
│  └──────────────┘  └──────────┘  └──────────────────────────────┘   │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────────────┐   │
│  │   config     │  │formatter │  │        discovery             │   │
│  └──────────────┘  └──────────┘  └──────────────────────────────┘   │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────────────┐   │
│  │     git      │  │  paths   │  │  validator (deprecated)      │   │
│  └──────────────┘  └──────────┘  └──────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Module Structure

### Core Modules

| Module   | File           | Responsibility                                            |
| -------- | -------------- | --------------------------------------------------------- |
| `cli`    | `src/cli.rs`   | CLI argument parsing, command dispatch, flag definitions  |
| `error`  | `src/error.rs` | Error type definitions using `thiserror`                  |
| `models` | `src/models.rs`| Data structures (`Diagnostic`, `Severity`, `Sizeyness`, `PipelineResult`) |
| `parser` | `src/parser.rs`| YAML frontmatter parsing, file discovery                  |
| `prompt` | `src/prompt.rs`| XML prompt generation                                     |
| `main`   | `src/main.rs`  | Binary entry point                                        |
| `lib`    | `src/lib.rs`   | Public API exports and re-exports                         |

### Pipeline Modules

| Module              | File                       | Responsibility                                                  |
| ------------------- | -------------------------- | --------------------------------------------------------------- |
| `pipeline`          | `src/pipeline.rs`          | Pipeline orchestration: runs all five passes, merges diagnostics, computes exit codes |
| `config`            | `src/config.rs`            | Config loading from TOML with environment variable overrides    |
| `formatter`         | `src/formatter.rs`         | Human-readable and JSON output formatting (`schema_version`)    |
| `passes::parse`     | `src/passes/parse.rs`      | Pass 1: Parse skill markdown into a pulldown-cmark AST          |
| `passes::structure` | `src/passes/structure.rs`  | Pass 2: File inventory, sizeyness classification, binary detection |
| `passes::content`   | `src/passes/content.rs`    | Pass 3: Frontmatter validation, quality checks, reinforcement   |
| `passes::references`| `src/passes/references.rs` | Pass 4: Reference chain walking, orphan detection               |
| `passes::security`  | `src/passes/security.rs`   | Pass 5: Semgrep integration, remote execution detection         |

### Scan Modules

| Module      | File               | Responsibility                                                  |
| ----------- | ------------------ | --------------------------------------------------------------- |
| `scan`      | `src/scan.rs`      | Scan orchestration, parallel validation, duplicate detection    |
| `discovery` | `src/discovery.rs` | Skill discovery via directory walking                           |
| `git`       | `src/git.rs`       | Git repository detection using git2                             |
| `paths`     | `src/paths.rs`     | Path configuration loading and expansion (`paths.jsonc`)        |

### Legacy Modules

| Module      | File               | Responsibility                                                  |
| ----------- | ------------------ | --------------------------------------------------------------- |
| `validator` | `src/validator.rs` | **Deprecated.** Legacy validation logic; wraps the pipeline for backwards compatibility |

### Public API Surface

```rust
// src/lib.rs
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

## Data Flow

### Validation Flow (Pipeline)

```text
┌─────────────┐    ┌──────────────────────────────────────────────────┐
│  Skill Dir  │───▶│                   scan()                         │
└─────────────┘    │  discover_skills → for each DiscoveredSkill:     │
                   │    run_pipeline(skill_path, config)               │
                   └──────────────────┬───────────────────────────────┘
                                      │
                                      ▼
                   ┌──────────────────────────────────────────────────┐
                   │             run_pipeline()                       │
                   │                                                  │
                   │  ┌──────────┐  produces Vec<Diagnostic>          │
                   │  │ Pass 1   │  Parse: AST from pulldown-cmark    │
                   │  └────┬─────┘                                    │
                   │       ▼                                          │
                   │  ┌──────────┐  produces Vec<Diagnostic>          │
                   │  │ Pass 2   │  Structure: inventory, sizeyness,  │
                   │  └────┬─────┘             binary detection       │
                   │       ▼                                          │
                   │  ┌──────────┐  produces Vec<Diagnostic>          │
                   │  │ Pass 3   │  Content: frontmatter, quality,    │
                   │  └────┬─────┘           reinforcement checks     │
                   │       ▼                                          │
                   │  ┌──────────┐  produces Vec<Diagnostic>          │
                   │  │ Pass 4   │  References: chain walk,           │
                   │  └────┬─────┘              orphan detection      │
                   │       ▼                                          │
                   │  ┌──────────┐  produces Vec<Diagnostic>          │
                   │  │ Pass 5   │  Security: semgrep, remote exec    │
                   │  └────┬─────┘                                    │
                   │       ▼                                          │
                   │  Merge all diagnostics → PipelineResult          │
                   └──────────────────┬───────────────────────────────┘
                                      │
                                      ▼
                   ┌──────────────────────────────────────────────────┐
                   │  format_human() or format_json()                 │
                   │  exit_code() based on highest severity           │
                   └──────────────────────────────────────────────────┘
```

### Prompt Generation Flow

```text
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Skill Dirs   │───▶│  read_properties │───▶│  XML Generation  │
│   Input      │    │   (per skill)    │    │ (escaped output) │
└──────────────┘    └──────────────────┘    └──────────────────┘
```

### Scan Flow

```text
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Scan Options │───▶│  Paths Config    │───▶│ Git Repo Detect  │
│  (--all,     │    │  (paths.jsonc)   │    │    (git2)        │
│   --user,    │    └────────┬─────────┘    └────────┬─────────┘
│   --repo,    │             │                       │
│   --tool)    │             ▼                       ▼
└──────────────┘    ┌──────────────────┐    ┌──────────────────┐
                    │  Path Expansion  │    │  Repo Root       │
                    │ ($HOME, $REPO_   │    │  Detection       │
                    │   ROOT, ~)       │    │                  │
                    └────────┬─────────┘    └────────┬─────────┘
                             │                       │
                             └───────────┬───────────┘
                                         ▼
                            ┌────────────────────────┐
                            │   Skill Discovery      │
                            │  (WalkDir SKILL.md)    │
                            └───────────┬────────────┘
                                        ▼
                            ┌────────────────────────┐
                            │ Parallel Validation    │
                            │    (rayon)             │
                            └───────────┬────────────┘
                                        ▼
                            ┌────────────────────────┐
                            │   Result Aggregation   │
                            │  (PipelineResult per   │
                            │    skill)              │
                            └───────────┬────────────┘
                                        ▼
                            ┌────────────────────────┐
                            │ Duplicate Detection    │
                            │ + Exit Code            │
                            └───────────┬────────────┘
                                        ▼
                            ┌────────────────────────┐
                            │      Output Results    │
                            └────────────────────────┘
```

---

## Design Patterns

### Error Handling

Uses `thiserror` for ergonomic error types with automatic `From` implementations:

```rust
#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Failed to parse SKILL.md: {0}")]
    ParseError(String),
    #[error("Skill validation failed: {0}")]
    ValidationError(String),
}
```

### Diagnostic/Severity Pattern

The primary result type for all pipeline passes. Each diagnostic carries a severity level and a human-readable message:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Suggestion,
    Warning,
    Error,
}
```

- `Error` diagnostics cause exit code 1 (or exit code 2 in `--strict` mode for warnings)
- `Warning` diagnostics are reported but do not block by default
- `Suggestion` and `Info` are informational

The `--severity` flag controls the minimum level reported. The `--strict` flag promotes warnings to blocking.

### Sizeyness Escalation

Pass 2 classifies each skill by size and emits progressively stronger diagnostics as content grows beyond recommended thresholds:

- Small: no diagnostic
- Medium: `Suggestion` to consider splitting
- Large: `Warning`
- Huge: `Error`

### ValidationResult Pattern (Deprecated)

The legacy pattern used before v0.2.0. Aggregated errors and warnings as `Vec<String>`:

```rust
// Deprecated — use Diagnostic/Severity and run_pipeline() instead
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

Still exported for backwards compatibility via `validator::validate()`.

### Progressive Disclosure

Content validation (Pass 3) warns when skills exceed recommended sizes, encouraging focused, modular skills over monolithic documents.

---

## Key Algorithms

### Pipeline Orchestration

`run_pipeline()` in `pipeline.rs` runs all five passes sequentially against a single skill path. Each pass receives the parsed state from previous passes as needed and appends its `Vec<Diagnostic>` to the accumulating result. The final `PipelineResult` merges all diagnostics and records overall pass/fail.

### Skill Name Normalization

Uses Unicode NFKC normalization for consistent comparison:

```rust
let normalized: String = name.nfkc().collect();
```

### Reference Chain Walking

Pass 4 traverses `references` links declared in skill frontmatter, walking the full chain to detect cycles, broken links, and orphaned skills not reachable from any entry point.

### Binary Detection

Pass 2 inspects file content to detect non-text assets accidentally placed inside a skill directory, emitting an `Error` diagnostic if binary files are found where text is expected.

### Sizeyness Classification

Pass 2 measures total content length against configurable thresholds and assigns one of four `Sizeyness` buckets (`Small`, `Medium`, `Large`, `Huge`). The bucket drives diagnostic severity for the progressive disclosure pattern.

### Field Validation

1. Check for unknown fields (error)
2. Check for Claude Code extensions (warning)
3. Validate required fields with type checking
4. Validate optional fields if present

### Content Keyword Detection

Case-insensitive search for directive keywords:

```rust
let body_lower = body.to_lowercase();
for (keyword, guidance) in keywords {
    if body_lower.contains(keyword) {
        // Good practice found
    } else {
        // Warning issued
    }
}
```

---

## Constraints

### Performance

- Single-threaded for validation commands (validate, read-properties, to-prompt)
- Parallel validation using rayon for scan command
- Git repository detection via git2
- JSONC parsing at compile time (embedded via `include_str!`)
- In-memory YAML parsing
- Suitable for CI/CD pipelines

### Safety

- No unsafe code
- UTF-8 validation on all text
- Graceful handling of missing files

### Portability

- Most code is pure Rust
- git2 crate for git repository detection (platform-specific)
- Cross-platform path handling
- Forward-slash enforcement for paths

---

## Extension Points

### Adding New Validation Rules

1. Add a new pass or extend an existing pass in `src/passes/`
2. Emit `Diagnostic` values with the appropriate `Severity`
3. Register the pass in `pipeline.rs`
4. Add tests in `tests/passes_<name>.rs`
5. Document in validation-rules.md

### Adding New Fields

1. Update `SkillProperties` struct in `models.rs`
2. Update `ALLOWED_FIELDS` in the content pass (`passes/content.rs`)
3. Add parsing logic in `parser.rs`
4. Update XML generation in `prompt.rs`

---

## Testing Architecture

```text
tests/
├── helpers.rs               # Test utilities and shared fixtures
├── fixtures_integration.rs  # Integration tests with on-disk fixtures
├── cli_output.rs            # CLI output format tests (human + JSON)
├── validator.rs             # Legacy validation logic unit tests
├── parser.rs                # Parser unit tests
├── models.rs                # Model serialization tests
├── prompt.rs                # Prompt generation tests
├── config.rs                # Config loading and env override tests
├── formatter.rs             # Formatter output tests (human + JSON)
├── pipeline.rs              # Pipeline orchestration integration tests
├── passes_parse.rs          # Pass 1 (Parse) unit tests
├── passes_structure.rs      # Pass 2 (Structure) unit tests
├── passes_content.rs        # Pass 3 (Content) unit tests
├── passes_references.rs     # Pass 4 (References) unit tests
└── passes_security.rs       # Pass 5 (Security) unit tests
```

Test philosophy:

- Unit tests for individual functions and pass logic
- Integration tests with temporary directories
- Fixture-based tests for edge cases
- Each pass has its own test module mirroring the source structure
