# Validation Rules Reference

## Overview

This document describes all validation rules enforced by skills-validator v0.2.0.

The validator runs a five-pass pipeline. Each pass contributes diagnostics that are collected and reported together. Passes are ordered so that each one can assume the previous passed without fatal error.

---

## Severity Tiers

The validator uses a four-tier severity model.

| Tier       | Exit code (normal) | Exit code (--strict) | Meaning                                       |
| ---------- | ------------------ | -------------------- | --------------------------------------------- |
| Info       | 0                  | 0                    | Informational or positive reinforcement       |
| Suggestion | 0                  | 1                    | Best practice guidance — not required         |
| Warning    | 0                  | 1                    | Likely problem — review recommended           |
| Error      | 1                  | 1                    | Definite problem — skill is invalid or unsafe |

**Exit code 2** is returned for scan or configuration errors (I/O failures, invalid config file).

In normal mode, only `Error` diagnostics cause a non-zero exit. In `--strict` mode, `Suggestion` and `Warning` diagnostics also cause exit code 1.

---

## Sizeyness Escalation

Skills are classified into three tiers based on file count, subdirectory count, and orchestration fields:

| Tier     | Default thresholds                                            |
| -------- | ------------------------------------------------------------- |
| Simple   | Fewer than 3 files, 0 subdirectories, no orchestration fields |
| Moderate | 3–5 files, or 1–2 subdirectories                              |
| Hefty    | 6+ files, 3+ subdirectories, or any orchestration field       |

Orchestration fields that trigger Hefty classification: `hooks`, `agent`, `context`.

Thresholds are configurable. See `$XDG_CONFIG_HOME/skills-validator/config.toml`.

**How escalation works:** Some checks have a `base_severity` of Suggestion but escalate for larger skills. The actual diagnostic severity depends on the skill's tier at validation time.

| Check                      | Simple     | Moderate   | Hefty   |
| -------------------------- | ---------- | ---------- | ------- |
| Scripts in root            | Suggestion | Warning    | Warning |
| Description trigger lang   | Suggestion | Warning    | Error   |
| Trigger conditions in body | Suggestion | Warning    | Error   |
| Has examples               | Suggestion | Warning    | Warning |
| Behavioral constraints     | Suggestion | Warning    | Warning |
| Has gotchas section        | Suggestion | Suggestion | Warning |
| Body length exceeded       | Suggestion | Warning    | Error   |
| Broken reference           | Warning    | Error      | Error   |
| Orphaned files             | Suggestion | Warning    | Warning |

---

## Field Validation

### Required Fields

#### `name`

**Rules:**

1. Must be present
2. Must be non-empty after trimming
3. Maximum 64 characters
4. Must be lowercase
5. Can only contain: lowercase letters, digits, hyphens
6. Cannot start with hyphen
7. Cannot end with hyphen
8. Cannot contain consecutive hyphens (`--`)
9. Must match directory name (normalized)

**Normalization:**

- Uses Unicode NFKC normalization before validation

**Regex Pattern:**

```regex
^[a-z0-9]+(-[a-z0-9]+)*$
```

**Examples:**

- ✅ `my-skill`
- ✅ `skill123`
- ✅ `go-module`
- ❌ `My-Skill` (uppercase)
- ❌ `-skill` (starts with hyphen)
- ❌ `skill-` (ends with hyphen)
- ❌ `my--skill` (consecutive hyphens)
- ❌ `my_skill` (underscore)

---

#### `description`

**Rules:**

1. Must be present
2. Must be non-empty after trimming
3. Maximum 250 characters

**Rationale:** Claude Code truncates long descriptions in tool listings. Keeping descriptions under 250 characters ensures full visibility.

**Examples:**

- ✅ `"Validates agent skills"`
- ❌ `""` (empty)
- ❌ `"   "` (whitespace only)
- ❌ 251 characters (too long)

---

### Optional Fields

#### `license`

**Rules:**

1. Can be any string if present
1. Typically: SPDX identifier or `SEE LICENSE IN <file>`

**Common values:**

- `Apache-2.0`
- `MIT`
- `GPL-3.0`
- `BSD-3-Clause`

---

#### `compatibility`

**Rules:**

1. Maximum 500 characters
2. Describes environment requirements

**Examples:**

- `v1.0+`
- `requires: python>=3.9`
- `compatible with: claude-code>=2.0`

---

#### `allowed-tools`

**Rules:**

1. Space-delimited list (experimental feature)
2. No specific format enforced

**Example:**

```yaml
allowed-tools: read write edit bash
```

---

#### `metadata`

**Rules:**

1. Must be a YAML mapping if present
2. Keys and values are strings
3. Can contain arbitrary key-value pairs

**Example:**

```yaml
metadata:
  author: John Doe
  version: "2.0"
  category: utility
```

---

## Unknown Fields

**Rule:** Any field not in the official spec and not a recognized Claude Code extension produces a **Warning** (not an error).

**Official spec fields:**

- `name`
- `description`
- `license`
- `allowed-tools`
- `metadata`
- `compatibility`

**Warning message format:**

```text
Unknown frontmatter field `custom-field`. Spec fields: name, description, license, allowed-tools, metadata, compatibility. Claude Code extensions: argument-hint, disable-model-invocation, user-invocable, model, context, agent, hooks.
```

---

## Claude Code Extensions (Suggestions Only)

These fields generate suggestions but don't fail validation:

| Field                      | Purpose                           |
| -------------------------- | --------------------------------- |
| `argument-hint`            | Hint shown during autocomplete    |
| `disable-model-invocation` | Prevent automatic loading         |
| `user-invocable`           | Hide from / menu                  |
| `model`                    | Model to use when skill is active |
| `context`                  | Run in forked subagent context    |
| `agent`                    | Which subagent type to use        |
| `hooks`                    | Hooks scoped to skill lifecycle   |

**Suggestion message format:**

```text
Field `argument-hint` is recognized by Claude Code but may not be used by other tools.
```

**Extension semantic checks (Pass 3):**

- `context` must equal `fork` if present. Any other value is an error.
- `agent` without `context: fork` produces a warning.
- `model` value is checked against the known models list. An unrecognized model produces a suggestion.

---

## Content Validation

### Keyword Detection

The validator checks for directive keywords in skill body content. These checks operate on the prose-only view of the body — code blocks, inline code, and URL text are stripped before matching. Keywords are matched using word-boundary anchors so
partial words are not counted (e.g. `whenever` does not satisfy the `never` check).

**Behavioral constraints (`never` / `always`):**

```regex
(?i)\bnever\b
(?i)\balways\b
```

Missing behavioral constraints: Suggestion for Simple, Warning for Moderate and Hefty.

**Trigger conditions (`use when` / `trigger when` / `activate when`):**

```regex
(?i)\b(use when|trigger when|activate when)\b
```

Also satisfied by a heading containing "When to Use". Missing trigger conditions: Suggestion for Simple, Warning for Moderate, Error for Hefty.

**Examples (code blocks or heading containing "example"):**

Missing examples: Suggestion for Simple, Warning for Moderate and Hefty.

**Gotchas section (heading containing "Gotchas", "Caveats", "Pitfalls", or "Common Mistakes"):**

Missing gotchas: Suggestion for Simple and Moderate, Warning for Hefty.

---

### Body Length

**Rule:** Body longer than 300 lines (default, configurable) generates a diagnostic.

**Escalation:** Suggestion for Simple, Warning for Moderate, Error for Hefty.

**Rationale:** Progressive disclosure — skills should be focused and modular. Use linked subdirectory files for extended content.

**Message format:**

```text
Body is 400 lines, exceeding the 300-line limit. Consider splitting into referenced files.
```

---

## File System Validation

### Binary File Detection (Pass 2)

**Rule:** Binary files in a skill directory are an error.

**Detection:** Files are classified as binary if their extension is in a known binary extension list (`.exe`, `.dll`, `.so`, `.dylib`, `.wasm`, `.o`, `.a`, `.pyc`, `.class`, `.obj`, `.lib`, `.bin`, `.elf`) or if the first 8192 bytes contain a null
byte.

**Error message format:**

```text
Binary file detected: `scripts/tool.exe`. Compiled binaries in skills are a security concern.
```

---

### Script Organization

**Rule:** Script files in the skill root directory generate a diagnostic.

**Detected extensions:** `.py`, `.sh`, `.bash`, `.rb`, `.js`, `.ts`, `.ps1`, `.bat`, `.cmd`

**Escalation:** Suggestion for Simple, Warning for Moderate and Hefty.

**Message format:**

```text
Scripts found in skill root — consider organizing into `scripts/`
```

**Recommendation:** Place scripts in a `scripts/` subdirectory.

---

### Windows Path Detection

**Rule:** Windows-style paths in prose content generate a suggestion.

**Detected patterns:**

- `C:\` style absolute paths (regex: `[A-Z]:\\[\w\\]+`)

**Note:** Detection runs on the prose-only view — code blocks are excluded.

**Message format:**

```text
Windows-style paths detected in prose. Consider using POSIX paths or platform-agnostic references.
```

---

### Sizeyness Classification (Pass 2)

**Rule:** Every skill receives a sizeyness classification. This is always emitted as an Info diagnostic.

**Message format:**

```text
Skill classified as Moderate (3 files, 1 subdirectories)
```

**Machine format:**

```text
sizeyness:moderate:files=3:subdirs=1:orchestration=false
```

---

## YAML Frontmatter Rules

### Format Requirements

**Rule 1:** Must start with `---`

**Error:**

```text
SKILL.md must start with YAML frontmatter (---)
```

**Rule 2:** Must close frontmatter with `---`

**Error:**

```text
SKILL.md frontmatter not properly closed with ---
```

**Rule 3:** Content between markers must be valid YAML

**Error:**

```text
Failed to parse SKILL.md: <yaml_error_details>
```

**Rule 4:** Frontmatter must be a YAML mapping (key-value pairs)

**Error:**

```text
SKILL.md frontmatter must be a YAML mapping
```

---

## SKILL.md Casing

**Rule:** The file must be named exactly `SKILL.md`. Variants such as `skill.md` or `Skill.md` are rejected with an error.

**Rationale:** Case-insensitive filesystems (e.g. macOS default) silently accept wrong casing. The validator reads the actual directory listing to enforce exact casing regardless of filesystem behavior.

**Error message format:**

```text
Found 'skill.md' but the file must be named exactly 'SKILL.md'. Please rename it.
```

---

## Reference Validation (Pass 4)

Pass 4 walks markdown reference chains starting from `SKILL.md` and checks every local file reference.

### Chain Walking

The validator follows internal markdown links up to **5 hops** (configurable via `references.markdown_hop_limit`). Fragment identifiers (e.g. `file.md#section`) are stripped before resolving. Backtick-quoted paths in prose text (e.g.
`` `docs/setup.md` ``) are also followed.

**Hop limit reached** produces an Info diagnostic (not an error). This is informational and does not indicate a problem.

**Circular references** produce an Info diagnostic.

### Broken References

A reference to a non-existent file produces a diagnostic. Escalates based on sizeyness: Warning for Simple, Error for Moderate and Hefty.

```text
Referenced file 'docs/setup.md' does not exist.
```

### Orphan Detection

Files present in the skill directory but not reachable from any markdown chain are reported as orphans.

**Escalation:** Suggestion for Simple, Warning for Moderate and Hefty.

**Exclusions (default):** `LICENSE*`, `CHANGELOG*`, `README*`, `.gitignore`, `.*`

**Message format:**

```text
These files aren't referenced from any markdown file: scripts/helper.py. They may still be used by scripts, but the validator can't verify that.
```

### Path Boundary Checks

**Rule:** References that resolve outside the skill directory are blocked and produce a Warning.

This catches both literal `../` traversal and symlinks that exit the skill directory boundary. Canonicalized paths are checked against the canonical skill directory path.

**Warning message format:**

```text
Reference '../../../etc/passwd' resolves outside the skill directory.
```

### Hooks Script Validation

If a skill uses the `hooks` extension field, all referenced script paths are checked for existence within the skill directory. Missing hook scripts are an error.

**Error message format:**

```text
Hooks reference script 'scripts/pre.sh' but the file does not exist.
```

---

## Security Validation (Pass 5)

### Remote Execution Pattern Detection

**Always runs.** The validator scans both prose text and fenced code block content for patterns that pipe remote content into a shell.

**Detected patterns:**

| Pattern            | Example                     |
| ------------------ | --------------------------- |
| `curl ... \| bash` | `curl https://x.sh \| bash` |
| `curl ... \| sh`   | `curl https://x.sh \| sh`   |
| `wget ... \| bash` | `wget https://x.sh \| bash` |
| `wget ... \| sh`   | `wget https://x.sh \| sh`   |
| `bash <(curl ...)` | `bash <(curl https://x.sh)` |
| `sh <(curl ...)`   | `sh <(curl https://x.sh)`   |

**Severity:** Warning for each match.

**Message format:**

```text
Skill may direct execution of remote code (`curl https://example.com/install.sh | bash`).
```

### Semgrep Integration

When the skill contains script files and `semgrep` is available, Pass 5 runs bundled semgrep rules against:

- Actual script files (`.py`, `.sh`, `.bash`, `.rb`, `.js`, `.ts`, `.ps1`, `.bat`, `.cmd`)
- Fenced code blocks with recognized language tags, extracted to temp files

**Bundled rule sets:**

- `shell-injection.yaml`
- `python-exec.yaml`
- `env-exfiltration.yaml`
- `hardcoded-urls.yaml`
- `filesystem-escape.yaml`

Semgrep severity levels map to validator severity: `ERROR` → Error, `WARNING` → Warning, `INFO` → Suggestion.

If semgrep is not installed or disabled, a Suggestion is emitted listing the script files that were not analyzed.

**No-semgrep message format:**

```text
This skill contains script files (scripts/deploy.sh). Install semgrep for automated security analysis.
```

Semgrep can be disabled via config (`security.semgrep_enabled = false`) or the `SKILLS_VALIDATOR_SECURITY_SEMGREP_ENABLED=false` environment variable.

---

## Exit Codes

| Exit Code | Meaning                                                         |
| --------- | --------------------------------------------------------------- |
| 0         | Success — valid skill (may have suggestions, warnings, or info) |
| 1         | Failure — validation errors present (or warnings in --strict)   |
| 2         | Scan or configuration error (I/O failure, invalid config file)  |

**Note:** In normal mode, only `Error` diagnostics cause exit code 1. In `--strict` mode, `Suggestion` and `Warning` also cause exit code 1.

---

## Validation Pipeline Order

Skills are validated in five sequential passes. A fatal error in an earlier pass stops the pipeline.

### Pass 1 (Parse)

1. Locate `SKILL.md` by reading the directory listing (exact casing enforced)
2. Read file contents
3. Extract YAML frontmatter
4. Parse markdown body with pulldown-cmark into typed collections (headings, links, code blocks, prose-only text view)

A failure in Pass 1 is fatal — subsequent passes do not run.

### Pass 2 (Structure)

1. Walk directory tree (symlinks not followed)
2. Classify every file (Markdown, Script, Config, Binary, Other)
3. Detect binary files by extension and null-byte sniffing
4. Compute sizeyness (Simple / Moderate / Hefty)
5. Check for scripts in root directory

### Pass 3 (Content)

1. Validate frontmatter fields (name format, description length, unknown fields, extension fields)
2. Check name/directory match
3. Check extension semantic rules (context value, agent+context pairing, model recognition)
4. Content quality checks (trigger conditions, examples, behavioral constraints, gotchas, body length)
5. Windows path detection in prose
6. Positive reinforcement diagnostics

### Pass 4 (References)

1. Collect links from `ctx.links` and backtick-quoted paths in prose
2. Walk markdown chain up to `markdown_hop_limit` hops, following internal links
3. Detect broken references, circular references, and hop limit reached
4. Check path boundaries (traversal and symlink escape detection)
5. Validate hooks script paths
6. Detect orphaned files

### Pass 5 (Security)

1. Scan prose and code blocks for remote execution patterns
2. If script files present and semgrep available: run semgrep with bundled rules against scripts and extracted code blocks
3. If semgrep unavailable: emit suggestion to install semgrep

---

## Common Validation Failures

### Missing SKILL.md

```text
SKILL.md not found in the skill directory.
```

**Fix:** Create SKILL.md in skill root directory.

### Wrong SKILL.md casing

```text
Found 'skill.md' but the file must be named exactly 'SKILL.md'. Please rename it.
```

**Fix:** Rename `skill.md` to `SKILL.md`.

### Missing name field

```text
Frontmatter must include a `name` field.
```

**Fix:** Add `name: your-skill-name` to frontmatter.

### Name/directory mismatch

```text
Skill name 'my-skill' does not match directory name 'myskill'.
```

**Fix:** Rename directory to match skill name, or vice versa.

### Unknown field (warning, not error)

```text
Unknown frontmatter field `custom_field`. Spec fields: name, description, ...
```

**Fix:** Remove custom fields or add them to the `metadata` mapping.

### Invalid character in name

```text
Skill name 'my_skill' has invalid format: only letters, digits, and hyphens allowed.
```

**Fix:** Use hyphens instead of underscores: `my-skill`

### Binary file detected

```text
Binary file detected: `bin/tool`. Compiled binaries in skills are a security concern.
```

**Fix:** Remove compiled binaries. Distribute source code and build instructions instead.

### Remote execution pattern

```text
Skill may direct execution of remote code (`curl https://example.com/install.sh | bash`).
```

**Fix:** Do not instruct the model to pipe remote content into a shell. Provide explicit installation steps using a package manager or verified checksums.
