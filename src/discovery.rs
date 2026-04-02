//! Skill discovery module.
//!
//! Discovers SKILL.md files within directory trees.

use std::path::PathBuf;
use walkdir::WalkDir;

/// A discovered skill file.
#[derive(Debug, Clone)]
pub struct DiscoveredSkill {
    /// Path to the SKILL.md file
    #[allow(dead_code)] // Will be used for output formatting
    pub path: PathBuf,
    /// Name of the tool this skill belongs to
    #[allow(dead_code)] // Will be used for output formatting
    pub tool_name: String,
    /// The directory that was scanned (parent of skill)
    pub directory: PathBuf,
}

/// Result of a discovery operation.
#[derive(Debug, Clone, Default)]
pub struct DiscoveryResult {
    /// All discovered skills
    pub skills: Vec<DiscoveredSkill>,
    /// Directories that were skipped (don't exist or not accessible)
    pub skipped_dirs: Vec<PathBuf>,
}

/// Discover all SKILL.md files in the given directories.
///
/// `directories` is a slice of (tool_name, expanded_path) tuples.
/// Each directory is walked recursively to find all SKILL.md files.
pub fn discover_skills(directories: &[(String, PathBuf)]) -> DiscoveryResult {
    let mut result = DiscoveryResult::default();

    for (tool_name, dir_path) in directories {
        if !dir_path.exists() {
            log::debug!("Skipping non-existent directory: {:?}", dir_path);
            result.skipped_dirs.push(dir_path.clone());
            continue;
        }

        // Walk the directory tree
        let walker = WalkDir::new(dir_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok());

        for entry in walker {
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    if filename == "SKILL.md" {
                        // Get the parent directory (the skill root)
                        let directory = path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| dir_path.clone());

                        result.skills.push(DiscoveredSkill {
                            path: path.to_path_buf(),
                            tool_name: tool_name.clone(),
                            directory,
                        });
                    }
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn discover_in_dir(dir: &Path, tool_name: &str) -> DiscoveryResult {
        discover_skills(&[(tool_name.to_string(), dir.to_path_buf())])
    }

    #[test]
    fn test_discover_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = discover_in_dir(temp_dir.path(), "test");

        assert!(result.skills.is_empty());
    }

    #[test]
    fn test_discover_single_skill() {
        let temp_dir = TempDir::new().unwrap();
        let skill_dir = temp_dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();

        let skill_file = skill_dir.join("SKILL.md");
        fs::write(&skill_file, "---\nname: test-skill\n---\n").unwrap();

        let result = discover_in_dir(temp_dir.path(), "test-tool");

        assert_eq!(result.skills.len(), 1);
        assert!(result.skills[0].path.ends_with("SKILL.md"));
        assert_eq!(result.skills[0].tool_name, "test-tool");
    }

    #[test]
    fn test_discover_nested_skills() {
        let temp_dir = TempDir::new().unwrap();

        let skill1 = temp_dir.path().join("skill1");
        let skill2 = temp_dir.path().join("skill2");
        fs::create_dir_all(&skill1).unwrap();
        fs::create_dir_all(&skill2).unwrap();

        fs::write(skill1.join("SKILL.md"), "---\nname: skill1\n---\n").unwrap();
        fs::write(skill2.join("SKILL.md"), "---\nname: skill2\n---\n").unwrap();

        let result = discover_in_dir(temp_dir.path(), "test");

        assert_eq!(result.skills.len(), 2);
    }

    #[test]
    fn test_discover_skips_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent = temp_dir.path().join("does-not-exist");

        let result = discover_in_dir(&nonexistent, "test");

        assert!(result.skills.is_empty());
        assert!(result.skipped_dirs.contains(&nonexistent));
    }
}
