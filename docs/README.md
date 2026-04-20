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

The skills-validator is a Rust CLI tool and library for validating agent skills according to the [Agent Skills specification](https://agentskills.io/specification). It uses a five-pass validation pipeline with configurable severity levels and
automatic escalation based on skill complexity.

### Key Features

- ✅ **Five-Pass Validation Pipeline** - Modular validation across schema, requirements, readability, performance, and security
- 📊 **Configurable Severity Levels** - Four-tier severity model (info, suggestion, warning, error) with automatic escalation for larger skills
- 🔍 **Multi-Directory Scanning** - Discover and validate skills across multiple locations with `scan --all`
- 🎯 **Output Flexibility** - Human-readable and JSON output formats for CI/CD integration
- 🔐 **Optional Security Scanning** - Semgrep integration for security vulnerability detection
- ⚙️ **Configuration System** - TOML-based configuration for customizable validation behavior
- 📤 **XML Generation** - Create `<available_skills>` blocks for agents
- 🧪 **Shell Completions** - Built-in shell completion generation for bash, zsh, and other shells
- 🔧 **Rust Library API** - Comprehensive library for custom validation tools

### Commands

```bash
# Validate a skill directory
skills-validator validate path/to/skill

# Strict validation mode (all warnings treated as errors)
skills-validator validate --strict path/to/skill

# Scan multiple directories for skills
skills-validator scan --all

# Read skill properties as YAML
skills-validator read-properties path/to/skill

# Generate <available_skills> XML
skills-validator to-prompt path/to/skill-a path/to/skill-b

# Generate or update configuration file
skills-validator setup

# Generate shell completions
skills-validator completions bash
```

### Exit Codes

| Code | Meaning                         |
| ---- | ------------------------------- |
| 0    | Valid (warnings may be present) |
| 1    | Invalid (errors present)        |
| 2    | Scan or configuration error     |

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
