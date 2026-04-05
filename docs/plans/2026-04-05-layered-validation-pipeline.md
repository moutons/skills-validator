# Layered Validation Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the validator into a five-pass pipeline with sizeyness-aware severity escalation, four-tier diagnostics, configurable thresholds, and optional semgrep integration.

**Architecture:** Five sequential passes (Parse → Structure → Content → References → Security), each returning `Result<Vec<Diagnostic>, PipelineError>`. A `SkillContext` accumulates state between passes. Sizeyness tier (simple/moderate/hefty) determines severity escalation.

**Tech Stack:** Rust, pulldown-cmark (markdown AST), toml (config), tempfile (secure temp files), semgrep (optional external)

**Spec:** `docs/specs/2026-04-05-layered-validation-pipeline-design.md`
**Decision:** `docs/decisions/0001-layered-analysis-pipeline.md`

---

### Task 1: Data Model — Diagnostic, Severity, Sizeyness, PipelineError

**Files:**
- Modify: `src/models.rs`
- Modify: `Cargo.toml` (add `pulldown-cmark`, `toml`, `tempfile` to dependencies)
- Test: `tests/models.rs`

- [ ] Write failing tests in `tests/models.rs`: `Severity` ordering, `Sizeyness::from_counts()` at each boundary, `Diagnostic` construction, severity escalation logic, `PipelineError` display
- [ ] Run tests — expect compile errors
- [ ] Add deps to Cargo.toml: `pulldown-cmark = "0.12"`, `toml = "0.8"`, move `tempfile = "3.10"` to deps
- [ ] Implement types: `Severity` (Info/Suggestion/Warning/Error with Ord), `Sizeyness` enum (Simple/Moderate/Hefty with `from_counts()`), `CheckName` enum (all ~30 check names, serializes to kebab-case strings), `Diagnostic` struct per spec (uses `CheckName` not `String`), `PipelineError` enum, `escalate()` function, `SkillContext` struct (accumulated pipeline state), `FileEntry` struct (path + file type classification). Keep existing `SkillProperties`/`ValidationResult` (deprecated, not removed)
- [ ] Run tests — all pass
- [ ] Commit: `feat: add Diagnostic, Severity, Sizeyness, PipelineError types`

---

### Task 2: Config System — Loading, Validation, Setup Subcommand

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs` (add `pub mod config`)
- Modify: `src/cli.rs` (add `setup` subcommand)
- Test: `tests/config.rs` (create)

- [ ] Write failing tests: default values, TOML parsing, validation (sizeyness thresholds ordered, positive values), env var override, invalid config diagnostics
- [ ] Implement `src/config.rs`: `ValidatorConfig` with sections (`SizeynessConfig`, `ContentConfig`, `ReferencesConfig`, `SecurityConfig`), `load()` → `(ValidatorConfig, Vec<Diagnostic>)`, XDG path resolution, env var overrides
- [ ] Add `Setup` subcommand to `src/cli.rs`: resolve XDG path, error if config exists, create dir if needed, write commented defaults
- [ ] Add `clap_complete` dependency, add `Completions` subcommand generating shell completions for bash/zsh/fish/elvish/powershell (`skills-validator completions bash`)
- [ ] Run tests — all pass
- [ ] Commit: `feat: add config system, setup and completions subcommands`

---

### Task 3: Pass 1 — Parse (pulldown-cmark AST)

**Files:**
- Create: `src/passes/mod.rs`, `src/passes/parse.rs`
- Modify: `src/parser.rs` (enforce exact `SKILL.md` casing)
- Modify: `src/lib.rs` (add `pub mod passes`)
- Test: `tests/passes_parse.rs` (create)

- [ ] Write failing tests: exact `SKILL.md` enforcement (reject `skill.md`/`Skill.md`), frontmatter extraction, AST extraction (headings, links, code blocks), prose-only view strips code blocks and URLs
- [ ] Create `src/passes/mod.rs` with all five submodule declarations
- [ ] Implement `src/passes/parse.rs`: `run(skill_dir: &Path) -> Result<SkillContext, PipelineError>` — find `SKILL.md`, parse frontmatter (reuse `parser.rs`), parse body with `pulldown_cmark::Parser`, extract typed collections into `SkillContext`
- [ ] Update `find_skill_md` in `src/parser.rs` to enforce exact `SKILL.md` only, with diagnostic if wrong casing found
- [ ] Run tests — all pass
- [ ] Commit: `feat: Pass 1 (Parse) with pulldown-cmark AST extraction`

---

### Task 4: Pass 2 — Structure (file inventory, sizeyness, binary detection)

**Files:**
- Create: `src/passes/structure.rs`
- Test: `tests/passes_structure.rs` (create)
- Create test fixtures: `tests/fixtures/skills/binary-in-skill/` (with a fake binary file)

- [ ] Write failing tests: file inventory from `security-ownership-map` fixture, file type classification, binary detection (null bytes), sizeyness at each boundary (2=simple, 3=moderate, 6=hefty), subdirectory counting, orchestration field promotion
- [ ] Create binary test fixture: `tests/fixtures/skills/binary-in-skill/` with SKILL.md + `lib/helper.so` (bytes with \0)
- [ ] Implement `src/passes/structure.rs`: `run(ctx, config) -> Result<Vec<Diagnostic>, PipelineError>` — walkdir, classify files, detect binaries (first 8KB), compute sizeyness tier, emit diagnostics
- [ ] Run tests — all pass
- [ ] Commit: `feat: Pass 2 (Structure) with file inventory, sizeyness tiers, binary detection`

---

### Task 5: Pass 3 — Content (frontmatter checks, content quality, positive reinforcement)

**Files:**
- Create: `src/passes/content.rs`
- Test: `tests/passes_content.rs` (create)

- [ ] Write failing tests: description >250 chars error, trigger language detection, word-boundary matching (`\bnever\b` vs "whenever"), unknown field as warning, extension field compatibility, `context` must be `fork`, `agent` without `context`, `model-recognized` against known models list, gotchas section (heading + content), body length >300 with escalation
- [ ] Implement `src/passes/content.rs`: `run(ctx, config) -> Result<Vec<Diagnostic>, PipelineError>` — frontmatter checks (adapted from `validator.rs` to produce `Diagnostic`), AST-based content checks on `ctx.prose_text` with regex word boundaries, heading analysis, positive reinforcement (verify substantive content beneath headings). Apply sizeyness escalation.
- [ ] Run tests — all pass
- [ ] Commit: `feat: Pass 3 (Content) with AST-based quality checks and escalation`

---

### Task 6: Pass 4 — References (chain walking, orphan detection)

**Files:**
- Create: `src/passes/references.rs`
- Test: `tests/passes_references.rs` (create)
- Create test fixtures: `tests/fixtures/skills/broken-ref/`, `tests/fixtures/skills/orphaned-files/`, `tests/fixtures/skills/circular-ref/`

- [ ] Write failing tests: link extraction, backtick path extraction, path canonicalization boundary check (reject `../../etc/passwd`), broken refs, orphan detection, circular reference reporting (A→B→A), hop limit diagnostic, LICENSE exclusion, symlink boundary check, hooks-script-missing error
- [ ] Create fixtures: `broken-ref/` (SKILL.md→nonexistent), `orphaned-files/` (unreferenced file), `circular-ref/` (A→B→A cycle)
- [ ] Implement `src/passes/references.rs`: `run(ctx, config) -> Result<Vec<Diagnostic>, PipelineError>` — extract refs from `ctx.links`/backtick paths, canonicalize + boundary check, NFC normalize, walk markdown chain (visited set, hop limit), build reachability set, diff against inventory for orphans, check hooks scripts
- [ ] Run tests — all pass
- [ ] Commit: `feat: Pass 4 (References) with chain walking, orphan detection, path safety`

---

### Task 7: Pass 5 — Security (semgrep integration, remote execution detection)

**Files:**
- Create: `src/passes/security.rs`
- Create: `rules/shell-injection.yaml`, `rules/python-exec.yaml`, `rules/env-exfiltration.yaml`, `rules/hardcoded-urls.yaml`, `rules/filesystem-escape.yaml`
- Test: `tests/passes_security.rs` (create)

- [ ] Write failing tests (no semgrep required): `script-detected` info, `scripts-detected-no-semgrep` suggestion, remote execution pattern detection (`curl | bash`), code block extraction to temp files with cleanup
- [ ] Write bundled semgrep rules in `rules/`: `shell-injection.yaml`, `python-exec.yaml`, `env-exfiltration.yaml`, `hardcoded-urls.yaml`, `filesystem-escape.yaml`. Embed via `include_str!`.
- [ ] Implement `src/passes/security.rs`: `run(ctx, config) -> Result<Vec<Diagnostic>, PipelineError>` — detect semgrep, if available: batch all scripts + temp files from code blocks (`tempfile` crate, 0o600) into single `Command::new("semgrep")` invocation, parse JSON output. If unavailable: emit advisory diagnostics. Always: scan AST for remote execution patterns.
- [ ] Run tests — all pass
- [ ] Commit: `feat: Pass 5 (Security) with optional semgrep integration`

---

### Task 8: Pipeline Orchestration

**Files:**
- Create: `src/pipeline.rs`
- Modify: `src/lib.rs` (add `pub mod pipeline`)
- Test: `tests/pipeline.rs` (create)

- [ ] Write failing tests: full pipeline on valid simple skill, invalid skill (parse fails → stops), moderate multi-file skill (escalation applied), `--strict` exit code behavior
- [ ] Implement `src/pipeline.rs`: `run_pipeline(skill_dir, config) -> PipelineResult` — orchestrate passes 1-5, stop on parse error, apply strict mode, convert `PipelineError` to system diagnostics
- [ ] Run tests — all pass
- [ ] Commit: `feat: pipeline orchestration connecting all five passes`

---

### Task 9: Formatter — Human and JSON Output

**Files:**
- Create: `src/formatter.rs`
- Modify: `src/lib.rs` (add `pub mod formatter`)
- Test: `tests/formatter.rs` (create)

- [ ] Write failing tests: human output emoji markers, severity grouping, doc URLs included, JSON has `schema_version: 2` and `sizeyness_reasons` field, JSON uses `machine_message`, `--severity` filter hides lower tiers
- [ ] Implement `src/formatter.rs`: `format_human()` (warm tone, grouped by severity, doc links) and `format_json()` (spare, `schema_version: 2`)
- [ ] Run tests — all pass
- [ ] Commit: `feat: human and JSON formatters with schema_version`

---

### Task 10: CLI Integration and Migration

**Files:**
- Modify: `src/cli.rs` (add `--strict`, `--output-format`, `--severity`, deprecate `--json`, wire pipeline)
- Modify: `src/lib.rs` (update exports)
- Modify: `src/scan.rs` (wire new pipeline into scan)
- Test: `tests/cli_output.rs` (update)

- [ ] Write failing tests: `--strict` exit codes, `--output-format json` output, deprecated `--json` migration message, `--severity` filtering, validate/scan subcommands use new pipeline
- [ ] Audit existing tests in `tests/validator.rs`, `tests/cli_output.rs`, `tests/fixtures_integration.rs` — ensure every existing test is ported or still compiles against new types. Do not silently drop coverage.
- [ ] Update CLI: add `--strict`, `--output-format` (human/json), `--severity` (info/suggestion/warning/error). `--json` continues to work but emits deprecation warning to stderr pointing to `--output-format json`. Wire validate/scan to `run_pipeline()` + formatter. Keep `ValidationResult` with `#[deprecated]`. All other `lib.rs` public exports (`validate`, `scan`, etc.) remain — mark `validate` as `#[deprecated]`.
- [ ] Update `src/scan.rs`: replace `validate()` with `run_pipeline()`. Pass 5 runs outside rayon — batch semgrep after parallel passes 1-4.
- [ ] Run `cargo test` — all tests pass (including existing)
- [ ] Run `just ensureci-sandbox` — all CI checks pass
- [ ] Commit: `feat: wire pipeline into CLI, add --strict/--output-format/--severity`

---

### Task 11: Bump Version, Update Docs, Final Verification

**Files:**
- Modify: `Cargo.toml` (version 0.1.7 → 0.2.0)
- Modify: `README.md` (document new CLI flags, config system, output tiers)
- Modify: `AGENTS.md` (verify still accurate)
- Modify: `CHANGELOG.md`

- [ ] Bump version in Cargo.toml to 0.2.0
- [ ] Update README.md: severity tiers, sizeyness escalation, new flags, setup command, config format, semgrep, breaking changes
- [ ] Update CHANGELOG.md: 0.2.0 section with breaking changes, new features, migration guide
- [ ] Run `just ensureci-sandbox` — all CI checks pass
- [ ] Commit: `chore: bump to 0.2.0, update docs for layered validation pipeline`

---

### Resolved Decisions

1. **"Complexity tier" → "Sizeyness"** — Rust enum `Sizeyness` with variants `Simple`/`Moderate`/`Hefty`. Human output uses "sizeyness" as the term. Config section: `[sizeyness]`.
2. **Check names are a typed enum** — `pub enum CheckName` serializes to kebab-case strings. Adding a check = adding a variant. Renames are compile errors.
3. **No JSON v1 compat** — clean break. README documents that output schema is subject to change pre-1.0.
4. **No ValidationResult compat shim** — clean break. README documents that library API is subject to change pre-1.0.
