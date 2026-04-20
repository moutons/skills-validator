# API Reference

This document covers the v0.2.0 public API surface of `skills-validator`.

## Pipeline API

The pipeline API is the primary entry point for validation. It runs all five
passes (Parse, Structure, Content, References, Security) in sequence and
returns a structured result.

### `run_pipeline`

Run the full validation pipeline against a skill directory.

```rust
pub fn run_pipeline(skill_dir: &Path, config: &ValidatorConfig) -> PipelineResult
```

**Parameters:**

- `skill_dir: &Path` - Path to the skill directory containing `SKILL.md`
- `config: &ValidatorConfig` - Validator configuration (use `ValidatorConfig::default()` for defaults)

**Returns:** `PipelineResult`

**Behavior:**

- Pass 1 (Parse) is fatal: if SKILL.md cannot be found or parsed, returns immediately with an error diagnostic.
- Passes 2–5 run independently even if a prior pass emits diagnostics.

---

### `PipelineResult`

```rust
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub diagnostics: Vec<Diagnostic>,
    pub skill_name: Option<String>,
    pub sizeyness: Sizeyness,
    pub sizeyness_reasons: Vec<String>,
}
```

**Fields:**

- `diagnostics` - All diagnostics produced by all passes
- `skill_name` - Skill name extracted from frontmatter `name` field, if present
- `sizeyness` - Complexity tier: `Simple`, `Moderate`, or `Hefty`
- `sizeyness_reasons` - Human-readable reasons for the sizeyness classification (e.g. `["4 files", "1 subdirectory"]`)

---

### `exit_code`

Compute the appropriate process exit code from pipeline diagnostics.

```rust
pub fn exit_code(diagnostics: &[Diagnostic], strict: bool) -> i32
```

**Parameters:**

- `diagnostics` - Slice of diagnostics from `PipelineResult`
- `strict` - If `true`, treat `Warning` and `Suggestion` as failures

**Returns:**

- `1` if any `Error`-severity diagnostic exists
- `1` (strict mode only) if any `Warning` or `Suggestion` diagnostic exists
- `0` otherwise

---

### `Diagnostic`

A single validation finding.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub check_name: CheckName,
    pub human_message: String,
    pub machine_message: String,
    pub doc_url: Option<String>,
    pub file_path: Option<PathBuf>,
    pub base_severity: Severity,
}
```

**Fields:**

- `severity` - Effective severity (may be escalated from `base_severity`)
- `check_name` - Machine-readable check identifier (serializes as kebab-case)
- `human_message` - Warm, descriptive message for human-readable output
- `machine_message` - Terse message for JSON/machine output
- `doc_url` - Optional URL to documentation for this check
- `file_path` - Optional path to the file that triggered this diagnostic
- `base_severity` - Unescalated severity before any pipeline adjustments

---

### `Severity`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Suggestion,
    Warning,
    Error,
}
```

Severity levels in ascending order. `Severity` implements `PartialOrd`, so
`Info < Suggestion < Warning < Error`.

`Severity` also implements `FromStr` (accepts `"info"`, `"suggestion"`,
`"warning"`, `"error"`) and `Display`.

---

### `Sizeyness`

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sizeyness {
    #[default]
    Simple,
    Moderate,
    Hefty,
}
```

Complexity tier for a skill. Determined by file count, subdirectory count,
and presence of orchestration frontmatter fields (`hooks`, `agent`, `context`).

**Thresholds (defaults):**

| Tier     | Condition                                                  |
| -------- | ---------------------------------------------------------- |
| Simple   | < 3 files, 0 subdirectories, no orchestration fields       |
| Moderate | >= 3 files or >= 1 subdirectory                            |
| Hefty    | >= 6 files, >= 3 subdirectories, or has orchestration fields |

---

### `ValidatorConfig`

Top-level configuration struct. All fields have sane defaults and can be
overridden via config file, environment variables, or by constructing manually.

```rust
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ValidatorConfig {
    pub sizeyness: SizeynessConfig,
    pub content: ContentConfig,
    pub references: ReferencesConfig,
    pub security: SecurityConfig,
}
```

**Sub-structs and defaults:**

`SizeynessConfig`:

| Field                      | Default |
| -------------------------- | ------- |
| `moderate_file_threshold`  | `3`     |
| `hefty_file_threshold`     | `6`     |
| `moderate_subdir_threshold`| `1`     |
| `hefty_subdir_threshold`   | `3`     |

`ContentConfig`:

| Field             | Default                                                          |
| ----------------- | ---------------------------------------------------------------- |
| `body_line_limit` | `300`                                                            |
| `known_models`    | `["claude-opus-4-6", "claude-sonnet-4-6", "claude-haiku-4-5-20251001"]` |

`ReferencesConfig`:

| Field                 | Default                                                        |
| --------------------- | -------------------------------------------------------------- |
| `markdown_hop_limit`  | `5`                                                            |
| `orphan_exclusions`   | `["LICENSE*", "CHANGELOG*", "README*", ".gitignore", ".*"]`   |

`SecurityConfig`:

| Field                | Default    |
| -------------------- | ---------- |
| `semgrep_enabled`    | `true`     |
| `semgrep_path`       | `"semgrep"`|
| `custom_rules_dir`   | `""`       |

**Config loading:**

```rust
// Load from XDG config file + env var overrides
pub fn load() -> (ValidatorConfig, Vec<Diagnostic>)

// Load from a TOML string (useful for testing)
pub fn load_from_str(toml_str: &str) -> (ValidatorConfig, Vec<Diagnostic>)
```

**Config file location:** `$XDG_CONFIG_HOME/skills-validator/config.toml`

**Environment variable overrides** use the pattern
`SKILLS_VALIDATOR_<SECTION>_<KEY>` (uppercase). For example:
`SKILLS_VALIDATOR_CONTENT_BODY_LINE_LIMIT=500`.

---

## Formatter API

### `format_human`

Format pipeline results for human consumption.

```rust
pub fn format_human(result: &PipelineResult, skill_dir: &Path, min_severity: Severity) -> String
```

**Parameters:**

- `result` - Pipeline result to format
- `skill_dir` - Used to derive the skill label if `skill_name` is absent
- `min_severity` - Diagnostics below this severity are filtered out

**Output:** Grouped by severity (Info, Suggestion, Warning, Error) with a
summary line. Falls back to the directory name when `skill_name` is `None`.

---

### `format_json`

Format pipeline results as JSON.

```rust
pub fn format_json(
    result: &PipelineResult,
    skill_dir: &Path,
    min_severity: Severity,
    strict: bool,
) -> String
```

**Parameters:**

- `result` - Pipeline result to format
- `skill_dir` - Used to relativize file paths in diagnostics and as fallback skill label
- `min_severity` - Diagnostics below this severity are filtered from the output
- `strict` - Passed to `exit_code()` for the `exit_code` field in output

**Output schema (`schema_version: 2`):**

```json
{
  "schema_version": 2,
  "skill": "my-skill",
  "path": "/path/to/my-skill",
  "sizeyness": "moderate",
  "sizeyness_reasons": ["4 files", "1 subdirectory"],
  "diagnostics": [
    {
      "check": "binary-detected",
      "severity": "error",
      "message": "binary detected: lib/helper.so",
      "file": "lib/helper.so"
    }
  ],
  "summary": {
    "errors": 1,
    "warnings": 0,
    "suggestions": 1,
    "info": 2
  },
  "exit_code": 1
}
```

Notes:
- `diagnostics[].message` uses `machine_message`, not `human_message`
- `diagnostics[].file` is relative to `skill_dir`, omitted if absent
- `exit_code` is computed from all diagnostics (unfiltered by `min_severity`)

---

## Scan API

### `scan`

Scan one or more tool directories for skills and validate each one.

```rust
pub fn scan(options: &ScanOptions) -> ScanResult
```

Validation runs in parallel using Rayon. Uses `ValidatorConfig::default()`.

---

### `ScanOptions`

```rust
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    pub all: bool,
    pub user: bool,
    pub repo: bool,
    pub tools: Vec<String>,
    pub verbose: bool,
}
```

**Fields:**

- `all` - Scan all locations (user home + repo root)
- `user` - Scan only user home directories
- `repo` - Scan only the git repository root (requires a git repo)
- `tools` - Filter to specific tool names; empty means all tools
- `verbose` - Include verbose per-skill output (reserved for future use)

---

### `ScanResult`

```rust
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub skills: Vec<SkillValidation>,
    pub total_skills: usize,
    pub valid_count: usize,
    pub invalid_count: usize,
    pub warning_count: usize,
    pub scanned_dirs: Vec<PathBuf>,
    pub skipped_dirs: Vec<PathBuf>,
}
```

**Fields:**

- `skills` - All discovered skills with their validation results
- `total_skills` - Total number of skills found
- `valid_count` - Skills with no errors
- `invalid_count` - Skills with at least one error
- `warning_count` - Valid skills that have at least one warning
- `scanned_dirs` - Directories that were walked
- `skipped_dirs` - Directories that were skipped (do not exist or not accessible)

---

### `SkillValidation`

```rust
#[derive(Debug, Clone)]
pub struct SkillValidation {
    pub skill: DiscoveredSkill,
    pub validation: ValidationResult,    // deprecated
    pub pipeline_result: Option<PipelineResult>,
    pub is_valid: bool,
}
```

**Fields:**

- `skill` - The discovered skill metadata
- `validation` - Legacy `ValidationResult` (deprecated, for backward compatibility)
- `pipeline_result` - Full pipeline result; `None` only in error paths
- `is_valid` - `true` when no `Error`-severity diagnostics exist

---

### `find_duplicates`

Find skills with the same directory name appearing in more than one location.

```rust
pub fn find_duplicates(result: &ScanResult) -> Vec<Vec<&SkillValidation>>
```

Returns a list of groups; each group contains two or more `SkillValidation`
references sharing the same directory name.

---

### `discover_skills`

Discover all `SKILL.md` files under a set of directories.

```rust
pub fn discover_skills(directories: &[(String, PathBuf)]) -> DiscoveryResult
```

**Parameters:**

- `directories` - Slice of `(tool_name, expanded_path)` tuples

Walks each directory recursively. Directories that do not exist are recorded
in `DiscoveryResult::skipped_dirs` rather than returning an error.

---

### `DiscoveredSkill`

```rust
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    pub path: PathBuf,         // Path to SKILL.md
    pub tool_name: String,     // Tool this skill belongs to
    pub directory: PathBuf,    // Parent directory (skill root)
}
```

---

### `DiscoveryResult`

```rust
#[derive(Debug, Clone, Default)]
pub struct DiscoveryResult {
    pub skills: Vec<DiscoveredSkill>,
    pub skipped_dirs: Vec<PathBuf>,
}
```

---

## Parser API

### `find_skill_md`

Find `SKILL.md` with exact casing enforcement.

```rust
pub fn find_skill_md(skill_dir: &Path) -> Option<PathBuf>
```

**Parameters:**

- `skill_dir: &Path` - Path to the skill directory

**Returns:**

- `Some(PathBuf)` - Path to the `SKILL.md` file
- `None` - No file named exactly `SKILL.md` exists

**Important:** This function enforces exact `SKILL.md` casing. On
case-insensitive filesystems (macOS), it verifies the casing by reading the
directory listing. It does **not** fall back to `skill.md` or other casings.

---

### `parse_frontmatter`

Parse YAML frontmatter from `SKILL.md` content.

```rust
pub fn parse_frontmatter(content: &str) -> Result<(serde_yaml::Value, String), SkillError>
```

**Parameters:**

- `content: &str` - Full content of `SKILL.md`

**Returns:**

- `Ok((metadata, body))` - Parsed YAML value and body text (trimmed)
- `Err(SkillError)` - Parse error with message

**Errors:**

- Content does not start with `---`
- Frontmatter is not properly closed with `---`
- Invalid YAML syntax
- Frontmatter is not a YAML mapping

---

### `read_properties`

Read and validate skill properties from a directory.

```rust
pub fn read_properties(skill_dir: &Path) -> Result<SkillProperties, SkillError>
```

**Parameters:**

- `skill_dir: &Path` - Path to skill directory

**Returns:**

- `Ok(SkillProperties)` - Parsed and validated properties
- `Err(SkillError)` - If `SKILL.md` is missing, unreadable, or invalid

**Required frontmatter fields:** `name`, `description` (both must be non-empty strings)

**Optional frontmatter fields:** `license`, `compatibility`, `allowed-tools`, `metadata`

---

## Prompt API

### `to_prompt`

Generate `<available_skills>` XML for agent prompts.

```rust
pub fn to_prompt(skill_dirs: &[&str]) -> String
```

**Parameters:**

- `skill_dirs: &[&str]` - Slice of skill directory paths

**Returns:** XML string with HTML-escaped content.

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

- HTML-escapes `name` and `description`
- Skips skills that fail to read; logs failures to stderr

---

## Infrastructure API

### `find_repo_root`

Find the root of the git repository containing the given path.

```rust
pub fn find_repo_root(start: Option<&Path>) -> Result<PathBuf, GitError>
```

**Parameters:**

- `start` - Starting path for repository discovery; uses `$CWD` if `None`

**Returns:**

- `Ok(PathBuf)` - Absolute path to the repository working directory
- `Err(GitError)` - If no git repository is found

---

### `GitError`

```rust
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    LibError(#[from] git2::Error),
}
```

---

### `expand_path`

Expand path variables in a template string.

```rust
pub fn expand_path(template: &str, repo_root: Option<&PathBuf>) -> Result<PathBuf, PathsError>
```

**Supported variables:**

| Variable      | Expands to                        |
| ------------- | --------------------------------- |
| `~`           | User home directory               |
| `$HOME`       | User home directory               |
| `$REPO_ROOT`  | Git repository root (if provided) |
| `$CWD`        | Current working directory         |

---

### `PathsConfig`

Configuration containing tool directory templates, loaded from the embedded
`paths.jsonc`.

```rust
pub struct PathsConfig {
    pub tools: HashMap<String, ToolConfig>,
}

impl PathsConfig {
    pub fn load() -> Result<Self, PathsError>;
    pub fn get_tool(&self, name: &str) -> Option<&ToolConfig>;
    pub fn tool_names(&self) -> Vec<String>;
    pub fn has_tool(&self, name: &str) -> bool;
}
```

Tool names are normalized to kebab-case. `get_tool` and `has_tool` accept
any casing.

---

### `PathsError`

```rust
#[derive(Debug, Error)]
pub enum PathsError {
    #[error("Failed to parse paths.jsonc: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Home directory not found")]
    HomeNotFound,

    #[error("Repository root not provided but required by path template")]
    RepoRootNotProvided,

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}
```

---

## Legacy API

The following items are **deprecated** as of v0.2.0. They remain for backward
compatibility but will be removed in a future release. Use the Pipeline API
instead.

### `validate` (deprecated)

```rust
#[deprecated]
pub fn validate(skill_dir: &Path) -> ValidationResult
```

Use `run_pipeline(skill_dir, &ValidatorConfig::default())` instead.

---

### `ValidationResult` (deprecated)

```rust
#[deprecated]
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

Use `PipelineResult` and `Diagnostic` instead.

---

## CLI API

### Global Options

| Option             | Short | Description                                               |
| ------------------ | ----- | --------------------------------------------------------- |
| `--log-level`      | `-l`  | Set log level: `error`, `warn`, `info`, `debug` (default: `info`) |
| `--output-format`  |       | Output format for `validate`: `human` (default) or `json` |
| `--severity`       |       | Minimum severity to display: `info` (default), `suggestion`, `warning`, `error` |
| `--strict`         |       | Promote warnings and suggestions to exit code 1           |
| `--json`           |       | **Deprecated.** Alias for `--output-format json`. Previously wrote JSON log lines to stderr; now writes structured JSON to stdout. |

---

### `validate`

Validate a skill directory against the Agent Skills spec.

```bash
skills-validator [OPTIONS] validate <PATH>
```

**Arguments:**

- `PATH` - Path to skill directory

**Exit codes:**

- `0` - Valid (no errors; warnings are present only if `--strict` is not set)
- `1` - Invalid (errors present, or warnings/suggestions present with `--strict`)
- `2` - Scan/config error (could not load configuration)

**Output:**

- stdout: Validation result (human text or JSON, controlled by `--output-format`)
- stderr: Log messages

**Examples:**

```bash
# Human output (default)
skills-validator validate ~/.claude/skills/my-skill

# JSON output for CI
skills-validator --output-format json validate ~/.claude/skills/my-skill

# Fail on warnings too
skills-validator --strict validate ~/.claude/skills/my-skill

# Show only errors and warnings
skills-validator --severity warning validate ~/.claude/skills/my-skill
```

---

### `scan`

Scan for skills across multiple tool directories.

```bash
skills-validator scan [--all | --user | --repo] [--tool <NAMES>] [--dry-run] [--verbose]
```

**Scope flags (mutually exclusive):**

| Flag     | Description                                     |
| -------- | ----------------------------------------------- |
| `--all`  | Scan all locations (user home + repo root)      |
| `--user` | Scan `$HOME`-based tool directories             |
| `--repo` | Scan repo-root-based directories (requires git) |

**Other flags:**

| Flag          | Description                                     |
| ------------- | ----------------------------------------------- |
| `--tool <NAMES>` | Comma-separated tool names to scan           |
| `--dry-run`   | Print what would be scanned without validating  |
| `--verbose`   | Show detailed output per skill                  |

**Exit codes:**

- `0` - All discovered skills are valid
- `1` - At least one skill is invalid
- `2` - Could not load paths configuration or unknown tool specified

---

### `read-properties`

Parse and output skill frontmatter as YAML.

```bash
skills-validator read-properties <PATH>
```

**Output (stdout):** YAML-formatted properties

```yaml
name: my-skill
description: What this skill does
license: Apache-2.0
metadata:
  author: Jane Doe
```

---

### `to-prompt`

Generate `<available_skills>` XML for agent prompts.

```bash
skills-validator to-prompt <PATH>...
```

**Arguments:**

- `PATH...` - One or more skill directory paths (glob-friendly)

---

### `setup`

Write a commented default config file to `$XDG_CONFIG_HOME/skills-validator/config.toml`.

```bash
skills-validator setup
```

Exits with an error if the file already exists. Remove it first to regenerate.

---

### `completions`

Generate shell completion scripts.

```bash
skills-validator completions <SHELL>
```

**Arguments:**

- `SHELL` - One of: `bash`, `zsh`, `fish`, `elvish`, `powershell`

---

### Output Streams

| Stream | Content                                          |
| ------ | ------------------------------------------------ |
| stdout | Data/results (validation output, YAML, XML, JSON)|
| stderr | Log messages (INFO, WARN, DEBUG, errors)         |

---

## Usage Examples

### Library Usage

```rust
use skills_validator::{run_pipeline, format_human, format_json};
use skills_validator::{ValidatorConfig, Severity};
use std::path::Path;

// Load config (reads XDG config file + env vars)
let (config, config_diags) = skills_validator::config::load();

// Validate a skill
let skill_dir = Path::new("my-skill");
let result = run_pipeline(skill_dir, &config);

// Human-readable output
let human = format_human(&result, skill_dir, Severity::Info);
print!("{}", human);

// JSON output (e.g., for CI)
let json = format_json(&result, skill_dir, Severity::Info, false);
println!("{}", json);

// Determine exit code
let code = skills_validator::pipeline::exit_code(&result.diagnostics, false);
std::process::exit(code);
```

### CLI Usage

```bash
# Validate with default human output
skills-validator validate ./my-skill

# Validate and output structured JSON
skills-validator --output-format json validate ./my-skill

# Strict mode: warnings are failures
skills-validator --strict validate ./my-skill

# Scan all known tool directories
skills-validator scan --all

# Scan only Claude Code skills in the current repo
skills-validator scan --repo --tool claude-code

# Read frontmatter properties
skills-validator read-properties ~/.claude/skills/my-skill

# Generate prompt XML for multiple skills
skills-validator to-prompt ~/.claude/skills/*

# Write default config
skills-validator setup

# Generate zsh completions
skills-validator completions zsh > ~/.zfunc/_skills-validator
```
