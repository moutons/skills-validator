use skills_validator::models::{
    CheckName, CodeBlock, Diagnostic, FileEntry, FileType, Heading, Link, PipelineError, Severity,
    Sizeyness, SkillContext,
};
use std::path::PathBuf;

// === Severity ordering tests ===

#[test]
fn test_severity_ordering() {
    assert!(Severity::Info < Severity::Suggestion);
    assert!(Severity::Suggestion < Severity::Warning);
    assert!(Severity::Warning < Severity::Error);
    assert!(Severity::Info < Severity::Error);
}

#[test]
fn test_severity_equality() {
    assert_eq!(Severity::Info, Severity::Info);
    assert_eq!(Severity::Error, Severity::Error);
    assert_ne!(Severity::Info, Severity::Error);
}

// === Sizeyness tests ===

#[test]
fn test_sizeyness_simple_one_file() {
    assert_eq!(Sizeyness::from_counts(1, 0, false), Sizeyness::Simple);
}

#[test]
fn test_sizeyness_simple_two_files() {
    assert_eq!(Sizeyness::from_counts(2, 0, false), Sizeyness::Simple);
}

#[test]
fn test_sizeyness_moderate_three_files() {
    assert_eq!(Sizeyness::from_counts(3, 0, false), Sizeyness::Moderate);
}

#[test]
fn test_sizeyness_moderate_five_files() {
    assert_eq!(Sizeyness::from_counts(5, 0, false), Sizeyness::Moderate);
}

#[test]
fn test_sizeyness_moderate_one_subdir() {
    assert_eq!(Sizeyness::from_counts(1, 1, false), Sizeyness::Moderate);
}

#[test]
fn test_sizeyness_moderate_two_subdirs() {
    assert_eq!(Sizeyness::from_counts(1, 2, false), Sizeyness::Moderate);
}

#[test]
fn test_sizeyness_hefty_six_files() {
    assert_eq!(Sizeyness::from_counts(6, 0, false), Sizeyness::Hefty);
}

#[test]
fn test_sizeyness_hefty_three_subdirs() {
    assert_eq!(Sizeyness::from_counts(1, 3, false), Sizeyness::Hefty);
}

#[test]
fn test_sizeyness_hefty_orchestration() {
    assert_eq!(Sizeyness::from_counts(1, 0, true), Sizeyness::Hefty);
}

// === Diagnostic construction tests ===

#[test]
fn test_diagnostic_construction() {
    let diag = Diagnostic {
        severity: Severity::Warning,
        check_name: CheckName::NameMissing,
        human_message: "Your skill is missing a name field.".to_string(),
        machine_message: "name field missing from frontmatter".to_string(),
        doc_url: Some("https://agentskills.io/spec#name".to_string()),
        file_path: Some(PathBuf::from("skill.md")),
        base_severity: Severity::Suggestion,
    };
    assert_eq!(diag.severity, Severity::Warning);
    assert_eq!(diag.check_name, CheckName::NameMissing);
    assert!(diag.doc_url.is_some());
    assert!(diag.file_path.is_some());
    assert_eq!(diag.base_severity, Severity::Suggestion);
}

#[test]
fn test_diagnostic_without_optionals() {
    let diag = Diagnostic {
        severity: Severity::Error,
        check_name: CheckName::FrontmatterPresent,
        human_message: "No frontmatter found.".to_string(),
        machine_message: "frontmatter missing".to_string(),
        doc_url: None,
        file_path: None,
        base_severity: Severity::Error,
    };
    assert!(diag.doc_url.is_none());
    assert!(diag.file_path.is_none());
}

// === Severity escalation tests ===

#[test]
fn test_escalate_zero_levels() {
    use skills_validator::models::escalate;
    assert_eq!(escalate(Severity::Suggestion, 0), Severity::Suggestion);
}

#[test]
fn test_escalate_one_level() {
    use skills_validator::models::escalate;
    assert_eq!(escalate(Severity::Suggestion, 1), Severity::Warning);
    assert_eq!(escalate(Severity::Warning, 1), Severity::Error);
    assert_eq!(escalate(Severity::Info, 1), Severity::Suggestion);
}

#[test]
fn test_escalate_two_levels() {
    use skills_validator::models::escalate;
    assert_eq!(escalate(Severity::Suggestion, 2), Severity::Error);
    assert_eq!(escalate(Severity::Info, 2), Severity::Warning);
}

#[test]
fn test_escalate_caps_at_error() {
    use skills_validator::models::escalate;
    assert_eq!(escalate(Severity::Error, 1), Severity::Error);
    assert_eq!(escalate(Severity::Error, 2), Severity::Error);
    assert_eq!(escalate(Severity::Warning, 2), Severity::Error);
}

// === PipelineError display tests ===

#[test]
fn test_pipeline_error_parse_failed_display() {
    let err = PipelineError::ParseFailed {
        path: PathBuf::from("skill.md"),
        reason: "invalid YAML".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("skill.md"), "should contain path");
    assert!(msg.contains("invalid YAML"), "should contain reason");
}

#[test]
fn test_pipeline_error_io_error_display() {
    let err = PipelineError::IoError {
        path: PathBuf::from("/tmp/missing.md"),
        reason: "file not found".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("/tmp/missing.md"));
    assert!(msg.contains("file not found"));
}

#[test]
fn test_pipeline_error_semgrep_failed_display() {
    let err = PipelineError::SemgrepFailed {
        reason: "timeout".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("timeout"));
}

#[test]
fn test_pipeline_error_config_invalid_display() {
    let err = PipelineError::ConfigInvalid {
        reason: "bad toml".to_string(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("bad toml"));
}

// === CheckName serialization tests ===

#[test]
fn test_check_name_serialization_kebab_case() {
    let serialized = serde_json::to_string(&CheckName::SkillFileExists).unwrap();
    assert_eq!(serialized, "\"skill-file-exists\"");

    let serialized = serde_json::to_string(&CheckName::FrontmatterValidYaml).unwrap();
    assert_eq!(serialized, "\"frontmatter-valid-yaml\"");

    let serialized = serde_json::to_string(&CheckName::NameDirectoryMatch).unwrap();
    assert_eq!(serialized, "\"name-directory-match\"");
}

#[test]
fn test_check_name_deserialization() {
    let name: CheckName = serde_json::from_str("\"broken-reference\"").unwrap();
    assert_eq!(name, CheckName::BrokenReference);
}

// === FileEntry / FileType tests ===

#[test]
fn test_file_entry_construction() {
    let entry = FileEntry {
        path: PathBuf::from("scripts/setup.sh"),
        file_type: FileType::Script,
    };
    assert_eq!(entry.file_type, FileType::Script);
}

// === SkillContext tests ===

#[test]
fn test_skill_context_default() {
    let ctx = SkillContext::default();
    assert_eq!(ctx.sizeyness, Sizeyness::Simple);
    assert!(ctx.file_inventory.is_empty());
    assert!(ctx.headings.is_empty());
    assert!(ctx.links.is_empty());
    assert!(ctx.code_blocks.is_empty());
    assert!(ctx.prose_text.is_empty());
    assert!(ctx.subdirectories.is_empty());
    assert!(ctx.referenced_files.is_empty());
}

// === Helper struct tests ===

#[test]
fn test_heading_construction() {
    let h = Heading {
        level: 2,
        text: "Examples".to_string(),
    };
    assert_eq!(h.level, 2);
    assert_eq!(h.text, "Examples");
}

#[test]
fn test_link_construction() {
    let l = Link {
        text: "see this".to_string(),
        url: "./other.md".to_string(),
    };
    assert_eq!(l.url, "./other.md");
}

#[test]
fn test_code_block_construction() {
    let cb = CodeBlock {
        language: Some("rust".to_string()),
        content: "fn main() {}".to_string(),
    };
    assert_eq!(cb.language, Some("rust".to_string()));
}

// === Existing tests below ===

#[test]
fn test_skill_properties_to_dict() {
    let props = skills_validator::models::SkillProperties {
        name: "test".to_string(),
        description: "Test description".to_string(),
        license: Some("MIT".to_string()),
        compatibility: None,
        allowed_tools: None,
        metadata: std::collections::HashMap::new(),
    };
    let dict = props.to_dict();
    assert!(dict.is_mapping());
}

#[test]
fn test_skill_properties_to_dict_with_metadata() {
    use std::collections::HashMap;
    let mut metadata = HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());
    let props = skills_validator::models::SkillProperties {
        name: "test".to_string(),
        description: "Test description".to_string(),
        license: None,
        compatibility: None,
        allowed_tools: None,
        metadata,
    };
    let dict = props.to_dict();
    let map = dict.as_mapping().unwrap();
    assert!(map.contains_key(&serde_yaml::Value::String("metadata".to_string())));
}
