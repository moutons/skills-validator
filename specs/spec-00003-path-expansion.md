# Path Expansion

## Goal
Expand template variables `$HOME`, `~`, and `$REPO_ROOT` in directory paths to absolute filesystem paths.

## Context
Path templates from `paths.jsonc` contain variables that must be resolved at runtime:
- `$HOME` and `~` → User's home directory via `dirs` crate
- `$REPO_ROOT` → Git repository root via `git2` (passed from caller)

**Function signature:**
```rust
pub fn expand_path(template: &str, repo_root: Option<&Path>) -> Result<PathBuf, PathError>;
```

**Error handling:**
- `$HOME`/`~` expansion fails → return `PathError::HomeNotFound`
- `$REPO_ROOT` used but `repo_root` is `None` → return `PathError::RepoRootNotProvided`
- Invalid path characters → return `PathError::InvalidPath`

## User Stories

**US-001:** Expand $HOME
As a user, I want `$HOME/.claude/skills` to resolve to `/Users/me/.claude/skills` on macOS.

**US-002:** Expand tilde
As a user, I want `~/.claude/skills` to work identically to `$HOME/.claude/skills`.

**US-003:** Expand $REPO_ROOT
As a user, I want `$REPO_ROOT/.agent/skills` to resolve when running inside a git repo.

**US-004:** Handle missing repo root gracefully
As a user, when `$REPO_ROOT` is in a template but no repo is detected, I want a clear error.

## Acceptance Criteria
- [ ] `expand_path("$HOME/.claude/skills", None)` returns valid `PathBuf`
- [ ] `expand_path("~/.claude/skills", None)` returns same result as `$HOME`
- [ ] `expand_path("$REPO_ROOT/.agent/skills", Some(Path::new("/repo")))` returns `/repo/.agent/skills`
- [ ] `expand_path("$REPO_ROOT/.agent/skills", None)` returns `PathError::RepoRootNotProvided`
- [ ] Unit tests cover: no variables, multiple variables, mixed separators, empty string

## Completion Signal
<promise>DONE</promise>