# Validation Rules Reference

## Overview

This document describes all validation rules enforced by skills-validator.

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
3. Maximum 1024 characters

**Examples:**

- ✅ `"Validates agent skills"`
- ❌ `""` (empty)
- ❌ `"   "` (whitespace only)
- ❌ 1025 characters (too long)

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

**Rule:** Any field not in the official spec causes a validation error.

**Official spec fields:**

- `name`
- `description`
- `license`
- `allowed-tools`
- `metadata`
- `compatibility`

**Error message format:**

```text
Unexpected field in frontmatter: 'custom-field'. Only fields defined in the official spec are allowed. See https://agentskills.io/specification
```

---

## Claude Code Extensions (Warnings Only)

These fields generate warnings but don't fail validation:

| Field                      | Purpose                           |
| -------------------------- | --------------------------------- |
| `argument-hint`            | Hint shown during autocomplete    |
| `disable-model-invocation` | Prevent automatic loading         |
| `user-invocable`           | Hide from / menu                  |
| `model`                    | Model to use when skill is active |
| `context`                  | Run in forked subagent context    |
| `agent`                    | Which subagent type to use        |
| `hooks`                    | Hooks scoped to skill lifecycle   |

**Warning message format:**

```text
Field 'argument-hint' is a Claude Code extension (not in official spec). See https://code.claude.com/docs/en/skills
```

---

## Content Validation

### Keyword Detection

The validator checks for directive keywords in skill body content. Missing keywords generate warnings.

| Keyword   | Guidance                                                                                                                                                       |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `never`   | A well-written skill includes clear directives to NEVER do something and preferably ALWAYS do an alternative. See <https://agentskills.io/what-are-skills>     |
| `always`  | A well-written skill includes clear directives to ALWAYS do something in certain circumstances. See <https://agentskills.io/what-are-skills>                   |
| `when`    | A well-written skill contains 'when' statements to inform the agent of what conditions trigger certain behaviors. See <https://code.claude.com/docs/en/skills> |
| `example` | A well-written skill contains examples to inform the agent of what to do in commonly encountered circumstances. See <https://opencode.ai/docs/skills>          |

**Detection:**

- Case-insensitive search
- Matches partial words (e.g., "examples" matches "example")

**Warning message format:**

```text
'never' not found in skill content. A well-written skill includes clear directives to NEVER do something...
```

---

### Body Length

**Rule:** Body longer than 500 lines generates a warning.

**Rationale:** Progressive disclosure - skills should be focused and modular.

**Warning message format:**

```text
SKILL.md body has 600 lines (recommended: 500 or fewer). Consider using progressive disclosure patterns to keep skills focused. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#progressive-disclosure-patterns
```

---

## File System Validation

### Windows Path Detection

**Rule:** Windows-style paths in text files generate warnings.

**Checked file extensions:**

- `.md`
- `.txt`
- `.yaml`, `.yml`
- `.json`
- `.toml`

**Detected patterns:**

- `C:\` style absolute paths
- `\\` UNC paths

**Warning message format:**

```text
Windows-style path found in script.sh (line 42). Use forward slashes for cross-platform compatibility. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#avoid-windows-style-paths
```

---

### Script Organization

**Rule:** Script files in skill root directory generate warnings.

**Detected extensions:**

- `.sh`
- `.py`
- `.ps1`
- `.bat`
- `.cmd`

**Warning message format:**

```text
Script file 'deploy.sh' found in skill root directory. Consider organizing scripts in a dedicated directory. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview#level-3-resources-and-code-loaded-as-needed and https://agentskills.io/specification#optional-directories
```

**Recommendation:** Place scripts in a `scripts/` subdirectory.

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

## Exit Codes

| Exit Code | Meaning                                   |
| --------- | ----------------------------------------- |
| 0         | Success - valid skill (may have warnings) |
| 1         | Failure - validation errors present       |

**Note:** Warnings alone don't cause exit code 1. Only errors do.

---

## Validation Order

1. Directory exists check
2. Directory (not file) check
3. SKILL.md exists check
4. File read check
5. Frontmatter parse check
6. Unknown fields check
7. Claude Code extension check
8. Required fields check (name, description)
9. Field-specific validation
10. Content keyword check
11. Body length check
12. Windows path check
13. Script organization check

---

## Common Validation Failures

### Missing SKILL.md

```text
Missing required file: SKILL.md
```

**Fix:** Create SKILL.md in skill root directory.

### Missing name field

```text
Missing required field in frontmatter: name
```

**Fix:** Add `name: your-skill-name` to frontmatter.

### Name/directory mismatch

```text
Directory name 'my-skill' must match skill name 'myskill'
```

**Fix:** Rename directory to match skill name, or vice versa.

### Unknown field

```text
Unexpected field in frontmatter: 'custom_field'. Only fields defined in the official spec are allowed.
```

**Fix:** Remove custom fields or add to `metadata`.

### Invalid character in name

```text
Skill name 'my_skill' contains invalid characters. Only letters, digits, and hyphens are allowed.
```

**Fix:** Use hyphens instead of underscores: `my-skill`
