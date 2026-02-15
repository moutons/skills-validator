# Project AGENTS.md

This is a Rust reimplementation of the [agentskills/skills-ref](https://github.com/agentskills/agentskills/tree/main/skills-ref) Python library with some valuable improvements.

## Project Structure

```text
src/
├── main.rs          # CLI entry point
├── cli.rs           # CLI commands
├── error.rs         # Error types
├── models.rs        # SkillProperties data model
├── parser.rs        # YAML frontmatter parsing
├── validator.rs     # Skill validation logic
└── prompt.rs        # XML prompt generation
tests/               # Integration tests
```

## Tech Stack

- **CLI**: clap
- **YAML parsing**: serde_yaml
- **HTML escaping**: html-escape

## Development Workflow

### Just Recipes

Use `just` recipes instead of raw tool commands:

```bash
# Run all CI checks locally (REQUIRED before committing)
just ensure-ci

# Run tests during development
just test

# Run linting
just clippy

# Format code
just fmt

# Security audit
just security

# Applies GitHub workflow file linting and pinning (this may make changes to workflow files, and that's okay)
just workflows

# Full check (all of the above + markdown lint)
just full
```

### Before Committing

**ALWAYS run `just ensure-ci` and fix any issues before committing work.**

This runs: fmt, clippy, test, security audit, github workflow lint, and markdown lint.

### GitHub Actions

- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Run `just workflows` to lint workflow files, ensure `dtolnay/rust-toolchain` is using `stable`, and all other workflows are pinned to git hashes
- When we need to know the most recent version of a GitHub action (to upgrade or fix issues in workflow files), run `gh release view`, see this example for usage:

```shell
❯ gh release view --repo katyo/publish-crates --json tagName --jq '.tagName'
v2
```

## Specification References

This implementation follows the Agent Skills specification and key implementations, see their documentation:

- **Official Spec**: <https://agentskills.io/specification>
- **OpenCode Implementation**: <https://opencode.ai/docs/skills/>
- **Claude Code Implementation**: <https://code.claude.com/docs/en/skills>

### Skill Directory Structure

A valid skill directory must contain:

- `SKILL.md` (or `skill.md`) - Required file
- Directory name must match the `name` field in frontmatter

### SKILL.md Frontmatter Format

```yaml
---
name: my-skill
description: What this skill does and when to use it
license: Apache-2.0          # optional
compatibility: v1.0+         # optional
allowed-tools: tool_pattern  # optional, experimental
metadata:                    # optional
  key: value
---
```

### Validation Rules

| Field           | Required | Constraints                                                                                                       |
| --------------- | -------- | ----------------------------------------------------------------------------------------------------------------- |
| `name`          | Yes      | Max 64 characters. Lowercase letters, numbers, and hyphens only. Must not start or end with a hyphen. Must not contain consecutive hyphens (`--`). Must match directory name. |
| `description`   | Yes      | Max 1024 characters. Non-empty.                                                                                   |
| `license`       | No       | License name or reference to a bundled license file.                                                              |
| `compatibility` | No       | Max 500 characters. Indicates environment requirements.                                                          |
| `metadata`      | No       | Arbitrary key-value mapping for additional metadata.                                                              |
| `allowed-tools` | No       | Space-delimited list of pre-approved tools. (Experimental)                                                        |

**Unknown fields are NOT allowed** - this validator strictly follows the spec and rejects any frontmatter fields not in the list above.

### Claude Code Extensions

Claude Code supports additional fields beyond the official spec. These will generate **warnings** but not errors:

- `argument-hint` - Hint shown during autocomplete
- `disable-model-invocation` - Prevent automatic loading
- `user-invocable` - Hide from / menu
- `model` - Model to use when skill is active
- `context` - Run in forked subagent context
- `agent` - Which subagent type to use
- `hooks` - Hooks scoped to skill lifecycle

See <https://code.claude.com/docs/en/skills> for details.

### Skill Name Validation Regex

```shell
^[a-z0-9]+(-[a-z0-9]+)*$
```

## Commands

```bash
# Validate a skill directory
skills-validator validate path/to/skill

# Read skill properties (YAML output)
skills-validator read-properties path/to/skill

# Generate <available_skills> XML
skills-validator to-prompt path/to/skill-a path/to/skill-b
```

## Notes

- The spec is king - unknown fields cause validation failures
- Claude Code extensions generate warnings but don't block validation
- Exit code 0 = valid, exit code 1 = errors present
