# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/moutons/skills-validator/compare/v0.2.1...v0.3.0) - 2026-04-28

### Other

- *(deps)* update pulldown-cmark requirement from 0.12 to 0.13
- *(deps)* update toml requirement from 0.8 to 1.1
- remove deprecated validator, add forbid(unsafe_code) ([#26](https://github.com/moutons/skills-validator/pull/26))
- refresh README with crates.io install, config reference, badges ([#25](https://github.com/moutons/skills-validator/pull/25))
- update codeql-action from v3 to v4 ([#23](https://github.com/moutons/skills-validator/pull/23))
- update codeql-action from v3 to v4

## [0.2.1](https://github.com/moutons/skills-validator/compare/v0.2.0...v0.2.1) - 2026-04-20

### Fixed

- remove tag trigger from CI, let release-plz own releases

## [0.2.0](https://github.com/moutons/skills-validator/compare/v0.1.7...v0.2.0) - 2026-04-05

### Breaking Changes

- **JSON output format changed**: `schema_version: 2` with a new diagnostic array structure. The old `{"valid":..., "errors":..., "warnings":...}` format is replaced.
- **`--json` flag deprecated**: Still works but emits a deprecation warning. Use `--output-format json` instead.
- **Rust API changes**: `validate()` deprecated in favor of `run_pipeline()`. `ValidationResult` deprecated in favor of `PipelineResult`/`Vec<Diagnostic>`.
- **Severity demotions**: `unknown-field` moved from error to warning, `body-length` from warning to suggestion, `windows-paths` from warning to suggestion.
- **Description length limit**: Changed from 1024 to 250 characters.
- Library API and JSON output schema are subject to change pre-1.0.

### Added

- Five-pass validation pipeline (Parse, Structure, Content, References, Security) replacing monolithic validator.
- Four-tier diagnostic severity model: Info, Suggestion, Warning, Error.
- Sizeyness-aware severity escalation: Simple, Moderate, and Hefty skills receive different severity levels for the same checks.
- Configurable thresholds via `~/.config/skills-validator/config.toml` with sections for `[sizeyness]`, `[content]`, `[references]`, and `[security]`.
- `skills-validator setup` subcommand to generate a default config file.
- `skills-validator completions <shell>` for shell completions (bash, zsh, fish, elvish, powershell).
- Optional semgrep integration for script security analysis.
- `--strict` flag: exit 1 on warnings or suggestions.
- `--output-format human|json` flag.
- `--severity info|suggestion|warning|error` filter flag.
- Positive reinforcement diagnostics (info tier) for good practices.
- Reference chain walking for markdown file links (up to 5 hops).
- Orphan file detection.
- Binary file detection.
- Remote execution pattern detection in scripts.
- Environment variable overrides for config: `SKILLS_VALIDATOR_<SECTION>_<KEY>`.

### Migration Guide

1. Replace `--json` with `--output-format json` in scripts and CI pipelines.
2. Update JSON consumers to handle `schema_version: 2` diagnostic array format.
3. Replace `validate()` calls with `run_pipeline()` in Rust code.
4. Review any logic that depends on specific error/warning classifications, as some severities have changed.

## [0.1.7](https://github.com/moutons/skills-validator/compare/v0.1.6...v0.1.7) - 2026-02-15

### Other

- add branching strategy to AGENTS.md ([#6](https://github.com/moutons/skills-validator/pull/6))
