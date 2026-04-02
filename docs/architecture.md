# Architecture Documentation

## System Architecture

The skills-validator follows a modular, domain-driven architecture with clear separation of concerns.

```text
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│  ┌──────────┐  ┌───────────────┐  ┌──────────────────┐      │
│  │ validate │  │read-properties│  │    to-prompt     │      │
│  └────┬─────┘  └──────┬────────┘  └────────┬─────────┘      │
│       │               │                    │                │
│       ▼               ▼                    ▼                │
│  ┌─────────────────────────────────────────────────────┐    │
│  │                       scan                          │    │
│  │  (discovers and validates skills across paths)      │    │
│  └───────────────────────┬─────────────────────────────┘    │
└──────────────────────────┼──────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                      Library Layer                          │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │   validator  │  │  parser  │  │       prompt         │   │
│  └──────────────┘  └──────────┘  └──────────────────────┘   │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │    models    │  │  error   │  │         cli          │   │
│  └──────────────┘  └──────────┘  └──────────────────────┘   │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │    scan      │  │ discovery│  │        git           │   │
│  └──────────────┘  └──────────┘  └──────────────────────┘   │
│  ┌──────────────┐                                           │
│  │    paths     │                                           │
│  └──────────────┘                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Structure

### Core Modules

| Module      | File               | Responsibility                                        |
| ----------- | ------------------ | ----------------------------------------------------- |
| `cli`       | `src/cli.rs`       | CLI argument parsing, command dispatch, logging setup |
| `error`     | `src/error.rs`     | Error type definitions using `thiserror`              |
| `models`    | `src/models.rs`    | Data structures (`SkillProperties`)                   |
| `parser`    | `src/parser.rs`    | YAML frontmatter parsing, file discovery              |
| `validator` | `src/validator.rs` | Validation logic, rules engine                        |
| `prompt`    | `src/prompt.rs`    | XML prompt generation                                 |

### Scan Modules

| Module      | File               | Responsibility                                               |
| ----------- | ------------------ | ------------------------------------------------------------ |
| `scan`      | `src/scan.rs`      | Scan orchestration, parallel validation, duplicate detection |
| `discovery` | `src/discovery.rs` | Skill discovery via directory walking                        |
| `git`       | `src/git.rs`       | Git repository detection using git2                          |
| `paths`     | `src/paths.rs`     | Path configuration loading and expansion (paths.jsonc)       |

### Public API Surface

```rust
// src/lib.rs
pub use parser::{find_skill_md, parse_frontmatter, read_properties};
pub use prompt::to_prompt;
pub use validator::{validate, ValidationResult};
```

---

## Data Flow

### Validation Flow

```text
┌─────────────┐    ┌──────────────┐    ┌──────────────────┐
│  Skill Dir  │───▶│  find_skill   │───▶│   Read SKILL.md  │
└─────────────┘    │    _md()     │    └────────┬─────────┘
                   └──────────────┘             │
                                                ▼
                   ┌──────────────┐    ┌───────────────────┐
                   │   Output     │◀───│  parse_frontmatter│
                   │  Results     │    │     _and_body()   │
                   └──────────────┘    └────────┬──────────┘
                                                │
                          ┌─────────────────────┼─────────────────────┐
                          ▼                     ▼                     ▼
                   ┌─────────────┐      ┌──────────────┐      ┌──────────────┐
                   │validate_name│      │validate_desc │      │validate_meta │
                   └──────┬──────┘      └──────┬───────┘      └──────┬───────┘
                          │                    │                     │
                          └────────────────────┼─────────────────────┘
                                               ▼
                                        ┌──────────────┐
                                        │validate_body │
                                        │_keywords()   │
                                        └──────────────┘
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
│ Scan Options │───▶│  Paths Config     │───▶│ Git Repo Detect  │
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
                            │ (valid/invalid/warning)│
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

### ValidationResult Pattern

Aggregates errors and warnings separately:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

- Errors block validation (exit code 1)
- Warnings don't block (exit code 0 with warnings message)

### Progressive Disclosure

Content validation warns when skills exceed recommended sizes, encouraging focused, modular skills over monolithic documents.

---

## Key Algorithms

### Skill Name Normalization

Uses Unicode NFKC normalization for consistent comparison:

```rust
let normalized: String = name.nfkc().collect();
```

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

1. Add validation function in `validator.rs`
2. Call from `validate()` function
3. Add tests in `tests/validator.rs`
4. Document in validation-rules.md

### Adding New Fields

1. Update `SkillProperties` struct in `models.rs`
2. Update `ALLOWED_FIELDS` in `validator.rs`
3. Add parsing logic in `parser.rs`
4. Update XML generation in `prompt.rs`

---

## Testing Architecture

```text
tests/
├── helpers.rs              # Test utilities
├── fixtures_integration.rs # Integration tests with fixtures
├── cli_output.rs          # CLI output format tests
├── validator.rs           # Validation logic unit tests
├── parser.rs              # Parser unit tests
├── models.rs              # Model serialization tests
└── prompt.rs              # Prompt generation tests
```

Test philosophy:

- Unit tests for individual functions
- Integration tests with temporary directories
- Fixture-based tests for edge cases
