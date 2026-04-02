# Integration Tests

## Goal
Create integration tests for the complete scan workflow using real filesystem operations with temp directories and git repos.

## Context
Integration tests use `tempfile` crate to create isolated test environments. Tests create actual directory structures, git repos (via `git2`), and SKILL.md files.

**Test file location:** `tests/scan_integration.rs`

**Test categories:**
1. `--user` mode: scan $HOME paths
2. `--repo` mode: scan repo root paths
3. `--tool` mode: scan specific tools
4. `--all` mode: combined scan
5. `--dry-run` mode: discovery only
6. Error conditions: invalid tool, outside repo, permissions

**Test utilities needed:**
```rust
// In tests/common/mod.rs or similar
pub fn create_temp_skill(dir: &Path, name: &str, content: &str) -> PathBuf;
pub fn create_temp_git_repo(dir: &Path) -> Repository;
pub fn create_temp_paths_config(dir: &Path, tools: &[(&str, &[&str])]) -> PathBuf;
```

## User Stories

**US-001:** Test --user scan end-to-end
As a developer, I want an integration test that creates temp skills in $HOME paths and verifies they're found.

**US-002:** Test --repo requires git
As a developer, I want a test that verifies `--repo` fails when outside a git repo.

**US-003:** Test --tool error handling
As a developer, I want tests for unknown tools, missing directories, and partial availability.

**US-004:** Test duplicate detection
As a developer, I want a test that creates same-named skills and verifies duplicate warning.

**US-005:** Test exit codes
As a developer, I want tests verifying each exit code scenario.

## Acceptance Criteria
- [ ] `tests/scan_integration.rs` exists with integration tests
- [ ] Tests use `tempfile::tempdir()` for isolation
- [ ] Tests create real git repos via `git2`
- [ ] `cargo test --test scan_integration` passes
- [ ] Coverage: --user, --repo, --tool, --all, --dry-run, error cases
- [ ] No tests depend on user's actual filesystem state

## Completion Signal
<promise>DONE</promise>