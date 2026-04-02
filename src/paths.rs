//! Paths configuration module.
//!
//! Parses and exposes the embedded `paths.jsonc` configuration with tool directory templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Embedded paths configuration from paths.jsonc
const PATHS_JSONC: &str = include_str!("../paths.jsonc");

/// Configuration for a single tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Display name of the tool
    pub name: String,
    /// Documentation URL
    pub documentation: Option<String>,
    /// Directory templates (e.g., "$HOME/.claude/skills")
    pub directories: Vec<String>,
}

/// Paths configuration containing all tool mappings.
#[derive(Debug, Clone)]
pub struct PathsConfig {
    /// Tool configurations, keyed by kebab-case name
    pub tools: HashMap<String, ToolConfig>,
}

impl PathsConfig {
    /// Parse the embedded paths.jsonc configuration.
    pub fn load() -> Result<Self, PathsError> {
        // Strip JSONC comments before parsing
        let json_str = strip_json_comments(PATHS_JSONC);

        let parsed: serde_json::Value =
            serde_json::from_str(&json_str).map_err(PathsError::ParseError)?;

        let mut tools = HashMap::new();

        if let Some(obj) = parsed.as_object() {
            for (key, value) in obj {
                // Skip the _unsupported section
                if key == "_unsupported" {
                    continue;
                }

                let tool_config: ToolConfig =
                    serde_json::from_value(value.clone()).map_err(PathsError::ParseError)?;

                // Normalize key to kebab-case
                let kebab_key = to_kebab_case(key);
                tools.insert(kebab_key, tool_config);
            }
        }

        Ok(Self { tools })
    }

    /// Get a tool config by name (case-insensitive).
    pub fn get_tool(&self, name: &str) -> Option<&ToolConfig> {
        let kebab_name = to_kebab_case(name);
        self.tools.get(&kebab_name)
    }

    /// Get all available tool names (sorted).
    pub fn tool_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.tools.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if a tool exists.
    pub fn has_tool(&self, name: &str) -> bool {
        let kebab_name = to_kebab_case(name);
        self.tools.contains_key(&kebab_name)
    }
}

/// Errors that can occur during path operations.
#[derive(Debug, thiserror::Error)]
pub enum PathsError {
    #[error("Failed to parse paths.jsonc: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Home directory not found")]
    HomeNotFound,

    #[error("Repository root not provided but required by path template")]
    RepoRootNotProvided,

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Expand path variables in a template string.
///
/// - `$HOME` and `~` are expanded to the user's home directory
/// - `$REPO_ROOT` is expanded to the git repository root (if provided)
pub fn expand_path(template: &str, repo_root: Option<&PathBuf>) -> Result<PathBuf, PathsError> {
    let mut result = template.to_string();

    // Expand ~ first (must be at start of path or after /)
    if result.contains('~') {
        let home = dirs::home_dir().ok_or(PathsError::HomeNotFound)?;
        let home_str = home.to_string_lossy();
        result = result.replace("~", &home_str);
    }

    // Expand $HOME
    if result.contains("$HOME") {
        let home = dirs::home_dir().ok_or(PathsError::HomeNotFound)?;
        let home_str = home.to_string_lossy();
        result = result.replace("$HOME", &home_str);
    }

    // Expand $REPO_ROOT
    if result.contains("$REPO_ROOT") {
        let root = repo_root.ok_or(PathsError::RepoRootNotProvided)?;
        let root_str = root.to_string_lossy();
        result = result.replace("$REPO_ROOT", &root_str);
    }

    // Expand $CWD (current working directory)
    if result.contains("$CWD") {
        let cwd = std::env::current_dir().map_err(|e| PathsError::InvalidPath(e.to_string()))?;
        let cwd_str = cwd.to_string_lossy();
        result = result.replace("$CWD", &cwd_str);
    }

    Ok(PathBuf::from(result))
}

/// Convert a string to kebab-case.
/// If the input already contains hyphens (like "claude-code"), just lowercase it.
/// Otherwise, splits on uppercase, underscores, and spaces.
fn to_kebab_case(s: &str) -> String {
    // If already kebab-case, just lowercase
    if s.contains('-') {
        return s.to_lowercase();
    }

    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else if c == '_' || c == ' ' {
            if !result.is_empty() && !result.ends_with('-') {
                result.push('-');
            }
        } else {
            result.push(c);
        }
    }
    // Collapse multiple hyphens
    while result.contains("--") {
        result = result.replace("--", "-");
    }
    // Remove leading/trailing hyphens
    result.trim_matches('-').to_string()
}

/// Strip JSONC comments from a string.
/// Handles both // and /* */ style comments, but NOT when inside strings.
fn strip_json_comments(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if c == '\\' {
            // Escape sequence - add both the backslash and next char
            result.push(c);
            if let Some(&next) = chars.peek() {
                result.push(next);
                chars.next();
            }
        } else if c == '"' {
            // Toggle string state
            in_string = !in_string;
            result.push(c);
        } else if !in_string && c == '/' {
            match chars.peek() {
                Some(&'/') => {
                    // Single-line comment: skip to end of line
                    chars.next();
                    while let Some(&next) = chars.peek() {
                        chars.next();
                        if next == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                }
                Some(&'*') => {
                    // Multi-line comment
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '*' {
                            if let Some(&'/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                }
                _ => {
                    result.push(c);
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_paths_config() {
        let config = PathsConfig::load().expect("Failed to load paths config");
        assert!(!config.tools.is_empty());
        let claude = config
            .get_tool("claude-code")
            .expect("claude-code not found");
        assert_eq!(claude.name, "Claude Code");
        assert!(!claude.directories.is_empty());
    }

    #[test]
    fn test_case_insensitive_lookup() {
        let config = PathsConfig::load().expect("Failed to load paths config");
        assert!(config.get_tool("claude-code").is_some());
        assert!(config.get_tool("CLAUDE-CODE").is_some());
        assert!(config.get_tool("Claude-Code").is_some());
    }

    #[test]
    fn test_tool_names_sorted() {
        let config = PathsConfig::load().expect("Failed to load paths config");
        let names = config.tool_names();
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn test_has_tool() {
        let config = PathsConfig::load().expect("Failed to load paths config");
        assert!(config.has_tool("claude-code"));
        assert!(!config.has_tool("unknown-tool"));
    }

    #[test]
    fn test_expand_home() {
        let result = expand_path("$HOME/.claude/skills", None).expect("Failed to expand $HOME");
        let home = dirs::home_dir().unwrap();
        assert!(result.starts_with(home));
        assert!(result.to_string_lossy().ends_with(".claude/skills"));
    }

    #[test]
    fn test_expand_tilde() {
        let result = expand_path("~/.claude/skills", None).expect("Failed to expand ~");
        let home = dirs::home_dir().unwrap();
        assert!(result.starts_with(home));
        assert!(result.to_string_lossy().ends_with(".claude/skills"));
    }

    #[test]
    fn test_expand_repo_root() {
        let repo = PathBuf::from("/test/repo");
        let result = expand_path("$REPO_ROOT/.agent/skills", Some(&repo))
            .expect("Failed to expand $REPO_ROOT");
        assert_eq!(result, PathBuf::from("/test/repo/.agent/skills"));
    }

    #[test]
    fn test_expand_repo_root_not_provided() {
        let result = expand_path("$REPO_ROOT/.agent/skills", None);
        assert!(matches!(result, Err(PathsError::RepoRootNotProvided)));
    }

    #[test]
    fn test_expand_multiple_vars() {
        let repo = PathBuf::from("/my/repo");
        let result = expand_path("$HOME/skills:$REPO_ROOT/skills", Some(&repo))
            .expect("Failed to expand multiple vars");
        let home = dirs::home_dir().unwrap();
        let result_str = result.to_string_lossy();
        let home_str = home.to_string_lossy();
        assert!(result_str.starts_with(home_str.as_ref()));
        assert!(result_str.contains("/my/repo/skills"));
    }

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(to_kebab_case("ClaudeCode"), "claude-code");
        assert_eq!(to_kebab_case("claude_code"), "claude-code");
        assert_eq!(to_kebab_case("claude code"), "claude-code");
        // Already kebab-case - just lowercase
        assert_eq!(to_kebab_case("GitHubCopilot"), "git-hub-copilot");
        // All caps with hyphen
        assert_eq!(to_kebab_case("CLAUDE-CODE"), "claude-code");
    }

    #[test]
    fn test_strip_json_comments() {
        let input = r#"
{
    // This is a comment
    "key": "value",
    /* Multi-line
       comment */
    "key2": "value2"
}
"#;
        let result = strip_json_comments(input);
        assert!(!result.contains("//"));
        assert!(!result.contains("/*"));
        assert!(result.contains("\"key\""));
        assert!(result.contains("\"key2\""));
    }
}
