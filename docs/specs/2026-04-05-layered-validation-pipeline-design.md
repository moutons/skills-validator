# Layered Validation Pipeline Design

**Date:** 2026-04-05
**Status:** Approved
**Decision:** [0001-layered-analysis-pipeline](../decisions/0001-layered-analysis-pipeline.md)

## Overview

Restructure the skills-validator from a single-pass checker with ~6 basic checks into a five-pass analysis pipeline with ~30 checks spanning content quality, structural integrity, referential integrity, and optional security analysis. The pipeline introduces sizeyness-aware severity escalation, a four-tier diagnostic model, and configurable thresholds — all without breaking the workflow for simple single-file skills.

### Goals

- Apply meaningful backpressure on both humans and agents developing skills
- Keep simple skills frictionless — most new checks are suggestions or info for simple skills
- Escalate strictness proportionally to skill sizeyness
- Prefer Claude Code and OpenCode skill conventions when making opinionated checks
- Provide warm, encouraging human output and spare, machine-useful JSON output
- Make security analysis available but optional (semgrep integration)

### Non-goals

- Replacing semgrep or building a full SAST tool
- Enforcing rules for tools we don't actively track
- Evaluating whether skill *content* is actually good advice

## Data Model

### Sizeyness Tiers

Skills are classified into three sizeyness tiers based on their directory structure and frontmatter:

| Tier | Triggers (any one) |
|------|-------------------|
| **Simple** | 1-2 files, no subdirectories, no orchestration fields |
| **Moderate** | 3-5 files OR 1-2 subdirectories |
| **Hefty** | 6+ files OR 3+ subdirectories OR orchestration frontmatter (`hooks`, `agent`, `context`) |

A skill is classified at the **highest tier** for which any single criterion is met. File count and subdirectory count are evaluated independently; orchestration fields are evaluated independently; the maximum tier wins.

Thresholds are configurable (see [Configuration](#configuration)).

### Diagnostic Severity

Four tiers replace the current two-tier (warning/error) model:

| Tier | Purpose | Exit code |
|------|---------|-----------|
| **Info** | Positive reinforcement — "you have this and it's valuable" | 0 |
| **Suggestion** | Gentle nudge — "consider adding X" | 0 (1 with `--strict`) |
| **Warning** | Absence measurably degrades agent behavior | 0 (1 with `--strict`) |
| **Error** | Broken, spec-violating, or dangerous | 1 always |

Severity escalates with sizeyness. A check with base severity `suggestion` may become `warning` for moderate skills and `error` for hefty skills.

**Decision rule:** Use **Suggestion** when the skill works correctly without the recommended practice. Use **Warning** when absence measurably degrades agent behavior (e.g., agents won't know when to activate, files are unreachable).

### Diagnostic

```rust
struct Diagnostic {
    severity: Severity,
    check_name: CheckName,         // e.g. "description-length", "orphaned-files"
    human_message: String,      // warm, friendly, encouraging
    machine_message: String,    // spare, factual
    doc_url: Option<String>,    // link to relevant docs
    file_path: Option<PathBuf>, // which file triggered this
    base_severity: Severity,    // severity before sizeyness escalation
}
```

Every check produces `Vec<Diagnostic>`. The formatter chooses `human_message` or `machine_message` based on output mode.

### Skill Context

Accumulated state flowing through the pipeline. Each pass returns `Result<Vec<Diagnostic>, PipelineError>`. If a pass returns `PipelineError`, the pipeline emits a system-level diagnostic and stops (or continues to the next independent pass, depending on the error).

```rust
struct SkillContext {
    // Pass 1: Parse
    frontmatter: Frontmatter,
    // Extracted from AST during Pass 1 — headings, links, code blocks, prose segments
    // The raw event stream is discarded after extraction to avoid unnecessary cloning.
    headings: Vec<Heading>,
    links: Vec<Link>,
    code_blocks: Vec<CodeBlock>,
    prose_text: String,

    // Pass 2: Structure
    sizeyness: Sizeyness,
    file_inventory: Vec<FileEntry>,    // path, size, type (markdown/script/binary/other)
    subdirectories: Vec<PathBuf>,

    // Pass 4: accumulated from markdown chain
    referenced_files: HashSet<PathBuf>,
}

enum PipelineError {
    ParseFailed { path: PathBuf, reason: String },
    IoError { path: PathBuf, reason: String },
    SemgrepFailed { reason: String },
    ConfigInvalid { reason: String },
}
```

`PipelineError` is distinct from `Diagnostic` — it represents infrastructure failures ("the validator broke"), not skill quality issues ("your skill has a problem"). When a `PipelineError` occurs, it is converted to a system-level `Diagnostic` with severity `Error` and a check name of `pipeline-error`.

### Exit Code Logic

```rust
fn exit_code(diagnostics: &[Diagnostic], strict: bool) -> i32 {
    if diagnostics.iter().any(|d| d.severity == Error) { return 1; }
    if strict && diagnostics.iter().any(|d| matches!(d.severity, Warning | Suggestion)) { return 1; }
    0
}
```

## Pipeline Architecture

Five passes, each building on the previous:

| Pass | Name | Inputs | Outputs |
|------|------|--------|---------|
| 1 | **Parse** | Raw SKILL.md | `pulldown-cmark` AST, frontmatter fields |
| 2 | **Structure** | Skill directory | File inventory, sizeyness tier, subdirectory map, binary detection |
| 3 | **Content** | AST + sizeyness tier | Heading analysis, keyword checks, description quality, content diagnostics |
| 4 | **References** | AST + file inventory + sizeyness tier | Reference chain validation, orphan detection, extension field validation |
| 5 | **Security** (optional) | File inventory + detected scripts | Semgrep analysis if available, otherwise advisory warnings |

If pass 1 fails (parse errors), the pipeline stops — no point running downstream passes.

## Pass 1: Parse

**Inputs:** Path to skill directory
**Outputs:** Frontmatter struct, markdown AST, prose-only body text

### Behavior

1. Look for exactly `SKILL.md` in the skill root. The filename is case-sensitive — `skill.md`, `Skill.md`, etc. are errors with guidance to rename.
2. Extract YAML frontmatter between `---` delimiters (existing logic).
3. Parse frontmatter as YAML into `Frontmatter` struct (existing logic).
4. Parse body through `pulldown-cmark` into an event stream.
5. Extract a "prose-only" view from the AST — strip fenced code blocks, inline code, and URLs. Content checks in pass 3 operate on this view.

### Diagnostics

| Check | Severity | Escalates? |
|-------|----------|------------|
| `skill-file-exists` | Error | No |
| `skill-file-casing` | Error | No — must be exactly `SKILL.md` |
| `frontmatter-present` | Error | No |
| `frontmatter-valid-yaml` | Error | No |
| `frontmatter-is-mapping` | Error | No |

All parse diagnostics are errors regardless of sizeyness — parse failures are always fatal.

## Pass 2: Structure

**Inputs:** Skill directory path, parsed frontmatter
**Outputs:** Sizeyness tier, file inventory, subdirectory map

### Behavior

1. Walk the skill directory tree, cataloging every file.
2. Classify each file:
   - **Markdown** — `.md` extension
   - **Script** — `.py`, `.sh`, `.bash`, `.rb`, `.js`, `.ts`, `.ps1`, `.bat`, `.cmd`
   - **Binary** — null bytes in first 8KB, or known binary extensions (`.exe`, `.dll`, `.so`, `.dylib`, `.wasm`, `.o`, `.a`, `.pyc`, `.class`)
   - **Config** — `.json`, `.yaml`, `.yml`, `.toml`, `.jsonc`
   - **Other** — everything else
3. Record subdirectories.
4. Compute sizeyness tier from thresholds.

### Diagnostics

| Check | Base severity | Escalates? | Details |
|-------|--------------|------------|---------|
| `binary-detected` | Error | All tiers: error | "Binary file detected: `{path}`. Compiled binaries in skills are a security concern and shouldn't be distributed this way." |
| `scripts-in-root` | Suggestion | Simple: suggestion, Moderate+: warning | "Scripts found in skill root — consider organizing into `scripts/`" |
| `sizeyness-info` | Info | All tiers: info | "Skill classified as {tier} ({reasons})" |

## Pass 3: Content

**Inputs:** Markdown AST (prose-only view), frontmatter, sizeyness tier
**Outputs:** Content quality diagnostics

### Frontmatter checks

| Check | Base severity | Escalates? | Details |
|-------|--------------|------------|---------|
| `name-missing` | Error | No | Required field |
| `name-format` | Error | No | Lowercase, hyphens, 1-64 chars |
| `name-directory-match` | Error | No | Must match directory name |
| `description-missing` | Error | No | Required field |
| `description-length` | Error | No | >250 chars. Link to Claude Code docs on truncation. |
| `description-trigger-language` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: error | Description should contain "use when", "trigger when", etc. |
| `unknown-field` | Warning | No | Field not in spec or known extensions. Docs pointer to spec field list. |
| `extension-field-compatibility` | Suggestion | No | "Field `{name}` is recognized by Claude Code but may not be used by other tools." |

### Extension field validation

| Check | Severity | Details |
|-------|----------|---------|
| `context-valid-value` | Error | If `context` is set, must be `fork` (only documented value per Claude Code docs) |
| `agent-with-context` | Warning | `agent` without `context: fork` has no effect |
| `model-recognized` | Suggestion | Check against a bundled list of known Claude model identifiers (e.g., `claude-opus-4-6`, `claude-sonnet-4-6`, `claude-haiku-4-5-20251001`). List is configurable via `[content] known_models` in config. Unknown values get a suggestion, not an error — new models ship faster than validator releases. |

### Content quality checks (prose-only AST)

| Check | Base severity | Escalates? | Details |
|-------|--------------|------------|---------|
| `has-trigger-conditions` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: error | Word-boundary match for "use when", "trigger when", "activate when", or heading containing "when to use" |
| `has-examples` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: warning | Fenced code blocks OR heading containing "example" |
| `has-behavioral-constraints` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: warning | Word-boundary `\bnever\b` and `\balways\b` |
| `has-gotchas` | Suggestion | Simple: suggestion, Moderate: suggestion, Hefty: warning | Heading containing "gotcha", "caveat", "pitfall", or "common mistake". Link to agentskills.io best practices. |
| `body-length` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: error | >300 lines (configurable). Link to progressive disclosure docs. |
| `windows-paths` | Suggestion | No | Existing check, AST-aware (prose only, not code blocks — a Windows path in a code example is likely intentional) |

**Escalation rationale:** `has-trigger-conditions` escalates to error for hefty skills because without trigger language, agents cannot determine when to activate an orchestrated skill — this directly breaks functionality. `has-examples` and `has-behavioral-constraints` escalate only to warning because their absence degrades quality but doesn't prevent the skill from working. `has-gotchas` stays at suggestion for moderate skills because gotchas are highest-value but not structurally required.

### Positive reinforcement (info tier)

These fire when good practices are present. Each checks for the structural pattern with substantive content beneath it (not just an empty heading):

| Check | Details |
|-------|---------|
| `has-gotchas-section` | Heading with gotcha/caveat/pitfall keyword AND at least one list item or paragraph beneath it. |
| `has-validation-loop` | Checklist patterns or "validate" + "run" in proximity with substantive content. |
| `has-progressive-disclosure` | SKILL.md references files in subdirectories. |
| `has-concrete-examples` | Fenced code blocks present near example headings with substantive content. |

## Pass 4: References

**Inputs:** AST, file inventory, sizeyness tier, subdirectory map
**Outputs:** Reference chain validation, orphan detection

### Reference chain walking

1. Extract file references from the already-parsed SKILL.md AST (from `SkillContext.links`):
   - Markdown links: `[text](path)`
   - Backtick-quoted paths: `` `scripts/setup.sh` ``
   - Fenced code blocks containing relative paths (heuristic — tokens matching `something/something.ext`)
2. **Canonicalize and bound all paths:** resolve each reference with `std::path::Path::canonicalize()`, then verify it starts with the skill root directory. Reject any path that escapes the skill directory (path traversal protection).
3. **Normalize to NFC:** apply Unicode NFC normalization to resolved paths before comparison, ensuring consistent matching for non-ASCII filenames.
4. For each referenced markdown file, parse it and extract its references too.
5. Follow up to 5 hops (configurable), where SKILL.md is hop 0. A file at hop 5 is the 6th file in the chain. Track visited files via `HashSet<PathBuf>` to detect cycles.
6. Build a reachability set rooted at SKILL.md.

### Symlink policy

Symlinks in the skill directory are followed and treated as their targets. The resolved (canonical) path is what's checked against the skill root boundary. A symlink pointing outside the skill directory is treated as a broken reference.

### Diagnostics

| Check | Base severity | Escalates? | Details |
|-------|--------------|------------|---------|
| `broken-reference` | Warning | Simple: warning, Moderate+: error | Referenced file doesn't exist. Base severity is warning for simple skills because simple skills may have WIP references during development. |
| `orphaned-files` | Suggestion | Simple: suggestion, Moderate: warning, Hefty: warning | Files unreachable from markdown chain. Message: "These files aren't referenced from any markdown file in this skill. They may still be used by scripts, but the validator can't verify that." |
| `hooks-script-missing` | Error | No, always error | `hooks` frontmatter references a script that doesn't exist |
| `circular-reference` | Info | No | "Circular reference detected: A.md → B.md → A.md. All files are reachable but this may indicate confusing documentation structure." |
| `hop-limit-reached` | Info | No | "Reference chain exceeded {limit} hops. Deeper references were not followed. Consider simplifying the documentation structure." |
| `path-traversal-blocked` | Warning | No | "Reference `{path}` resolves outside the skill directory and was not followed." |

### Exclusions from orphan detection

Conventional files are excluded by default: `LICENSE*`, `CHANGELOG*`, `README*` (when not the entrypoint), `.gitignore`, and dot-prefixed files. The exclusion list is configurable via `[references] orphan_exclusions` in config.

## Pass 5: Security (optional)

**Inputs:** File inventory (scripts from pass 2), AST (embedded code blocks)
**Outputs:** Script security diagnostics

### When semgrep is available

1. Detect `semgrep` on PATH (or configured path).
2. **Batch all targets for a single semgrep invocation:** collect all script files from the file inventory, plus temp files for embedded code blocks (see below). Invoke semgrep once with the full list of paths. Map findings back to skills by path prefix.
3. **Embedded code block extraction:** extract fenced code blocks with language tags from the AST, write each to a temp file using the `tempfile` crate (mode 0o600, user read/write only). Clean up temp files via RAII drop guards — cleanup happens even if semgrep crashes.
4. **Semgrep invocation:** use `std::process::Command` with explicit argument arrays (never shell interpolation). Parse semgrep `--json` output. If semgrep exits non-zero or produces malformed JSON, emit a `semgrep-execution-failed` diagnostic (Warning) and continue — do not propagate the failure as a pipeline error.
5. Detect remote execution patterns in the AST (e.g., `curl | bash`). These are heuristic pattern matches, not semgrep rules.
6. **Parallelism note:** Pass 5 should run **outside** the rayon parallel iterator used for per-skill validation. Collect all script targets across skills first, then batch into a single semgrep invocation. This avoids spawning N concurrent semgrep processes.

### Bundled semgrep rules

Shipped as YAML, embedded at compile time:

| Rule | Languages | What it catches |
|------|-----------|-----------------|
| `shell-injection` | Bash, sh | `eval`, backtick execution, `curl \| bash`, unsanitized variable expansion |
| `python-exec` | Python | `eval()`, `exec()`, `subprocess.call(shell=True)`, `os.system()` |
| `env-exfiltration` | All | Environment variables sent to network destinations |
| `hardcoded-urls` | All | URLs/IPs that aren't localhost or well-known documentation sites |
| `filesystem-escape` | All | `../` traversal patterns, absolute paths outside skill directory |

Semgrep severity mapping:

| Semgrep | Ours |
|---------|------|
| ERROR | Error |
| WARNING | Warning |
| INFO | Suggestion |

### When semgrep is NOT available

| Check | Severity | Details |
|-------|----------|---------|
| `scripts-detected-no-semgrep` | Suggestion | "This skill contains script files ({list}). Install semgrep for automated security analysis." |
| `script-detected` | Info | Per-script: "Script `{path}` detected ({language}). Use appropriate linters and security tooling to verify." |

### Remote execution detection

| Check | Severity | Details |
|-------|----------|---------|
| `remote-execution-pattern` | Warning | "Skill may direct execution of remote code (`{pattern}`). This can't be verified by the validator." |

Detected via heuristic pattern matching in the AST — not a guarantee, just a signal.

## Output Formatting

### Human output (default)

Grouped by pass, with severity-appropriate formatting. Warm, friendly, encouraging tone. Positive reinforcement for good practices, gentle nudges for improvements, clear errors for real problems. Each warning/error includes a documentation link where applicable.

Example:
```
📁 my-skill/ (moderate — 4 files, 1 subdirectory)

  ✅ Skill includes a gotchas section with concrete content — that's one of
     the highest-value things you can add.
  ✅ Good use of progressive disclosure — agents load detail on demand.

  💡 Consider adding trigger language to your description so agents know
     when to activate this skill.
     → https://code.claude.com/docs/en/skills#frontmatter-reference

  ⚠️  2 files in this skill aren't referenced from any markdown file.
     They may still be used by scripts, but the validator can't verify that:
       - scripts/unused.py
       - data/config.json

  ❌ Binary file detected: lib/helper.so
     Compiled binaries in skills are a security concern.
     → https://agentskills.io/skill-creation/best-practices

Summary: 1 error, 1 warning, 1 suggestion, 2 passed checks
```

### JSON output (`--json`)

Spare, machine-useful. Same data, no warmth.

```json
{
  "schema_version": 2,
  "skill": "my-skill",
  "path": "/home/user/.claude/skills/my-skill",
  "sizeyness": "moderate",
  "sizeyness_reasons": ["4 files", "1 subdirectory"],
  "diagnostics": [
    {
      "check": "has-gotchas-section",
      "severity": "info",
      "message": "Skill includes gotchas section with content",
      "file": "SKILL.md"
    },
    {
      "check": "binary-detected",
      "severity": "error",
      "message": "Binary file detected",
      "file": "lib/helper.so",
      "doc_url": "https://agentskills.io/skill-creation/best-practices"
    }
  ],
  "summary": {
    "errors": 1,
    "warnings": 1,
    "suggestions": 1,
    "info": 2
  },
  "exit_code": 1
}
```

## Breaking Changes

This design is a **semver-breaking change** (0.1.x → 0.2.0). Changes that affect existing users:

### Public Rust API

- `ValidationResult { errors: Vec<String>, warnings: Vec<String> }` is replaced by `Vec<Diagnostic>`. All consumers of the library API must update.
- The `validate()` function signature changes to return the new `Diagnostic` type.

### JSON output schema

- The current schema `{ "valid": bool, "errors": [...], "warnings": [...] }` is replaced entirely. A `"schema_version": 2` field is added so consumers can detect the format.
- No v1 compat — clean break. README documents that output schema is subject to change pre-1.0.

### CLI flag semantics

- `--json` currently controls log formatting to stderr. The new design repurposes it for structured validation output to stdout. To avoid a silent behavioral reversal, rename to `--output-format json` and deprecate `--json` with a migration message.

### Severity demotions

These checks now exit 0 where they previously exited 1 (without `--strict`):
- `unknown-field`: error → warning
- `body-length`: warning → suggestion
- `windows-paths`: warning → suggestion

Users relying on exit codes to gate CI should add `--strict` to preserve previous behavior.

### Check name renames

Existing check names change. Check names are a typed `pub enum CheckName` with kebab-case serialization — renaming a check is a compile error.

## CLI Changes

### New flags

| Flag | Purpose |
|------|---------|
| `--strict` | Promote warnings and suggestions to errors (exit 1) |
| `--output-format <fmt>` | Output format: `human` (default) or `json`. Replaces `--json` which is deprecated with a migration message. |
| `--severity <level>` | Minimum severity to display: `info` (default), `suggestion`, `warning`, `error` |

### New subcommand

| Command | Purpose |
|---------|---------|
| `skills-validator setup` | Generate config file at `$XDG_CONFIG_HOME/skills-validator/config.toml` with all defaults shown and commented out. Errors if config already exists, showing the file path. |

## Configuration

### Config file

Located at `$XDG_CONFIG_HOME/skills-validator/config.toml` (typically `~/.config/skills-validator/config.toml`).

Generated by `skills-validator setup` with all values commented out:

```toml
# [sizeyness]
# moderate_file_threshold = 3
# hefty_file_threshold = 6
# moderate_subdir_threshold = 1
# hefty_subdir_threshold = 3

# [content]
# body_line_limit = 300
# known_models = ["claude-opus-4-6", "claude-sonnet-4-6", "claude-haiku-4-5-20251001"]

# [references]
# markdown_hop_limit = 5
# orphan_exclusions = ["LICENSE*", "CHANGELOG*", "README*", ".gitignore", ".*"]

# [security]
# semgrep_enabled = true
# semgrep_path = "semgrep"
# custom_rules_dir = ""
```

### Config validation

On load, validate all config values:
- Thresholds must be positive integers
- `moderate_file_threshold` < `hefty_file_threshold`
- `moderate_subdir_threshold` < `hefty_subdir_threshold`
- `body_line_limit` > 0
- `markdown_hop_limit` > 0

Invalid values emit a warning-level diagnostic naming the bad key and the default it reverted to. Invalid TOML syntax emits an error-level diagnostic with the parse error and line number, and suggests `skills-validator setup` to regenerate.

### Override order

Compiled defaults → config file → env vars → CLI flags.

Env var naming: `SKILLS_VALIDATOR_` prefix + section + `_` + key, uppercase. Example: `SKILLS_VALIDATOR_SIZEYNESS_MODERATE_FILE_THRESHOLD=4`.

### Setup subcommand behavior

- No config directory: create it, write config file with commented defaults.
- Config directory exists, no config file: write config file with commented defaults.
- Config file already exists: error, print the path to the existing file.
- All behavior documented in `--help` and README.

## Codebase Impact

### New dependencies

| Crate | Purpose | Required? |
|-------|---------|-----------|
| `pulldown-cmark` | Markdown AST parsing | Yes |
| `toml` | Config file parsing | Yes |
| `tempfile` | Secure temp file creation for embedded code block scanning | Yes |
| `dirs` | XDG config directory resolution | Already a dependency |

External: `semgrep` — optional, detected at runtime.

### New files

| File | Purpose |
|------|---------|
| `src/pipeline.rs` | Pipeline orchestration |
| `src/passes/mod.rs` | Pass module root |
| `src/passes/parse.rs` | Pass 1 |
| `src/passes/structure.rs` | Pass 2 |
| `src/passes/content.rs` | Pass 3 |
| `src/passes/references.rs` | Pass 4 |
| `src/passes/security.rs` | Pass 5 |
| `src/config.rs` | Config loading, env var resolution, defaults |
| `src/formatter.rs` | Human and JSON output formatting |
| `rules/` | Bundled semgrep YAML rule files |

### Files that change

| File | Change |
|------|--------|
| `src/validator.rs` | Major refactor — split into pipeline passes |
| `src/models.rs` | Expand: `Diagnostic`, `Severity`, `Sizeyness`, `SkillContext`, `FileEntry`, config types |
| `src/parser.rs` | Add `pulldown-cmark` AST, prose extraction, enforce `SKILL.md` exact casing |
| `src/cli.rs` | Add `--strict`, `--severity`, `setup` subcommand, config overrides |
| `src/scan.rs` | Wire new pipeline into scan orchestration |
| `Cargo.toml` | Add `pulldown-cmark`, `toml` |

### Unchanged

- `src/discovery.rs` — skill discovery logic
- `src/git.rs` — git detection
- `src/paths.rs` — path config and JSONC parsing
- `src/prompt.rs` — XML generation
- `paths.jsonc` — tool directory mappings
- All existing test fixtures — still valid, gain additional diagnostics

### Migration of existing checks

| Current check | Moves to | Severity change |
|---------------|----------|-----------------|
| `name-*` checks | Pass 3 | No change — still errors |
| `description-length` | Pass 3 | Threshold changes to 250, stays error |
| `unknown-field` | Pass 3 | Demoted from error to warning |
| `extension-field` | Pass 3 | Split: compatibility suggestion + value validation |
| `body-length` | Pass 3 | Threshold changes to 300 (configurable), becomes suggestion with escalation |
| `windows-paths` | Pass 3 | Demoted to suggestion, AST-aware |
| `scripts-in-root` | Pass 2 | Becomes suggestion with escalation |
| Content keywords | Pass 3 | Word-boundary/AST-based, split into specific checks |

## Resolved decisions

- **"Complexity tier" → "Sizeyness":** Renamed to avoid confusion with computational complexity. Rust enum: `Sizeyness` with variants `Simple`, `Moderate`, `Hefty`. Config section: `[sizeyness]`. Human output uses "sizeyness" as the label.
- **Check names are a typed enum:** `pub enum CheckName` with kebab-case serialization. Compiler-enforced stability — renaming a check is a compile error, not a silent break.
- **No JSON v1 compat:** Clean break at 0.2.0. README documents that output schema is subject to change at any time pre-1.0.
- **No ValidationResult compat shim:** Clean break at 0.2.0. README documents that library API is subject to change at any time pre-1.0.

## Future work (not in scope)

- **TODO:** Detect "provide defaults, not menus" anti-pattern — warn when skill presents multiple tools/approaches as equal options without a clear recommendation. Deferred because heuristic is fuzzy.
- **Inflection to plugin architecture:** See [Decision 0001](../decisions/0001-layered-analysis-pipeline.md) for migration triggers.
