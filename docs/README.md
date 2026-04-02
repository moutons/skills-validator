# Skills-Validator Documentation

Complete documentation for the skills-validator project.

## Table of Contents

1. **[Specification](specification.md)** - Project requirements and specification compliance
2. **[Architecture](architecture.md)** - System design and module structure
3. **[API Reference](api-reference.md)** - Rust library and CLI API documentation
4. **[Validation Rules](validation-rules.md)** - Detailed validation rules reference
5. **[Development Guide](development-guide.md)** - Development workflow and conventions
6. **[Testing](testing.md)** - Testing strategy and best practices

## Quick Links

### External Specifications

- [Agent Skills Specification](https://agentskills.io/specification)
- [OpenCode Skills](https://opencode.ai/docs/skills/)
- [Claude Code Skills](https://code.claude.com/docs/en/skills)

### Project Resources

- [README.md](../README.md) - User-facing documentation
- [AGENTS.md](../AGENTS.md) - Project conventions and rules
- [Cargo.toml](../Cargo.toml) - Dependencies and metadata
- [Justfile](../Justfile) - Development tasks

### Source Code

- [src/](../src/) - Source code directory
- [tests/](../tests/) - Test suite
- [.github/](../.github/) - GitHub Actions workflows

## Overview

The skills-validator is a Rust CLI tool and library for validating agent skills according to the [Agent Skills specification](https://agentskills.io/specification).

### Key Features

- ✅ **Strict Spec Compliance** - Unknown fields cause validation failures
- ⚠️ **Claude Code Support** - Extensions generate warnings but don't block
- 📋 **Multiple Commands** - `validate`, `read-properties`, `to-prompt`
- 🔧 **Rust API** - Library for custom tools
- 📤 **XML Generation** - Create `<available_skills>` blocks for agents
- 🧪 **CI/CD Ready** - Exit codes and JSON output

### Commands

```bash
# Validate a skill directory
skills-validator validate path/to/skill

# Read skill properties as YAML
skills-validator read-properties path/to/skill

# Generate <available_skills> XML
skills-validator to-prompt path/to/skill-a path/to/skill-b
```

### Exit Codes

| Code | Meaning                         |
| ---- | ------------------------------- |
| 0    | Valid (warnings may be present) |
| 1    | Invalid (errors present)        |

---

## Contributing

See the [Development Guide](development-guide.md) for:

- Setting up your development environment
- Running tests
- Code conventions
- Adding new features
- Release process

## License

Apache 2.0 - See [LICENSE](../LICENSE)
