# Testing Documentation

## Test Strategy

The skills-validator uses a comprehensive testing approach combining unit tests and integration tests.

---

## Test Organization

```text
tests/
├── helpers.rs              # Shared test utilities
├── cli_output.rs           # CLI output format tests
├── config.rs               # Config system tests
├── fixtures_integration.rs # Tests with fixture files
├── formatter.rs            # Formatter output tests
├── models.rs               # Data model tests
├── parser.rs               # Parser tests
├── passes_content.rs       # Pass 3: Content checks
├── passes_parse.rs         # Pass 1: Parse checks
├── passes_references.rs    # Pass 4: Reference checks
├── passes_security.rs      # Pass 5: Security checks
├── passes_structure.rs     # Pass 2: Structure checks
├── pipeline.rs             # Full pipeline integration tests
├── prompt.rs               # Prompt generation tests
└── validator.rs            # Legacy validator tests
```

---

## Test Types

### Unit Tests

Tests for individual functions in isolation.

**Location:** Inline in source files with `#[cfg(test)]` or in `tests/` directory

**Example:**

```rust
// tests/validator.rs
#[test]
fn test_validate_name_valid() {
    let result = validate_name("my-skill", None);
    assert!(result.errors.is_empty());
}

#[test]
fn test_validate_name_uppercase() {
    let result = validate_name("My-Skill", None);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].contains("must be lowercase"));
}
```

### Integration Tests

Tests for complete workflows and CLI behavior.

**Location:** `tests/` directory

**Example:**

```rust
// tests/pipeline.rs
#[test]
fn valid_minimal_skill_produces_no_errors() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty());
}
```

### CLI Output Tests

Tests for command-line interface behavior and output formats.

**Location:** `tests/cli_output.rs`

**Coverage:**

- Exit codes
- Output format (text vs JSON)
- Error messages
- Warning display

---

## Test Utilities

### helpers.rs

Provides shared utilities for test setup:

```rust
// Create a temporary skill directory
pub fn create_test_skill(name: &str, content: &str) -> TempDir {
    let dir = tempdir().unwrap();
    let skill_md = dir.path().join("SKILL.md");
    fs::write(&skill_md, content).unwrap();
    dir
}

// Create skill with additional files
pub fn create_test_skill_with_files(
    name: &str,
    skill_content: &str,
    files: Vec<(&str, &str)>
) -> TempDir {
    // implementation...
}
```

---

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Test

```bash
cargo test test_name
cargo test test_validate_name
```

### Test Module

```bash
cargo test validator::
cargo test parser::
```

### With Output

```bash
cargo test -- --nocapture
cargo test -- --show-output
```

### Filtered

```bash
cargo test validate -- --nocapture
cargo test --test validator
```

---

## Test Coverage Areas

### Config Tests (`tests/config.rs`)

- Default configuration values (thresholds, limits)
- TOML configuration parsing
- Environment variable overrides
- Known models list validation
- Orphan exclusion patterns

### Validator Tests (`tests/validator.rs`)

- Name validation (format, length, characters)
- Description validation (empty, length)
- Compatibility validation (length)
- Metadata field validation
- Unknown field detection
- Claude Code extension warnings
- Content keyword detection
- Body length warnings
- Windows path detection
- Script organization warnings

### Parser Tests (`tests/parser.rs`)

- Frontmatter parsing (valid YAML)
- Frontmatter edge cases (missing markers, invalid YAML)
- File discovery (SKILL.md vs skill.md)
- Property extraction
- Error handling

### Model Tests (`tests/models.rs`)

- Serialization/deserialization
- `to_dict()` method
- Optional field handling
- Metadata mapping

### Formatter Tests (`tests/formatter.rs`)

- Human output with emoji markers (folder icon, severity icons)
- Severity-based grouping and filtering
- JSON output with schema_version field
- Severity filtering in output
- Summary line generation

### Prompt Tests (`tests/prompt.rs`)

- XML generation
- HTML escaping
- Multiple skill handling
- Error handling for invalid skills
- Empty input handling

### Pipeline Tests (`tests/pipeline.rs`)

- Full pipeline on valid skills (no errors produced)
- Pipeline orchestration across all five passes
- Parse failures stop pipeline processing
- Sizeyness escalation (Simple, Moderate, Hefty)
- Strict mode enforcement

### Pass 1 (Parse) Tests (`tests/passes_parse.rs`)

- Exact SKILL.md casing requirement
- Frontmatter extraction and TOML validation
- Abstract syntax tree (AST) extraction for structure
- Prose-only view generation (without code blocks)

### Pass 2 (Structure) Tests (`tests/passes_structure.rs`)

- File inventory and statistics
- Binary file detection
- Sizeyness boundaries and calculation
- Subdirectory counting logic

### Pass 3 (Content) Tests (`tests/passes_content.rs`)

- Description length validation
- Trigger language detection and validation
- Word-boundary matching for keywords
- Unknown field detection

### Pass 4 (References) Tests (`tests/passes_references.rs`)

- Link extraction from markdown
- Broken reference detection
- Orphan file detection
- Circular reference detection
- Markdown hop limit enforcement
- File path boundary validation

### Pass 5 (Security) Tests (`tests/passes_security.rs`)

- Script detection and flagging
- Remote execution pattern detection
- Code block extraction and analysis
- Semgrep integration (when enabled)

### Integration Tests (`tests/fixtures_integration.rs`)

- End-to-end validation workflows
- Complex skill structures
- Edge cases
- Cross-platform compatibility

### CLI Tests (`tests/cli_output.rs`)

- Exit code verification
- Output format validation (text vs JSON)
- --strict flag behavior
- --output-format flag (human vs json)
- --severity flag filtering
- Error message formatting

---

## Test Fixtures

### Fixture Directory Structure

```text
tests/fixtures/
├── valid-skill/              # Simple valid skill
├── invalid-name/             # Invalid name format
├── missing-description/      # Missing required field
└── skills/
    ├── valid/                # Valid skill variants
    │   ├── minimal/          # Minimal valid skills
    │   ├── complete/         # Full-featured skills
    │   └── multi-file/       # Multi-file skills
    ├── invalid/              # Invalid skill variants
    │   ├── invalid-name/     # Invalid name format
    │   ├── missing-frontmatter/
    │   ├── missing-name/     # Missing required name field
    │   ├── malformed-toml/   # Invalid TOML syntax
    │   └── unknown-fields/   # Unrecognized metadata
    ├── edge-cases/           # Edge case testing
    │   ├── empty-optional-fields/
    │   ├── large-file/       # Large file sizeyness testing
    │   └── unicode-content/  # Unicode handling
    ├── multi-location/       # Multiple skill locations
    ├── binary-in-skill/      # Binary file detection
    ├── broken-ref/           # Broken reference chain
    ├── circular-ref/         # Circular reference (A→B→A)
    └── orphaned-files/       # Unreferenced files
```

### Using Fixtures

```rust
#[test]
fn test_with_fixture() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);
    assert!(result.diagnostics.is_empty());
}

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}
```

---

## Writing New Tests

### Test Naming Convention

```rust
// Pattern: test_<function>_<scenario>_<expected>
#[test]
fn test_validate_name_uppercase_error()
#[test]
fn test_parse_frontmatter_missing_delimiter_error()
#[test]
fn test_to_prompt_empty_skills_empty_xml()
```

### Test Structure

```rust
#[test]
fn test_feature_scenario() {
    // Arrange
    let input = setup_test_data();

    // Act
    let result = function_under_test(input);

    // Assert
    assert_eq!(result, expected);
}
```

### Temporary Files

Always use `tempfile` crate for temporary directories:

```rust
use tempfile::tempdir;
use std::fs;

#[test]
fn test_with_temp_files() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("SKILL.md");
    fs::write(&file_path, "content").unwrap();

    // test using file_path...

    // dir is automatically cleaned up when it goes out of scope
}
```

---

## Continuous Integration

Tests run automatically on:

- Pull requests
- Pushes to main
- Releases

See `.github/workflows/ci.yml` for CI configuration.

---

## Debugging Failed Tests

### View Full Output

```bash
cargo test -- --nocapture
```

### Single Test with Output

```bash
cargo test test_name -- --exact --nocapture
```

### Backtrace on Panic

```bash
RUST_BACKTRACE=1 cargo test
```

### Verbose Test Output

```bash
cargo test -- --test-threads=1 --nocapture
```

---

## Best Practices

1. **Test one thing per test** - Keep tests focused
2. **Use descriptive names** - Name describes what's being tested
3. **Arrange-Act-Assert** - Clear test structure
4. **Clean up** - Use tempdir for file operations
5. **Test edge cases** - Empty inputs, max lengths, special characters
6. **Test errors** - Verify error messages are helpful
7. **Keep tests fast** - Avoid I/O when possible
