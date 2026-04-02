# Project Specification: skills-validator

## Overview

A Rust CLI tool and library for validating agent skills according to the [Agent Skills specification](https://agentskills.io/specification). This is a reimplementation of the Python `agentskills/skills-ref` library with improvements, supporting both the official spec and Claude Code extensions.

---

## Goals and Objectives

### Primary Goals
1. **Validate Skill Compliance**: Ensure agent skill directories conform to the official Agent Skills specification
2. **Enforce Strict Standards**: Unknown fields cause validation failures, maintaining spec purity
3. **Generate Agent Prompts**: Create XML-formatted `<available_skills>` blocks for system prompts
4. **Support Multiple Implementations**: Work with both OpenCode and Claude Code skill formats

### Success Criteria
- Exit code 0 = valid skill with no errors
- Exit code 1 = errors present (warnings alone don't fail validation)
- Fast, reliable validation for CI/CD pipelines
- Clear, actionable error messages

---

## Target Audience

| Audience | Use Case |
|----------|----------|
| **Skill Authors** | Validate skills before publishing |
| **DevOps/CI** | Automated validation in pipelines |
| **Agent Developers** | Generate prompt XML for system configuration |
| **Tool Builders** | Rust API for custom validation tools |

---

## Specification Compliance

### Official Spec (agentskills.io)

The validator strictly enforces the official specification. Unknown fields cause validation failures.

#### Required Fields

| Field | Constraints |
|-------|-------------|
| `name` | Max 64 chars. Lowercase letters, numbers, hyphens. No leading/trailing hyphen. No consecutive hyphens. Must match directory name. Pattern: `^[a-z0-9]+(-[a-z0-9]+)*$` |
| `description` | Max 1024 chars. Non-empty string. |

#### Optional Fields

| Field | Constraints |
|-------|-------------|
| `license` | License name or reference to bundled license file |
| `compatibility` | Max 500 chars. Environment requirements |
| `allowed-tools` | Space-delimited list of pre-approved tools (experimental) |
| `metadata` | Arbitrary key-value mapping for additional metadata |

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

```
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

| Constraint | Description |
|------------|-------------|
| UTF-8 Only | All text files must be UTF-8 encoded |
| YAML Frontmatter | Must start with `---` and close with `---` |
| No Windows Paths | Cross-platform compatibility required |
| Max Body Lines | 500 lines recommended for progressive disclosure |

### Content Best Practices (Warnings)

The validator warns when skill content is missing key directive words:

| Keyword | Purpose |
|---------|---------|
| `never` | Clear directives on what NOT to do |
| `always` | Clear directives on what TO do |
| `when` | Condition triggers for behaviors |
| `example` | Concrete examples of usage |

### File Organization Warnings

- Scripts in root directory trigger warnings (should be in `scripts/`)
- Windows-style paths (`C:\`, `\\`) trigger warnings
- Files exceeding 500 lines trigger warnings

---

## Dependencies

### Runtime Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| clap | 4.5 | CLI argument parsing with derive macros |
| serde | 1.0 | Serialization framework |
| serde_yaml | 0.9 | YAML parsing for frontmatter |
| serde_json | 1.0 | JSON output support |
| html-escape | 0.2 | XML escaping for prompts |
| thiserror | 2.0 | Error type definitions |
| unicode-normalization | 0.1 | NFKC normalization for skill names |
| log | 0.4 | Logging facade |
| env_logger | 0.11 | Environment-based logging |
| owo-colors | 4 | Terminal colors |
| regex | 1.10 | Pattern matching |

### Development Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| tempfile | 3.10 | Temporary file creation in tests |

---

## Version

Current: **0.1.7**

See [Cargo.toml](../Cargo.toml) for dependency versions.

---

## License

Apache 2.0

See [LICENSE](../LICENSE) or <https://www.apache.org/licenses/LICENSE-2.0>
