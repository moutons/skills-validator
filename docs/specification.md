# Project Specification: skills-validator

## Overview

A Rust CLI tool and library for validating agent skills according to the [Agent Skills specification](https://agentskills.io/specification). This is a reimplementation of the Python `agentskills/skills-ref` library with improvements, supporting both
the official spec and Claude Code extensions.

---

## Goals and Objectives

### Primary Goals

1. **Validate Skill Compliance**: Ensure agent skill directories conform to the official Agent Skills specification
2. **Enforce Strict Standards**: Unknown fields produce warnings; `--strict` mode fails on any non-info diagnostic
3. **Generate Agent Prompts**: Create XML-formatted `<available_skills>` blocks for system prompts
4. **Support Multiple Implementations**: Work with both OpenCode and Claude Code skill formats

### Success Criteria

- Exit code 0 = valid skill with no errors
- Exit code 1 = errors present (warnings alone don't fail validation)
- Fast, reliable validation for CI/CD pipelines
- Clear, actionable error messages

---

## Target Audience

| Audience             | Use Case                                     |
| -------------------- | -------------------------------------------- |
| **Skill Authors**    | Validate skills before publishing            |
| **DevOps/CI**        | Automated validation in pipelines            |
| **Agent Developers** | Generate prompt XML for system configuration |
| **Tool Builders**    | Rust API for custom validation tools         |

---

## Commands

| Command           | Purpose                                           |
| ----------------- | ------------------------------------------------- |
| `validate`        | Validate a single skill directory                 |
| `scan`            | Discover and validate skills across tool dirs     |
| `read-properties` | Extract and display skill metadata                |
| `to-prompt`       | Generate XML-formatted `<available_skills>` block |
| `setup`           | Initialize configuration and directories          |
| `completions`     | Generate shell completions for bash/zsh/fish      |

---

## Specification Compliance

### Official Spec (agentskills.io)

The validator checks against the official specification. Unknown fields produce warnings; Claude Code extensions are recognized separately.

#### Required Fields

| Field         | Constraints                                                                                                                                                           |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `name`        | Max 64 chars. Lowercase letters, numbers, hyphens. No leading/trailing hyphen. No consecutive hyphens. Must match directory name. Pattern: `^[a-z0-9]+(-[a-z0-9]+)*$` |
| `description` | Max 250 chars. Non-empty string.                                                                                                                                      |

#### Optional Fields

| Field           | Constraints                                               |
| --------------- | --------------------------------------------------------- |
| `license`       | License name or reference to bundled license file         |
| `compatibility` | Max 500 chars. Environment requirements                   |
| `allowed-tools` | Space-delimited list of pre-approved tools (experimental) |
| `metadata`      | Arbitrary key-value mapping for additional metadata       |

### Claude Code Extensions

Claude Code supports additional fields. These generate **warnings** but don't block validation:

- `argument-hint` - Hint shown during autocomplete
- `disable-model-invocation` - Prevent automatic loading
- `user-invocable` - Hide from / menu
- `model` - Model to use when skill is active
- `context` - Run in forked subagent context
- `agent` - Which subagent type to use
- `hooks` - Hooks scoped to skill lifecycle

Reference: [Claude Code Skills Documentation](https://code.claude.com/docs/en/skills)

---

## Skill Directory Structure

```text
skill-name/
├── SKILL.md          # Required - skill definition with YAML frontmatter
├── scripts/          # Optional - script resources
├── assets/           # Optional - additional resources
└── ...               # Other optional files/directories
```

### SKILL.md Format

```yaml
---
name: my-skill
description: What this skill does and when to use it
license: Apache-2.0
compatibility: v1.0+
allowed-tools: read write
metadata:
  author: John Doe
  version: "1.0"
---
# Skill Content

This is the skill documentation that agents will read when loading the skill.
```

---

## Constraints and Assumptions

### Technical Constraints

| Constraint       | Description                                      |
| ---------------- | ------------------------------------------------ |
| UTF-8 Only       | All text files must be UTF-8 encoded             |
| YAML Frontmatter | Must start with `---` and close with `---`       |
| No Windows Paths | Cross-platform compatibility required            |
| Max Body Lines   | 300 lines recommended for progressive disclosure |

### Content Best Practices (Warnings)

The validator warns when skill content is missing key directive words:

| Keyword   | Purpose                            |
| --------- | ---------------------------------- |
| `never`   | Clear directives on what NOT to do |
| `always`  | Clear directives on what TO do     |
| `when`    | Condition triggers for behaviors   |
| `example` | Concrete examples of usage         |

### File Organization Warnings

- Scripts in root directory trigger warnings (should be in `scripts/`)
- Windows-style paths (`C:\`, `\\`) trigger warnings
- Files exceeding 300 lines trigger warnings

---

## Validation Pipeline

Validation runs as a five-pass pipeline:

1. **Parse** - Reads and parses SKILL.md files with YAML frontmatter
2. **Structure** - Validates skill structure and required fields
3. **Content** - Analyzes markdown content for quality directives and best practices
4. **References** - Checks internal references and metadata consistency
5. **Security** - Scans for security issues using pattern matching and remote execution detection

Each pass produces diagnostics at various severity levels, enabling gradual enforcement.

---

## Diagnostic Severity Tiers

Diagnostics are classified into four tiers:

| Tier       | Meaning                                                           |
| ---------- | ----------------------------------------------------------------- |
| Info       | Informational messages, no action required                        |
| Suggestion | Best practice recommendations, skills pass validation             |
| Warning    | Violations of content guidelines, skills pass validation          |
| Error      | Specification violations, causes validation failure (exit code 1) |

---

## Sizeyness Classification

Skills are classified based on file count, subdirectory count, and orchestration fields:

- **Simple** - Fewer than 3 files, 0 subdirectories, no orchestration fields
- **Moderate** - 3+ files or 1+ subdirectories
- **Hefty** - 6+ files, 3+ subdirectories, or has orchestration fields (`hooks`, `agent`, `context`)

Sizeyness affects severity escalation: a check that produces a suggestion for a simple skill may escalate to a warning or error for a moderate or hefty one.

---

## Configuration

The validator reads configuration from TOML files at `$XDG_CONFIG_HOME/skills-validator/config.toml` (or `~/.config/skills-validator/config.toml` on Linux/macOS).

Configuration options include severity overrides, enabled passes, and logging levels.

---

## Scan Command

The `scan` command discovers and validates skills across multiple tool directories:

- Searches tool directories (e.g., `~/.claude/tools`, `/usr/local/tools`)
- Recursively finds skill directories containing SKILL.md
- Runs the full five-pass pipeline on each skill
- Aggregates and reports results by directory

---

## Dependencies

### Runtime Dependencies

| Crate                 | Version | Purpose                                 |
| --------------------- | ------- | --------------------------------------- |
| clap                  | 4.5     | CLI argument parsing with derive macros |
| clap_complete         | 4.5     | Shell completion generation             |
| serde                 | 1.0     | Serialization framework                 |
| serde_yaml            | 0.9     | YAML parsing for frontmatter            |
| serde_json            | 1.0     | JSON output support                     |
| html-escape           | 0.2     | XML escaping for prompts                |
| thiserror             | 2.0     | Error type definitions                  |
| unicode-normalization | 0.1     | NFKC normalization for skill names      |
| log                   | 0.4     | Logging facade                          |
| env_logger            | 0.11    | Environment-based logging               |
| owo-colors            | 4       | Terminal colors                         |
| regex                 | 1.10    | Pattern matching                        |
| dirs                  | 6.0     | XDG directory resolution                |
| git2                  | 0.20    | Git repository detection                |
| walkdir               | 2.5     | Directory tree walking                  |
| rayon                 | 1.10    | Parallel validation                     |
| pulldown-cmark        | 0.12    | Markdown AST parsing                    |
| toml                  | 0.8     | TOML config file parsing                |
| tempfile              | 3.10    | Secure temporary files                  |

### Development Dependencies

None (all testing dependencies are development-only)

---

## Version

Current: **0.2.0**

See [Cargo.toml](../Cargo.toml) for dependency versions.

---

## License

Apache 2.0

See [LICENSE](../LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>
