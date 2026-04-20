# Agent Instructions

Read this file first. It contains the development methodology and project conventions for this repository. Claude-specific configuration is in `.claude/CLAUDE.md`.

## Development Methodology

This project uses spec-driven development with [obra/superpowers](https://github.com/obra/superpowers).

1. **Spec first.** Every feature, bugfix, or refactor starts with a spec in `docs/specs/`. No implementation without an approved spec.
2. **Plan second.** Implementation plans live in `docs/plans/`. Plans are written before code and executed task-by-task.
3. **Code third.** Follow the plan. Commit after each task.

## Project Context

- This project validates agent skills per the [Agent Skills spec](https://agentskills.io/specification)
- Claude Code and OpenCode skill formats are the primary targets — prefer their conventions when making design decisions
- Reference docs: [agentskills.io best practices](https://agentskills.io/skill-creation/best-practices), [Claude Code skills docs](https://code.claude.com/docs/en/skills)

## Agent Dispatch

Prefer the lightest-weight agent possible for any task:

- **Cheapest model** (e.g., haiku): deterministic work -- file lookups, formatting checks, simple searches, running `just` recipes.
- **Mid-tier model** (e.g., sonnet): moderate complexity -- code review, standard implementations, test writing, debugging.
- **Expensive model** (e.g., opus): deep reasoning only -- architectural decisions, complex refactors, ambiguous specs.

Give agents focused, complete prompts so they can work autonomously and return exactly what you need.

## Progressive Disclosure

Load only the context you need. Do not read the entire codebase to fix a typo.

## Design Decisions

Architectural decisions are documented in `docs/decisions/`. Read these before proposing structural changes — they capture context and rejected alternatives that aren't obvious from the code.

## Git Hooks

This project uses lefthook for pre-commit and pre-push hooks. **Hooks must always pass before committing or pushing — no exceptions.** If a hook reports failures, fix them even if the issues are pre-existing and unrelated to your changes. Never use `--no-verify` to bypass hooks.

Deterministic checks are enforced via lefthook:

- **Pre-commit** (lightweight): markdown linting, format checking. These run fast and should never be skipped.
- **Pre-push** (full): `just ensure-ci` runs all CI checks. This is the gate that matters. If pre-push fails, fix the issue before pushing.

Agents do not need to internalize formatting or linting rules. The hooks catch violations automatically. Focus on writing correct code; the hooks handle style.

## Justfile as Single Source of Truth

All common tasks go through `just`. Do not invoke formatters, linters, or test runners directly -- use the justfile recipes. This prevents divergence when different agents default to different tools.

## Code Style

- **Rust**: Follow standard Rust idioms and conventions
- **Formatting**: Use `cargo fmt`
- **Linting**: Use `cargo clippy` - fix warnings before committing
- **Testing**: All features require tests - run `cargo test` before committing

## Documentation

- **README.md**: User-facing CLI/API documentation
- **docs/**: Technical documentation (architecture, API reference, etc.)
- **docs/specs/**: Design specs (`YYYY-MM-DD-<topic>-design.md`). Kept as permanent reference after implementation.
- **docs/plans/**: Implementation plans (`YYYY-MM-DD-<feature-name>.md`). Delete plans once fully implemented — the code and git history are the source of truth.
- **docs/decisions/**: ADRs (`NNNN-title.md`). One decision per file. Short (100–200 lines). Never batch multiple decisions into one ADR.
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

Should be personable, friendly, and encouraging — like a knowledgeable friend reviewing your work. Use positive reinforcement for good practices ("Nice — your skill includes a gotchas section, which is one of the highest-value things you can add"),
gentle nudges for improvements. The goal is to make skill authors feel supported, not scolded.

### JSON output

Spare and machine-useful. Same data points, no warmth. Keep factual descriptions ("skill includes gotchas section"), drop the encouragement. This output is consumed by CI pipelines, editors, and other tools.

### Severity tiers

| Tier           | Purpose                                                    | Exit code             |
| -------------- | ---------------------------------------------------------- | --------------------- |
| **Info**       | Positive reinforcement — "you have this and it's valuable" | 0                     |
| **Suggestion** | Gentle nudge — "consider adding X"                         | 0 (1 with `--strict`) |
| **Warning**    | Real quality concern affecting agent behavior              | 0 (1 with `--strict`) |
| **Error**      | Broken, spec-violating, or dangerous                       | 1 always              |

Severity escalates with skill sizeyness. A check that's a suggestion for a simple skill may become a warning or error for a moderate or hefty one. See design docs for sizeyness tier definitions.

## Emergent Decisions

When a decision arises about conventions, tooling, or patterns: **ask the user** whether it belongs in:

- **Project settings** (`AGENTS.md`, `.claude/settings.json`) -- applies to this repo only
- **User-profile settings** (`~/.claude/CLAUDE.md`, `~/.claude/settings.json`) -- applies to all repos

Do not silently commit to a convention without surfacing this choice.

## Git Worktrees

When using git worktrees for isolated work, place them in `.worktrees/` at the repository root. This directory is gitignored.

## Shell Script Portability

All shell scripts must be compatible with macOS default bash 3.2 (GPLv2). Do not use bash 4+ features:

- No `${VAR^}` / `${VAR,,}` (case modification) -- use `awk` or `tr` instead
- No `declare -A` (associative arrays) -- use indexed arrays
- No `|&` (pipe stderr) -- use `2>&1 |`
- When in doubt, prefer POSIX sh constructs over bash-specific features

## Version Pinning

Preferred pinning order (most preferred first):

1. Commit SHA
2. Full version number (e.g., `v1.2.3`)
3. Unversioned pin (e.g., `v1`)
4. No pinning -- **never use this**

Apply to: CI action references, dependency locks, tool versions. Never use `@latest` tags.
