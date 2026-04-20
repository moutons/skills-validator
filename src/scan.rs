use rayon::prelude::*;
use std::path::PathBuf;

use crate::config::ValidatorConfig;
use crate::discovery::{discover_skills, DiscoveredSkill};
use crate::git::find_repo_root;
use crate::models::Severity;
use crate::paths::{expand_path, PathsConfig};
use crate::pipeline::{run_pipeline, PipelineResult};

/// Result of a full scan operation.
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// All skills discovered and validated
    pub skills: Vec<SkillValidation>,
    /// Total count of skills found
    pub total_skills: usize,
    /// Count of valid skills
    pub valid_count: usize,
    /// Count of invalid skills
    pub invalid_count: usize,
    /// Count of skills with warnings
    pub warning_count: usize,
    /// Directories that were scanned
    pub scanned_dirs: Vec<PathBuf>,
    /// Directories that were skipped
    pub skipped_dirs: Vec<PathBuf>,
}

/// A skill with its validation result.
#[derive(Debug, Clone)]
pub struct SkillValidation {
    /// The discovered skill
    pub skill: DiscoveredSkill,
    /// Pipeline result from validation
    pub pipeline_result: PipelineResult,
    /// Is the skill valid?
    pub is_valid: bool,
}

/// Scan mode determining which directories to scan.
#[derive(Debug, Clone, Default)]
pub struct ScanOptions {
    /// Scan all locations (repo + user home)
    pub all: bool,
    /// Scan only user home directories
    pub user: bool,
    /// Scan only repository root
    pub repo: bool,
    /// Scan specific tools only
    pub tools: Vec<String>,
    /// Include verbose output
    #[allow(dead_code)] // Will be used for verbose scan output
    pub verbose: bool,
}

/// Perform a scan based on the given options.
pub fn scan(options: &ScanOptions) -> ScanResult {
    let mut result = ScanResult::default();

    // Load paths configuration
    let config = match PathsConfig::load() {
        Ok(c) => c,
        Err(e) => {
            log::error!("Failed to load paths configuration: {}", e);
            return result;
        }
    };

    // Determine directories to scan
    let mut directories: Vec<(String, PathBuf)> = Vec::new();

    // Get repository root if needed
    let repo_root = if options.repo || options.all {
        find_repo_root(None).ok()
    } else {
        None
    };

    // Add user home directories
    if options.user || options.all {
        for tool_name in config.tool_names() {
            if !options.tools.is_empty() && !options.tools.contains(&tool_name) {
                continue;
            }

            if let Some(tool) = config.get_tool(&tool_name) {
                for dir_template in &tool.directories {
                    // Only expand $HOME based paths for user scan
                    if dir_template.contains("$HOME") || dir_template.contains("~") {
                        if let Ok(expanded) = expand_path(dir_template, None) {
                            directories.push((tool_name.clone(), expanded));
                        }
                    }
                }
            }
        }
    }

    // Add repository root directories
    if options.repo || options.all {
        if let Some(ref root) = repo_root {
            for tool_name in config.tool_names() {
                if !options.tools.is_empty() && !options.tools.contains(&tool_name) {
                    continue;
                }

                if let Some(tool) = config.get_tool(&tool_name) {
                    for dir_template in &tool.directories {
                        // Only expand $REPO_ROOT based paths for repo scan
                        if dir_template.contains("$REPO_ROOT") {
                            if let Ok(expanded) = expand_path(dir_template, Some(root)) {
                                directories.push((tool_name.clone(), expanded));
                            }
                        }
                    }
                }
            }
        }
    }

    // Add specific tools if requested
    if !options.tools.is_empty() {
        for tool_name in &options.tools {
            if let Some(tool) = config.get_tool(tool_name) {
                for dir_template in &tool.directories {
                    let expanded = if dir_template.contains("$REPO_ROOT") {
                        expand_path(dir_template, repo_root.as_ref())
                    } else {
                        expand_path(dir_template, None)
                    };

                    if let Ok(path) = expanded {
                        directories.push((tool_name.clone(), path));
                    }
                }
            }
        }
    }

    // Discover skills
    let discovery = discover_skills(&directories);

    // Track scanned and skipped directories
    result.scanned_dirs = directories.iter().map(|(_, p)| p.clone()).collect();
    result.skipped_dirs = discovery.skipped_dirs;
    result.total_skills = discovery.skills.len();

    if result.total_skills == 0 {
        return result;
    }

    // Validate skills in parallel using the new pipeline
    let validator_config = ValidatorConfig::default();
    let skills: Vec<SkillValidation> = discovery
        .skills
        .into_par_iter()
        .map(|skill| {
            let pipeline_result = run_pipeline(skill.directory.as_path(), &validator_config);
            let has_errors = pipeline_result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error);

            SkillValidation {
                is_valid: !has_errors,
                skill,
                pipeline_result,
            }
        })
        .collect();

    // Process results
    for s in &skills {
        if s.is_valid {
            result.valid_count += 1;
            let has_warnings = s
                .pipeline_result
                .diagnostics
                .iter()
                .any(|d| d.severity == Severity::Warning);
            if has_warnings {
                result.warning_count += 1;
            }
        } else {
            result.invalid_count += 1;
        }
    }

    result.skills = skills;
    result
}

/// Find duplicate skills by directory name.
pub fn find_duplicates(result: &ScanResult) -> Vec<Vec<&SkillValidation>> {
    use std::collections::HashMap;

    let mut by_name: HashMap<String, Vec<&SkillValidation>> = HashMap::new();

    for skill in &result.skills {
        // Use the directory name as the skill identifier
        let name = skill
            .skill
            .directory
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());
        by_name.entry(name).or_default().push(skill);
    }

    by_name
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(_, v)| v)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_options_default() {
        let opts = ScanOptions::default();
        assert!(!opts.all);
        assert!(!opts.user);
        assert!(!opts.repo);
        assert!(opts.tools.is_empty());
    }
}
