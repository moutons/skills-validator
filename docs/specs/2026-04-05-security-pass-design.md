# Security Pass Design (Pass 5)

**Date:** 2026-04-05
**Status:** Approved

## Overview

Pass 5 is the final stage of the five-pass validation pipeline. It performs
advisory security analysis on skill directories, scanning for remote code
execution patterns in prose and code blocks, and optionally running semgrep
static analysis on script files and extracted code blocks.

Entry point: `passes::security::run(skill_dir, ctx, config)` returns
`Result<Vec<Diagnostic>, PipelineError>`. The pass never halts the pipeline on
failure -- every error path emits a diagnostic and continues.

## Detection Strategy

### Remote Execution Patterns

Six regex patterns in `REMOTE_EXEC_PATTERNS` detect piping remote content into
a shell:

| Pattern                         | Matches                            |
|---------------------------------|------------------------------------|
| `curl\s+[^|]*\|\s*bash`        | `curl URL \| bash`                 |
| `curl\s+[^|]*\|\s*sh`          | `curl URL \| sh`                   |
| `wget\s+[^|]*\|\s*bash`        | `wget URL \| bash`                 |
| `wget\s+[^|]*\|\s*sh`          | `wget URL \| sh`                   |
| `bash\s+<\(\s*curl`            | `bash <(curl ...)`                 |
| `sh\s+<\(\s*curl`              | `sh <(curl ...)`                   |

`check_remote_execution_patterns` compiles all six, then scans two sources from
the `SkillContext`:

1. `ctx.prose_text` -- the concatenated prose of the skill markdown.
2. Each `ctx.code_blocks[].content` -- the body of every fenced code block.

Each match produces a `Diagnostic` with:

- **Severity:** `Warning` (advisory -- the skill may be documenting what NOT to do).
- **CheckName:** `RemoteExecutionPattern`.
- **Display:** match text truncated to 60 characters (`matched[..57] + "..."`).

### Semgrep Integration

When `config.security.semgrep_enabled` is true and the semgrep binary is found
via `which_semgrep`, the pass runs static analysis on all script files and
extracted code blocks.

Execution flow in `run_semgrep`:

1. Write all `BUNDLED_RULES` to a `tempfile::tempdir()`.
2. Create a second tempdir for code block extractions.
3. Collect paths of actual script files (`FileType::Script` from `ctx.file_inventory`).
4. Extract code blocks with recognized languages to temp files named
   `codeblock_{i}{ext}`, with `0o600` permissions on Unix.
5. Build a `std::process::Command`: `semgrep --json --config <rules_dir> [--config <custom_dir>] <files...>`.
6. Parse JSON output via `parse_semgrep_output`.

Exit code handling:

| Code  | Meaning           | Action                          |
|-------|-------------------|---------------------------------|
| 0     | No findings       | Parse output normally           |
| 1     | Findings present  | Parse output normally           |
| Other | Execution error   | Emit `SemgrepExecutionFailed`   |

Severity mapping in `parse_semgrep_output`:

| Semgrep severity | Maps to            |
|------------------|--------------------|
| `ERROR`          | `Severity::Error`  |
| `WARNING`        | `Severity::Warning`|
| `INFO`           | `Severity::Suggestion` |
| anything else    | `Severity::Warning`|

Each finding becomes a `Diagnostic` with `CheckName::RemoteExecutionPattern`,
`human_message` formatted as `[{check_id}] {message}`, and `file_path` from
the semgrep result.

## Bundled Rules

Five semgrep rule files are embedded at compile time via `include_str!` from
the `rules/` directory:

| File                     | Detects                                     |
|--------------------------|---------------------------------------------|
| `shell-injection.yaml`   | Shell command injection patterns            |
| `python-exec.yaml`       | Python `exec()`/`eval()` usage              |
| `env-exfiltration.yaml`  | Environment variable exfiltration           |
| `hardcoded-urls.yaml`    | Hardcoded URLs in scripts                   |
| `filesystem-escape.yaml` | Path traversal / filesystem escape attempts |

Stored in `BUNDLED_RULES: &[(&str, &str)]`. At runtime, each is written to a
tempdir so semgrep can read them as files on disk.

## Language Mapping

`lang_to_extension` maps code block language tags to file extensions for temp
file creation. Case-insensitive.

| Tags                          | Extension |
|-------------------------------|-----------|
| `python`, `py`                | `.py`     |
| `bash`, `sh`, `shell`, `zsh` | `.sh`     |
| `ruby`, `rb`                  | `.rb`     |
| `javascript`, `js`            | `.js`     |
| `typescript`, `ts`            | `.ts`     |

Unrecognized languages (e.g. `rust`, `go`) return `None` and are skipped.

## Configuration

All settings live in `SecurityConfig` (in `config.rs`), under the `[security]`
TOML section. Environment variable overrides are also supported.

| Field              | Type   | Default      | Env var                                     |
|--------------------|--------|--------------|---------------------------------------------|
| `semgrep_enabled`  | `bool` | `true`       | `SKILLS_VALIDATOR_SECURITY_SEMGREP_ENABLED`  |
| `semgrep_path`     | `String` | `"semgrep"` | `SKILLS_VALIDATOR_SECURITY_SEMGREP_PATH`   |
| `custom_rules_dir` | `String` | `""`        | `SKILLS_VALIDATOR_SECURITY_CUSTOM_RULES_DIR`|

`which_semgrep` resolves the binary: absolute paths are checked directly;
relative names are searched across `$PATH`.

`custom_rules_dir` is only passed to semgrep as an additional `--config` if the
directory exists on disk.

## Graceful Degradation

The pass degrades without failing the pipeline:

| Condition                        | Behavior                                                 |
|----------------------------------|----------------------------------------------------------|
| semgrep not found / disabled     | `emit_no_semgrep_diagnostics`: one `ScriptsDetectedNoSemgrep` (Suggestion), plus one `ScriptDetected` (Info) per script file |
| Tempdir creation fails           | `SemgrepExecutionFailed` (Warning), falls back to no-semgrep diagnostics |
| Rule file write fails            | `SemgrepExecutionFailed` (Warning), returns early        |
| Code block temp file write fails | `SemgrepExecutionFailed` (Warning) for that block, continues with remaining blocks |
| semgrep process fails to start   | `SemgrepExecutionFailed` (Warning), returns              |
| semgrep exits with code != 0,1   | `SemgrepExecutionFailed` (Warning), returns              |
| JSON parse fails                 | `SemgrepExecutionFailed` (Warning), returns              |
| No scannable files               | Returns silently (no diagnostics)                        |

## Security Considerations

- **Temp file permissions:** Code block temp files are created with mode `0o600`
  on Unix via `std::os::unix::fs::PermissionsExt`. This is behind `#[cfg(unix)]`.
- **Path handling:** `which_semgrep` handles both absolute and relative paths.
  Custom rules directories are only used if they exist on disk, preventing path
  injection via configuration.
- **No network access:** The pass itself makes no network calls. Semgrep is
  invoked locally against local files.
- **Temp cleanup:** Both tempdirs (`rules_dir`, `temp_dir`) are `tempfile::TempDir`
  instances that are dropped (and cleaned up) when `run_semgrep` returns.

## CheckName Variants

Security-specific variants in the `CheckName` enum:

| Variant                    | Serde name                      | Used by                      |
|----------------------------|---------------------------------|------------------------------|
| `RemoteExecutionPattern`   | `remote-execution-pattern`      | Pattern scan, semgrep findings |
| `ScriptsDetectedNoSemgrep` | `scripts-detected-no-semgrep`   | No-semgrep fallback          |
| `ScriptDetected`           | `script-detected`               | No-semgrep fallback          |
| `SemgrepExecutionFailed`   | `semgrep-execution-failed`      | All semgrep error paths      |
