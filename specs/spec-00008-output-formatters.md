# Output Formatters

## Goal

Format scan results for both human-readable (terminal) and machine-readable (JSON) output.

## Context

The scan command outputs results in two formats. Human-readable uses the same style as the existing `validate` command. JSON includes a summary section.

**Human-readable format (per skill):**

```text
07:57:31 WARN Field 'argument-hint' is a Claude Code extension...
07:57:31 WARN 'example' not found in skill content...
✓ Skill is valid (with warnings)
```

**JSON format:**

```json
{
  "summary": {
    "total": 10,
    "valid": 8,
    "invalid": 2,
    "warnings": 5,
    "duplicates": 1
  },
  "skills": [
    {"path": "...", "valid": true, "errors": [], "warnings": [...]},
    ...
  ],
  "duplicates": [...]
}
```

**Function signature:**

```rust
pub fn format_human(result: &ScanResult, verbose: bool) -> String;
pub fn format_json(result: &ScanResult) -> Result<String, serde_json::Error>;
```

## User Stories

**US-001:** Human output matches validate command As a user, I want scan output to look consistent with `validate` command output.

**US-002:** JSON includes summary As a CI pipeline, I want JSON output with counts for programmatic processing.

**US-003:** Verbose shows all details As a user, I want `--verbose` to show expanded paths and discovery details.

**US-004:** Non-verbose shows summary only As a user, without `--verbose` I want only the overall summary and error list.

## Acceptance Criteria

- [ ] `format_human()` produces output matching existing validate style
- [ ] `format_json()` produces valid JSON with summary and skills array
- [ ] `--verbose` includes discovery details (paths expanded, dirs skipped)
- [ ] Without `--verbose`, only summary counts and errors shown
- [ ] Colors work consistently with existing terminal output

## Completion Signal
