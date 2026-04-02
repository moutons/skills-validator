# Testing Documentation

## Test Strategy

The skills-validator uses a comprehensive testing approach combining unit tests and integration tests.

---

## Test Organization

```text
tests/
├── helpers.rs              # Shared test utilities
├── fixtures_integration.rs # Tests using fixture files
├── cli_output.rs          # CLI output format validation
├── validator.rs           # Validation logic unit tests
├── parser.rs              # Parser unit tests
├── models.rs              # Data model tests
└── prompt.rs              # Prompt generation tests
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
// tests/fixtures_integration.rs
#[test]
fn test_valid_skill_directory() {
    let temp_dir = create_test_skill("valid-skill", r#"
---
name: valid-skill
description: A valid test skill
---

Always test your code.
Never skip validation.
"#);

    let result = validate(temp_dir.path());
    assert!(result.errors.is_empty());
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

### Prompt Tests (`tests/prompt.rs`)

- XML generation
- HTML escaping
- Multiple skill handling
- Error handling for invalid skills
- Empty input handling

### Integration Tests (`tests/fixtures_integration.rs`)

- End-to-end validation workflows
- Complex skill structures
- Edge cases
- Cross-platform compatibility

### CLI Tests (`tests/cli_output.rs`)

- Exit code verification
- Output format validation
- Log level filtering
- JSON output format
- Error message formatting

---

## Test Fixtures

### Fixture Directory Structure

```text
tests/fixtures/
├── valid/
│   └── basic/
│       └── SKILL.md
├── invalid/
│   ├── missing-name/
│   │   └── SKILL.md
│   └── bad-format/
│       └── SKILL.md
└── edge-cases/
    └── unicode-name/
        └── SKILL.md
```

### Using Fixtures

```rust
#[test]
fn test_with_fixture() {
    let fixture_path = Path::new("tests/fixtures/valid/basic");
    let result = validate(fixture_path);
    assert!(result.errors.is_empty());
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
