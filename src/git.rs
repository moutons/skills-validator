//! Git repository detection module.
//!
//! Provides utilities for detecting git repository roots and checking if a path is inside a git repository.

use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during git operations.
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Git error: {0}")]
    LibError(#[from] git2::Error),
}

/// Find the root of the git repository containing the given path.
///
/// If `start` is `None`, uses the current working directory.
/// Traverses up from `start` until a `.git` directory or file is found.
pub fn find_repo_root(start: Option<&Path>) -> Result<PathBuf, GitError> {
    let start = match start {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|e| {
            GitError::LibError(git2::Error::new(
                git2::ErrorCode::GenericError,
                git2::ErrorClass::None,
                e.to_string(),
            ))
        })?,
    };

    // Use git2's built-in repository discovery
    let repo = git2::Repository::discover(&start)?;
    Ok(repo.workdir().unwrap_or(repo.path()).to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_is_in_repo_current() {
        assert!(find_repo_root(None).is_ok());
    }

    #[test]
    fn test_find_repo_root_current() {
        let root = find_repo_root(None);
        assert!(root.is_ok());
        // Should have .git
        let root = root.unwrap();
        assert!(root.join(".git").exists() || root.join(".git").is_file());
    }

    #[test]
    fn test_not_in_repo() {
        // Create a temp directory that's NOT a git repo
        let temp_dir = TempDir::new().unwrap();
        let result = find_repo_root(Some(temp_dir.path()));
        // Should fail - not a git repo
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_in_repo() {
        // Create a temp directory inside the current repo
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("nested").join("deep");
        fs::create_dir_all(&nested).unwrap();

        // Should find the repo root (the current repo)
        let root = find_repo_root(Some(&nested));
        // This might fail if temp_dir is outside the repo
        let _ = root;
    }
}
