# Agent Conventions

This document outlines project conventions and rules for AI agents working on this codebase.

## Project Context

- This project validates agent skills per the [Agent Skills spec](https://agentskills.io/specification)
- Claude Code and OpenCode skill formats are the primary targets — prefer their conventions when making design decisions
- Reference docs: [agentskills.io best practices](https://agentskills.io/skill-creation/best-practices), [Claude Code skills docs](https://code.claude.com/docs/en/skills)

## Design Decisions

Architectural decisions are documented in `docs/decisions/`. Read these before proposing structural changes — they capture context and rejected alternatives that aren't obvious from the code.

## Code Style

- **Rust**: Follow standard Rust idioms and conventions
- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy` - fix warnings before committing
- **Testing**: All features require tests - run `cargo test` before committing

## Documentation

- **README.md**: User-facing CLI/API documentation
- **docs/**: Technical documentation (architecture, API reference, etc.)
- **docs/plans/**: Implementation plans - move completed plans to `docs/plans/completed/`
- Update documentation when adding new features

## Dependencies

- Keep dependencies minimal
- Use `cargo outdated` to check for updates
- Verify new dependencies don't introduce vulnerabilities

## Commit Messages

- Use clear, descriptive commit messages
- Reference issue numbers when applicable
- Squash related commits before merging

## Commands

```bash
# Development
cargo build --release
cargo test
cargo clippy

# Installation
cargo install --locked

# Code quality
cargo fmt
cargo clippy -- -D warnings
```

## Project Structure

```text
src/
├── cli.rs        # CLI argument parsing
├── config.rs     # Config loading (TOML + env overrides)
├── discovery.rs  # Skill discovery
├── error.rs      # Error types
├── formatter.rs  # Human and JSON output formatting
├── git.rs        # Git repository detection
├── lib.rs        # Public API exports
├── main.rs       # Binary entry point
├── models.rs     # Data structures (Diagnostic, Severity, Sizeyness, PipelineResult)
├── parser.rs     # YAML parsing
├── passes/       # Five-pass validation pipeline
│   ├── mod.rs        # Pass trait and module exports
│   ├── parse.rs      # Pass 1: Parse (pulldown-cmark AST)
│   ├── structure.rs  # Pass 2: Structure (file inventory, sizeyness, binary detection)
│   ├── content.rs    # Pass 3: Content (frontmatter, quality, reinforcement)
│   ├── references.rs # Pass 4: References (chain walking, orphan detection)
│   └── security.rs   # Pass 5: Security (semgrep, remote execution detection)
├── paths.rs      # Path configuration
├── pipeline.rs   # Pipeline orchestration
├── prompt.rs     # XML generation
├── scan.rs       # Scan orchestration
└── validator.rs  # Legacy validation logic (deprecated, wraps pipeline)
```

## Key Files

- `paths.jsonc`: Tool directory configurations (embedded at compile time)
- `Cargo.toml`: Dependencies and project metadata
- `docs/architecture.md`: System architecture documentation

## Diagnostic Output Style

This tool emits diagnostics at four severity tiers: **info**, **suggestion**, **warning**, and **error**.

### Human-readable output
Should be personable, friendly, and encouraging — like a knowledgeable friend reviewing your work. Use positive reinforcement for good practices ("Nice — your skill includes a gotchas section, which is one of the highest-value things you can add"), gentle nudges for improvements. The goal is to make skill authors feel supported, not scolded.

### JSON output
Spare and machine-useful. Same data points, no warmth. Keep factual descriptions ("skill includes gotchas section"), drop the encouragement. This output is consumed by CI pipelines, editors, and other tools.

### Severity tiers

| Tier | Purpose | Exit code |
|------|---------|-----------|
| **Info** | Positive reinforcement — "you have this and it's valuable" | 0 |
| **Suggestion** | Gentle nudge — "consider adding X" | 0 (1 with `--strict`) |
| **Warning** | Real quality concern affecting agent behavior | 0 (1 with `--strict`) |
| **Error** | Broken, spec-violating, or dangerous | 1 always |

Severity escalates with skill sizeyness. A check that's a suggestion for a simple skill may become a warning or error for a moderate or hefty one. See design docs for sizeyness tier definitions.
