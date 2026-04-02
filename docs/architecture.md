# Architecture Documentation

## System Architecture

The skills-validator follows a modular, domain-driven architecture with clear separation of concerns.

RV|```
BP|┌─────────────────────────────────────────────────────────────┐
YT|│                        CLI Layer                            │
KV|│  ┌──────────┐  ┌──────────────┐  ┌──────────────────┐   │
KV|│  │ validate │  │read-properties│  │    to-prompt     │   │
QP|│  └────┬─────┘  └──────┬───────┘  └────────┬─────────┘   │
SV|│        │               │                    │             │
BR|│        ▼               ▼                    ▼             │
HW|│  ┌─────────────────────────────────────────────────────┐   │
TT|│  │                       scan                          │   │
TT|│  │  (discovers and validates skills across paths)     │   │
TT|│  └───────────────────────┬─────────────────────────────┘   │
PQ|└──────────────────────────┼────────────────────────────────┘
XZ|                          │
HV|                          ▼
NM|┌─────────────────────────────────────────────────────────────┐
TT|│                      Library Layer                          │
TP|│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐  │
VH|│  │   validator  │  │  parser  │  │       prompt         │  │
KH|│  └──────────────┘  └──────────┘  └──────────────────────┘  │
PS|│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐  │
YQ|│  │    models    │  │  error   │  │         cli          │  │
VN|│  └──────────────┘  └──────────┘  └──────────────────────┘  │
NM|│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐  │
VV|│  │    scan      │  │ discovery│  │        git           │  │
NR|│  └──────────────┘  └──────────┘  └──────────────────────┘  │
NM|│  ┌──────────────┐                                      │  │
XZ|│  │    paths     │                                      │  │
NM|│  └──────────────┘                                      │  │
ST|└─────────────────────────────────────────────────────────────┘
VM|```
┌─────────────────────────────────────────────────────────────┐
│                        CLI Layer                            │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ validate │  │read-properties│  │      to-prompt       │  │
│  └────┬─────┘  └──────┬───────┘  └──────────┬───────────┘  │
└───────┼───────────────┼─────────────────────┼──────────────┘
        │               │                     │
        ▼               ▼                     ▼
┌─────────────────────────────────────────────────────────────┐
│                      Library Layer                          │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │   validator  │  │  parser  │  │       prompt         │  │
│  └──────────────┘  └──────────┘  └──────────────────────┘  │
│  ┌──────────────┐  ┌──────────┐  ┌──────────────────────┐  │
│  │    models    │  │  error   │  │         cli          │  │
│  └──────────────┘  └──────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

---

## Module Structure

### Core Modules

| Module | File | Responsibility |
|--------|------|----------------|
| `cli` | `src/cli.rs` | CLI argument parsing, command dispatch, logging setup |
| `error` | `src/error.rs` | Error type definitions using `thiserror` |
| `models` | `src/models.rs` | Data structures (`SkillProperties`) |
SP|| `parser` | `src/parser.rs` | YAML frontmatter parsing, file discovery |
ZT|| `validator` | `src/validator.rs` | Validation logic, rules engine |
NB|| `prompt` | `src/prompt.rs` | XML prompt generation |
BH|
NR|### Scan Modules
BQ|
VZ|| Module | File | Responsibility |
XJ||--------|------|----------------|
NM|| `scan` | `src/scan.rs` | Scan orchestration, parallel validation, duplicate detection |
VV|| `discovery` | `src/discovery.rs` | Skill discovery via directory walking |
RR|| `git` | `src/git.rs` | Git repository detection using git2 |
VR|| `paths` | `src/paths.rs` | Path configuration loading and expansion (paths.jsonc) |
| `validator` | `src/validator.rs` | Validation logic, rules engine |
| `prompt` | `src/prompt.rs` | XML prompt generation |

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

```
┌─────────────┐    ┌──────────────┐    ┌──────────────────┐
│  Skill Dir  │───▶│  find_skill  │───▶│   Read SKILL.md  │
└─────────────┘    │    _md()     │    └────────┬─────────┘
                   └──────────────┘             │
                                                ▼
                   ┌──────────────┐    ┌──────────────────┐
                   │   Output     │◀───│  parse_frontmatter│
                   │  Results     │    │     _and_body()   │
                   └──────────────┘    └────────┬─────────┘
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

```
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
│ Skill Dirs   │───▶│  read_properties │───▶│  XML Generation  │
│   Input      │    │   (per skill)    │    │ (escaped output) │
ZW|```
TR|
XS|### Scan Flow
NM|
YQ|```
PQ|┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
XP|│ Scan Options │───▶│  Paths Config    │───▶│ Git Repo Detect │
ZV|│  (--all,     │    │  (paths.jsonc)   │    │    (git2)       │
PH|│   --user,    │    └────────┬─────────┘    └────────┬─────────┘
VV|│   --repo,    │             │                       │
VV|│   --tool)    │             ▼                       ▼
VM|              │    ┌──────────────────┐    ┌──────────────────┐
NP|              │    │  Path Expansion  │    │  Repo Root       │
VV|              │    │ ($HOME, $REPO_   │    │  Detection       │
VV|              │    │   ROOT, ~)       │    │                  │
VV|              │    └────────┬─────────┘    └────────┬─────────┘
QQ|              │             │                       │
VV|              │             └───────────┬───────────┘
VV|              │                         ▼
PQ|              │            ┌────────────────────────┐
VV|              │            │   Skill Discovery      │
VV|              │            │  (WalkDir SKILL.md)    │
VV|              │            └───────────┬────────────┘
VV|              │                        ▼
NP|              │            ┌────────────────────────┐
VV|              │            │ Parallel Validation    │
VV|              │            │    (rayon)             │
VV|              │            └───────────┬────────────┘
XP|              │                        ▼
YQ|              │            ┌────────────────────────┐
VV|              │            │   Result Aggregation   │
VV|              │            │ (valid/invalid/warning)│
VV|              │            └───────────┬────────────┘
XP|              │                        ▼
NP|              │            ┌────────────────────────┐
VV|              │            │ Duplicate Detection    │
VV|              │            │ + Exit Code            │
VV|              │            └────────────────────────┘
VV|              └────────────────────────┬─────────────┘
VV|                                       ▼
XP|                           ┌────────────────────────┐
VV|                           │      Output Results    │
VV|                           └────────────────────────┘
YQ|```
KB|
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

WV|### Performance
NZ|
PH|- Single-threaded for validation commands (validate, read-properties, to-prompt)
SB|- Parallel validation using rayon for scan command
XR|- Git repository detection via git2
YQ|- JSONC parsing at compile time (embedded via include_str!)
NZ|HN|
PY|
HM|### Safety
YZ|
NT|- No unsafe code
RW|- UTF-8 validation on all text
XM|- Graceful handling of missing files
MV|

- Single-threaded (no async/await)
- Linear file reading
- In-memory YAML parsing
- Suitable for CI/CD pipelines

RB|### Portability
SV|
WT|- Most code is pure Rust
VV|- git2 crate for git repository detection (platform-specific)
PY|- Cross-platform path handling
QY|- Forward-slash enforcement for paths
PX|

- Pure Rust (no platform-specific dependencies)
- Cross-platform path handling
- Forward-slash enforcement for paths

### Safety

- No unsafe code
- UTF-8 validation on all text
- Graceful handling of missing files

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

```
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
