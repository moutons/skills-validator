# Git Repository Detection

## Goal
Detect if the current working directory is inside a git repository and return the repository root path.

## Context
The `--repo` and `--all` scan modes require knowing the git repository root to expand `$REPO_ROOT` variables. We use `git2` crate for this.

**Function signature:**
```rust
pub fn find_repo_root(start: Option<&Path>) -> Result<PathBuf, GitError>;

// Convenience wrapper:
pub fn is_in_repo() -> bool;
```

**Behavior:**
- `find_repo_root(None)` starts from current directory
- `find_repo_root(Some(path))` starts from given path
- Returns `GitError::NotARepository` if no `.git` found in ancestry

## User Stories

**US-001:** Find repo root from CWD
As a user inside a git repo, I want the scanner to automatically detect the repo root.

**US-002:** Error outside repo
As a user outside any git repo, I want `--repo` to fail with a clear message.

**US-003:** Handle subdirectories
As a user in a nested subdirectory, I want repo detection to traverse up to find `.git`.

## Acceptance Criteria
- [ ] `find_repo_root(None)` returns `PathBuf` when called from inside a repo
- [ ] `find_repo_root(None)` returns `GitError::NotARepository` when called outside any repo
- [ ] `is_in_repo()` returns `true` in a repo, `false` otherwise
- [ ] Works correctly from nested subdirectories (traverses up to repo root)
- [ ] Unit tests use temp git repos created via `git2`

## Completion Signal
<promise>DONE</promise>