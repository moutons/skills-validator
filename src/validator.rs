#![allow(clippy::only_used_in_recursion)]

use std::path::Path;
use unicode_normalization::UnicodeNormalization;

use crate::parser::{find_skill_md, parse_frontmatter_and_body};

const MAX_SKILL_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;
const MAX_COMPATIBILITY_LENGTH: usize = 500;
const MAX_SKILL_BODY_LINES: usize = 500;

const ALLOWED_FIELDS: &[&str] = &[
    "name",
    "description",
    "license",
    "allowed-tools",
    "metadata",
    "compatibility",
];

const CLAUDE_CODE_EXTENSIONS: &[&str] = &[
    "argument-hint",
    "disable-model-invocation",
    "user-invocable",
    "model",
    "context",
    "agent",
    "hooks",
];

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_name(name: &str, skill_dir: Option<&Path>) -> ValidationResult {
    let mut result = ValidationResult::new();

    if name.is_empty() {
        result
            .errors
            .push("Field 'name' must be a non-empty string".to_string());
        return result;
    }

    let normalized: String = name.nfkc().collect();
    let name = normalized.trim();

    if name.len() > MAX_SKILL_NAME_LENGTH {
        result.errors.push(format!(
            "Skill name '{}' exceeds {} character limit ({} chars)",
            name,
            MAX_SKILL_NAME_LENGTH,
            name.len()
        ));
    }

    if name != name.to_lowercase() {
        result
            .errors
            .push(format!("Skill name '{}' must be lowercase", name));
    }

    if name.starts_with('-') || name.ends_with('-') {
        result
            .errors
            .push("Skill name cannot start or end with a hyphen".to_string());
    }

    if name.contains("--") {
        result
            .errors
            .push("Skill name cannot contain consecutive hyphens".to_string());
    }

    if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
        result.errors.push(format!(
            "Skill name '{}' contains invalid characters. Only letters, digits, and hyphens are allowed.",
            name
        ));
    }

    if let Some(dir) = skill_dir {
        let dir_name: String = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .nfkc()
            .collect();
        if dir_name != name {
            result.errors.push(format!(
                "Directory name '{}' must match skill name '{}'",
                dir_name, name
            ));
        }
    }

    result
}

fn validate_description(description: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    if description.trim().is_empty() {
        result
            .errors
            .push("Field 'description' must be a non-empty string".to_string());
        return result;
    }

    if description.len() > MAX_DESCRIPTION_LENGTH {
        result.errors.push(format!(
            "Description exceeds {} character limit ({} chars)",
            MAX_DESCRIPTION_LENGTH,
            description.len()
        ));
    }

    result
}

fn validate_compatibility(compatibility: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    if compatibility.len() > MAX_COMPATIBILITY_LENGTH {
        result.errors.push(format!(
            "Compatibility exceeds {} character limit ({} chars)",
            MAX_COMPATIBILITY_LENGTH,
            compatibility.len()
        ));
    }

    result
}

fn validate_metadata_fields(metadata: &serde_yaml::Mapping) -> ValidationResult {
    let mut result = ValidationResult::new();

    let present_fields: std::collections::HashSet<&str> =
        metadata.keys().filter_map(|k| k.as_str()).collect();

    let spec_fields: std::collections::HashSet<&str> = ALLOWED_FIELDS.iter().cloned().collect();

    let claude_code_fields: std::collections::HashSet<&str> =
        CLAUDE_CODE_EXTENSIONS.iter().cloned().collect();

    for field in &present_fields {
        if spec_fields.contains(field) {
            continue;
        }
        if claude_code_fields.contains(field) {
            result.warnings.push(format!(
                "Field '{}' is a Claude Code extension (not in official spec). See https://code.claude.com/docs/en/skills",
                field
            ));
        } else {
            result.errors.push(format!(
                "Unexpected field in frontmatter: '{}'. Only fields defined in the official spec are allowed. See https://agentskills.io/specification",
                field
            ));
        }
    }

    result
}

pub fn validate_metadata(
    metadata: &serde_yaml::Mapping,
    skill_dir: Option<&Path>,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    let meta_result = validate_metadata_fields(metadata);
    result.errors.extend(meta_result.errors);
    result.warnings.extend(meta_result.warnings);

    let name_val = metadata.get(serde_yaml::Value::String("name".to_string()));
    if name_val.is_none() {
        result
            .errors
            .push("Missing required field in frontmatter: name".to_string());
    } else if let Some(name) = name_val.and_then(|v| v.as_str()) {
        let name_result = validate_name(name, skill_dir);
        result.errors.extend(name_result.errors);
        result.warnings.extend(name_result.warnings);
    }

    let desc_val = metadata.get(serde_yaml::Value::String("description".to_string()));
    if desc_val.is_none() {
        result
            .errors
            .push("Missing required field in frontmatter: description".to_string());
    } else if let Some(desc) = desc_val.and_then(|v| v.as_str()) {
        let desc_result = validate_description(desc);
        result.errors.extend(desc_result.errors);
        result.warnings.extend(desc_result.warnings);
    }

    if let Some(compat) = metadata.get(serde_yaml::Value::String("compatibility".to_string())) {
        if let Some(compat_str) = compat.as_str() {
            let compat_result = validate_compatibility(compat_str);
            result.errors.extend(compat_result.errors);
            result.warnings.extend(compat_result.warnings);
        }
    }

    result
}

fn validate_content_keywords(body: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    let body_lower = body.to_lowercase();

    let keywords = [
        (
            "never",
            "A well-written skill includes clear directives to NEVER do something and preferably ALWAYS do an alternative. See https://agentskills.io/what-are-skills",
        ),
        (
            "always",
            "A well-written skill includes clear directives to ALWAYS do something in certain circumstances. See https://agentskills.io/what-are-skills",
        ),
        (
            "when",
            "A well-written skill contains 'when' statements to inform the agent of what conditions trigger certain behaviors. See https://code.claude.com/docs/en/skills",
        ),
        (
            "example",
            "A well-written skill contains examples to inform the agent of what to do in commonly encountered circumstances. See https://opencode.ai/docs/skills",
        ),
    ];

    for (keyword, guidance) in keywords {
        if body_lower.contains(keyword) {
            log::debug!("Good: Found '{}' in skill content. {}", keyword, guidance);
        } else {
            result.warnings.push(format!(
                "'{}' not found in skill content. {}",
                keyword, guidance
            ));
        }
    }

    result
}

fn validate_body_length(body: &str) -> ValidationResult {
    let mut result = ValidationResult::new();

    let line_count = body.lines().count();
    if line_count > MAX_SKILL_BODY_LINES {
        result.warnings.push(format!(
            "SKILL.md body has {} lines (recommended: {} or fewer). Consider using progressive disclosure patterns to keep skills focused. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#progressive-disclosure-patterns",
            line_count, MAX_SKILL_BODY_LINES
        ));
    }

    result
}

fn validate_windows_paths(skill_dir: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();

    fn check_directory(dir: &Path, result: &mut ValidationResult) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();

                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if ext == "md"
                            || ext == "txt"
                            || ext == "yaml"
                            || ext == "yml"
                            || ext == "json"
                            || ext == "toml"
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                for (line_num, line) in content.lines().enumerate() {
                                    let line_lower = line.to_lowercase();
                                    if (line_lower.contains(":\\") || line_lower.contains(":/"))
                                        && line_lower.contains(":\\")
                                    {
                                        result.warnings.push(format!(
                                                "Windows-style path found in {} (line {}). Use forward slashes for cross-platform compatibility. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#avoid-windows-style-paths",
                                                path.file_name().unwrap_or_default().to_string_lossy(),
                                                line_num + 1
                                            ));
                                        break;
                                    }
                                    if line.contains("\\\\") {
                                        result.warnings.push(format!(
                                            "UNC path found in {} (line {}). Use forward slashes for cross-platform compatibility. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices#avoid-windows-style-paths",
                                            path.file_name().unwrap_or_default().to_string_lossy(),
                                            line_num + 1
                                        ));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                } else if path.is_dir() {
                    check_directory(&path, result);
                }
            }
        }
    }

    check_directory(skill_dir, &mut result);

    result
}

fn validate_no_scripts_in_base(skill_dir: &Path) -> ValidationResult {
    let mut result = ValidationResult::new();

    let script_extensions = ["sh", "py", "ps1", "bat", "cmd"];

    if let Ok(entries) = std::fs::read_dir(skill_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if script_extensions.contains(&ext) {
                        result.warnings.push(format!(
                            "Script file '{}' found in skill root directory. Consider organizing scripts in a dedicated directory. See https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview#level-3-resources-and-code-loaded-as-needed and https://agentskills.io/specification#optional-directories",
                            path.file_name().unwrap_or_default().to_string_lossy()
                        ));
                    }
                }
            }
        }
    }

    result
}

pub fn validate(skill_dir: &Path) -> ValidationResult {
    let skill_dir = skill_dir.to_path_buf();
    let mut result = ValidationResult::new();

    if !skill_dir.exists() {
        result
            .errors
            .push(format!("Path does not exist: {:?}", skill_dir));
        return result;
    }

    if !skill_dir.is_dir() {
        result
            .errors
            .push(format!("Not a directory: {:?}", skill_dir));
        return result;
    }

    let skill_md = find_skill_md(&skill_dir);
    if skill_md.is_none() {
        result
            .errors
            .push("Missing required file: SKILL.md".to_string());
        return result;
    }

    let skill_md = skill_md.unwrap();

    match std::fs::read_to_string(&skill_md) {
        Ok(content) => match parse_frontmatter_and_body(&content) {
            Ok((map, body)) => {
                let validation_result = validate_metadata(&map, Some(&skill_dir));
                result.errors = validation_result.errors;
                result.warnings = validation_result.warnings;

                let keyword_result = validate_content_keywords(&body);
                result.warnings.extend(keyword_result.warnings);

                let body_length_result = validate_body_length(&body);
                result.warnings.extend(body_length_result.warnings);
            }
            Err(e) => result.errors.push(e.to_string()),
        },
        Err(e) => result.errors.push(e.to_string()),
    }

    let windows_path_result = validate_windows_paths(&skill_dir);
    result.warnings.extend(windows_path_result.warnings);

    let scripts_result = validate_no_scripts_in_base(&skill_dir);
    result.warnings.extend(scripts_result.warnings);

    result
}
