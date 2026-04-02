# API Reference

## Rust Library API

### Module: `parser`

#### `find_skill_md`

Finds the SKILL.md file in a skill directory.

```rust
pub fn find_skill_md(skill_dir: &Path) -> Option<std::path::PathBuf>
```

**Parameters:**
- `skill_dir: &Path` - Path to the skill directory

**Returns:**
- `Some(PathBuf)` - Path to SKILL.md or skill.md
- `None` - If neither file exists

**Behavior:**
- Checks for `SKILL.md` first (preferred)
- Falls back to `skill.md` (lowercase)
- Returns canonical path

---

#### `parse_frontmatter`

Parses YAML frontmatter from SKILL.md content.

```rust
pub fn parse_frontmatter(content: &str) -> Result<(serde_yaml::Value, String), SkillError>
```

**Parameters:**
- `content: &str` - Full content of SKILL.md

**Returns:**
- `Ok((metadata, body))` - Parsed YAML and body content
- `Err(SkillError)` - Parse error with message

**Errors:**
- Content doesn't start with `---`
- Frontmatter not properly closed
- Invalid YAML syntax
- Metadata is not a YAML mapping

---

#### `parse_frontmatter_and_body`

Convenience function returning a YAML mapping directly.

```rust
pub fn parse_frontmatter_and_body(content: &str) -> Result<(serde_yaml::Mapping, String), SkillError>
```

**Returns:**
- `Ok((map, body))` - YAML mapping and body content
- `Err(SkillError)` - If metadata is not a valid mapping

---

#### `read_properties`

Reads and validates skill properties from a directory.

```rust
pub fn read_properties(skill_dir: &Path) -> Result<SkillProperties, SkillError>
```

**Parameters:**
- `skill_dir: &Path` - Path to skill directory

**Returns:**
- `Ok(SkillProperties)` - Parsed properties
- `Err(SkillError)` - If SKILL.md missing or invalid

**Required fields:**
- `name` - Non-empty string
- `description` - Non-empty string

**Optional fields:**
- `license`
- `compatibility`
- `allowed-tools`
- `metadata` - HashMap<String, String>

---

### Module: `validator`

#### `validate`

Main validation function for a skill directory.

```rust
pub fn validate(skill_dir: &Path) -> ValidationResult
```

**Parameters:**
- `skill_dir: &Path` - Path to skill directory

**Returns:** `ValidationResult` with:
- `errors: Vec<String>` - Validation errors
- `warnings: Vec<String>` - Validation warnings

**Validation checks:**
1. Directory exists
2. SKILL.md exists
3. Frontmatter parses correctly
4. Required fields present (name, description)
5. Name format validation (regex, length, case)
6. Name matches directory name
7. Description length (max 1024 chars)
8. Compatibility length (max 500 chars)
9. Unknown fields detection
10. Claude Code extension warnings
11. Content keywords (never, always, when, example)
12. Body length (max 500 lines warning)
13. Windows path detection
14. Scripts in root directory

---

#### `validate_metadata`

Validates frontmatter metadata fields.

```rust
pub fn validate_metadata(metadata: &serde_yaml::Mapping, skill_dir: Option<&Path>) -> ValidationResult
```

**Parameters:**
- `metadata: &serde_yaml::Mapping` - Parsed frontmatter
- `skill_dir: Option<&Path>` - Optional directory for name matching

**Returns:** `ValidationResult` with errors and warnings

---

#### `ValidationResult`

Result container for validation.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self;
}

impl Default for ValidationResult;
```

**Methods:**
- `new()` - Creates empty result
- `default()` - Same as new()

**Fields:**
- `errors` - Fatal validation errors (block validation)
- `warnings` - Non-fatal issues (don't block validation)

---

### Module: `prompt`

#### `to_prompt`

Generates `<available_skills>` XML block for agent prompts.

```rust
pub fn to_prompt(skill_dirs: &[&str]) -> String
```

**Parameters:**
- `skill_dirs: &[&str]` - Slice of skill directory paths

**Returns:**
- XML string with escaped content

**Output format:**
```xml
<available_skills>
<skill>
<name>
my-skill
</name>
<description>
What this skill does...
</description>
<location>
/path/to/my-skill/SKILL.md
</location>
</skill>
</available_skills>
```

**Notes:**
- HTML-escapes name and description
- Skips skills that fail to read
- Outputs to stderr on read failures

---

### Module: `models`

#### `SkillProperties`

Data structure for skill metadata.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProperties {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub allowed_tools: Option<String>,
    pub metadata: HashMap<String, String>,
}
```

**Fields:**
- `name` - Skill identifier (required)
- `description` - Skill description (required)
- `license` - License identifier (optional)
- `compatibility` - Version requirements (optional)
- `allowed_tools` - Pre-approved tools (optional)
- `metadata` - Additional key-value pairs (optional)

**Methods:**

```rust
impl SkillProperties {
    /// Convert to YAML mapping for serialization
    pub fn to_dict(&self) -> serde_yaml::Value;
}
```

**Serialization:**
- Uses `#[serde(skip_serializing_if = "Option::is_none")]` for optional fields
- Metadata serializes as nested mapping

---

### Module: `error`

#### `SkillError`

Error types for the library.

```rust
#[derive(Error, Debug)]
pub enum SkillError {
    #[error("Failed to parse SKILL.md: {0}")]
    ParseError(String),

    #[error("Skill validation failed: {0}")]
    ValidationError(String),
}
```

**Variants:**
- `ParseError` - YAML parsing, file I/O errors
- `ValidationError` - Validation rule violations

**Conversions:**
- `From<std::io::Error>` - I/O errors become ParseError
- `From<serde_yaml::Error>` - YAML errors become ParseError

---

## CLI API

### Commands

#### `validate`

Validate a skill directory.

```bash
skills-validator validate <PATH>
```

**Arguments:**
- `PATH` - Path to skill directory

**Exit codes:**
- `0` - Valid (may have warnings)
- `1` - Invalid (errors present)

**Output:**
- stdout: Validation result message
- stderr: Log messages and errors

---

#### `read-properties`

Parse and output skill properties as YAML.

```bash
skills-validator read-properties <PATH>
```

**Arguments:**
- `PATH` - Path to skill directory

**Output:**
- stdout: YAML formatted properties
- stderr: Log messages

**Example output:**
```yaml
name: my-skill
description: What this skill does
license: Apache-2.0
metadata:
  author: John Doe
```

---

#### `to-prompt`

Generate `<available_skills>` XML for agent prompts.

```bash
skills-validator to-prompt <PATH>...
```

**Arguments:**
- `PATH...` - One or more skill directory paths

**Output:**
- stdout: XML block
- stderr: Warnings for unreadable skills

---

### Global Options

| Option | Short | Description |
|--------|-------|-------------|
| `--log-level` | `-l` | Set log level: error, warn, info, debug (default: info) |
| `--json` | | Output logs as JSON to stderr |

**Log levels:**
- `error` - Show only errors
- `warn` - Show warnings and errors (default)
- `info` - Show informational messages
- `debug` - Show detailed debug info

---

### Output Streams

| Stream | Content |
|--------|---------|
| stdout | Data/results (YAML, XML, validation results) |
| stderr | All log messages (INFO, WARN, DEBUG, errors) |

**Best practice:**
- Parse stdout for programmatic use
- Read stderr for diagnostics

---

## Usage Examples

### Library Usage

```rust
use skills_validator::{validate, read_properties, to_prompt};
use std::path::Path;

// Validate a skill
let result = validate(Path::new("my-skill"));
if !result.errors.is_empty() {
    println!("Errors: {:?}", result.errors);
}
for warning in &result.warnings {
    println!("Warning: {}", warning);
}

// Read properties
let props = read_properties(Path::new("my-skill")).unwrap();
println!("{}: {}", props.name, props.description);

// Generate prompt
let prompt = to_prompt(&["skill-a", "skill-b"]);
println!("{}", prompt);
```

### CLI Usage

```bash
# Validate with verbose logging
skills-validator -l debug validate ./my-skill

# Output as JSON for CI
skills-validator --json validate ./my-skill

# Read properties
skills-validator read-properties ~/.agents/skills/rust

# Generate prompt for multiple skills
skills-validator to-prompt ~/.agents/skills/*
```
