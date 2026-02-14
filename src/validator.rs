use std::path::Path;
use unicode_normalization::UnicodeNormalization;

use crate::parser::{find_skill_md, parse_frontmatter_and_body};

const MAX_SKILL_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 1024;
const MAX_COMPATIBILITY_LENGTH: usize = 500;

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
            result.warnings.push(format!(
                "Good: Found '{}' in skill content. {}",
                keyword, guidance
            ));
        } else {
            result.warnings.push(format!(
                "Warning: '{}' not found in skill content. {}",
                keyword, guidance
            ));
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
            }
            Err(e) => result.errors.push(e.to_string()),
        },
        Err(e) => result.errors.push(e.to_string()),
    }

    result
}
