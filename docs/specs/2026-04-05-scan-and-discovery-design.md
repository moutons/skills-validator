# Scan and Discovery System Design

**Date:** 2026-04-05 **Status:** Approved

## Overview

The scan and discovery system is the entry point for skills-validator. It resolves agent tool directories from an embedded configuration file, walks those directories to find `SKILL.md` files, validates each discovered skill through the pipeline, and
reports results with appropriate exit codes. The design separates path resolution, discovery, and validation into distinct phases to keep each concern testable in isolation.

## Architecture

Four modules collaborate in sequence:

1. **`paths.rs`** -- loads and resolves tool directory templates from the embedded `paths.jsonc`.
2. **`discovery.rs`** -- walks resolved directories to find `SKILL.md` files.
3. **`scan.rs`** -- orchestrates the full flow: load config, resolve dirs, discover, validate in parallel, aggregate results.
4. **`pipeline.rs`** -- runs the five-pass validation pipeline on each discovered skill (documented separately).

Data flows linearly: `PathsConfig` -> `Vec<(String, PathBuf)>` -> `DiscoveryResult` -> parallel `run_pipeline()` -> `ScanResult`.

## Path Configuration

### Embedding

`paths.jsonc` is embedded at compile time via `include_str!("../paths.jsonc")` into the constant `PATHS_JSONC` in `paths.rs`. This means the tool registry is baked into the binary with zero runtime file I/O.

The file defines 33 tools across supported and `_unsupported` sections. The `_unsupported` key is explicitly skipped during `PathsConfig::load()`.

### JSONC Comment Stripping

`strip_json_comments()` is a hand-rolled character-level parser that handles:

- `//` single-line comments (consumed to end of line)
- `/* */` block comments (consumed including nesting delimiters)
- String-awareness: comments inside `"..."` are preserved as literal content
- Escape-awareness: `\"` inside strings does not toggle the in-string flag

The stripped output is fed to `serde_json::from_str`.

### Variable Expansion

`expand_path()` replaces four variables in directory templates:

| Variable     | Source                          | Required context  |
| ------------ | ------------------------------- | ----------------- |
| `$HOME`      | `dirs::home_dir()`              | Always available  |
| `~`          | `dirs::home_dir()` (alias)      | Always available  |
| `$REPO_ROOT` | `find_repo_root()` from git mod | `repo_root` param |
| `$CWD`       | `std::env::current_dir()`       | Always available  |

If `$REPO_ROOT` is referenced but no repo root is provided, `expand_path` returns `Err(PathsError::RepoRootNotProvided)`.

### Tool Lookup

Each tool entry is stored in a `HashMap<String, ToolConfig>` keyed by kebab-case name. `to_kebab_case()` normalizes `CamelCase`, `snake_case`, and `UPPER-KEBAB` inputs so that `get_tool()` is effectively case-insensitive. A `ToolConfig` contains
`name` (display), `documentation` (optional URL), and `directories` (template strings).

## Discovery Process

`discover_skills()` in `discovery.rs` accepts a slice of `(tool_name, expanded_path)` tuples. For each tuple:

1. If the path does not exist, it is added to `DiscoveryResult::skipped_dirs` and skipped.
2. Otherwise, `WalkDir` recursively walks the directory (following symlinks).
3. Any file named exactly `SKILL.md` produces a `DiscoveredSkill` with:
   - `path`: full path to the `SKILL.md` file
   - `tool_name`: which agent tool owns this directory
   - `directory`: parent directory of the `SKILL.md` (the skill root)

Discovery is sequential -- it runs before validation begins.

## Validation Strategy

### Parallel Execution

After discovery completes, `scan()` validates all skills in parallel using rayon's `into_par_iter()`. Each skill is passed to `run_pipeline()` which returns a `PipelineResult` containing a list of `Diagnostic` values with severity levels.

### Validity Definition

A skill is valid if it has **no `Severity::Error` diagnostics**. Warnings and suggestions do not make a skill invalid. This check is performed inline during the parallel map:

```rust
let has_errors = pipeline_result.diagnostics.iter()
    .any(|d| d.severity == Severity::Error);
```

### Legacy Compatibility

Each `SkillValidation` carries both a `PipelineResult` (new) and a `ValidationResult` (legacy, `#[deprecated]`). The legacy struct is built by partitioning diagnostics into `errors` (Error severity) and `warnings` (Warning severity) string vectors.
This preserves backward compatibility with output formatters that predate the pipeline.

### Strict Mode

The `exit_code()` function in `pipeline.rs` supports a `strict` flag. In strict mode, warnings and suggestions are promoted to exit code 1 alongside errors.

## Duplicate Detection

`find_duplicates()` groups all `SkillValidation` entries by their directory's final path component (via `Path::file_name()`). Any group with more than one entry is returned as a duplicate set. This catches the same skill name appearing under both
user-home and repo-root paths.

```rust
let name = skill.skill.directory.file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_else(|| "unknown".to_string());
```

## Scan Modes

`ScanOptions` controls which directories are resolved:

| Field     | Type          | Effect                                                     |
| --------- | ------------- | ---------------------------------------------------------- |
| `all`     | `bool`        | Scan both `$HOME`-based and `$REPO_ROOT`-based directories |
| `user`    | `bool`        | Scan only directories containing `$HOME` or `~`            |
| `repo`    | `bool`        | Scan only directories containing `$REPO_ROOT`              |
| `tools`   | `Vec<String>` | Filter to specific tool names (applied on top of mode)     |
| `verbose` | `bool`        | Reserved for verbose output (currently unused)             |

When `tools` is non-empty, only matching tool names are included regardless of mode. If `repo` or `all` is set, `find_repo_root(None)` is called to locate the git root.

Directory filtering happens at template level: `$HOME`/`~` templates are only expanded in user mode, `$REPO_ROOT` templates only in repo mode. This prevents unnecessary expansion errors when no repo root exists.

## Exit Codes

| Code | Meaning                                                                                                              |
| ---- | -------------------------------------------------------------------------------------------------------------------- |
| `0`  | All discovered skills are valid                                                                                      |
| `1`  | One or more skills have Error-severity diagnostics (or, in `--strict` mode, Warning/Suggestion-severity diagnostics) |
| `2`  | Scan or configuration error (failed to load config, parse error)                                                     |

## Key Types

| Type              | Module         | Purpose                                            |
| ----------------- | -------------- | -------------------------------------------------- |
| `ScanOptions`     | `scan.rs`      | Controls scan mode and filtering                   |
| `ScanResult`      | `scan.rs`      | Aggregated scan output with counts                 |
| `SkillValidation` | `scan.rs`      | Single skill with both pipeline and legacy results |
| `PathsConfig`     | `paths.rs`     | Parsed tool registry from `paths.jsonc`            |
| `ToolConfig`      | `paths.rs`     | Single tool entry (name, docs URL, dirs)           |
| `DiscoveredSkill` | `discovery.rs` | A found `SKILL.md` with its tool and path          |
| `DiscoveryResult` | `discovery.rs` | All discovered skills plus skipped directories     |
