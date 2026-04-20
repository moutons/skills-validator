//! Integration tests for Pass 3 (Content).

use std::path::PathBuf;

use skills_validator::config::ValidatorConfig;
use skills_validator::models::{
    CheckName, CodeBlock, Heading, Link, Severity, Sizeyness, SkillContext,
};
use skills_validator::passes::content;

// ─── Helpers ───────────────────────────────────────────────────────────────────

fn make_frontmatter(fields: &[(&str, &str)]) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    for (k, v) in fields {
        map.insert(
            serde_yaml::Value::String(k.to_string()),
            serde_yaml::Value::String(v.to_string()),
        );
    }
    serde_yaml::Value::Mapping(map)
}

fn make_ctx(frontmatter: serde_yaml::Value) -> SkillContext {
    SkillContext {
        frontmatter,
        ..Default::default()
    }
}

fn skill_dir(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}

fn default_config() -> ValidatorConfig {
    ValidatorConfig::default()
}

fn find_diag(
    diags: &[skills_validator::models::Diagnostic],
    check: CheckName,
) -> Option<&skills_validator::models::Diagnostic> {
    diags.iter().find(|d| d.check_name == check)
}

fn has_check(diags: &[skills_validator::models::Diagnostic], check: CheckName) -> bool {
    find_diag(diags, check).is_some()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Frontmatter — name checks
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn name_missing_emits_error() {
    let fm = make_frontmatter(&[("description", "A short desc")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("test-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::NameMissing).expect("expected name-missing");
    assert_eq!(d.severity, Severity::Error);
}

#[test]
fn name_empty_string_is_missing() {
    let fm = make_frontmatter(&[("name", ""), ("description", "A desc")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("test-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::NameMissing));
}

#[test]
fn name_format_uppercase_is_error() {
    let fm = make_frontmatter(&[("name", "MySkill"), ("description", "A desc")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("MySkill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::NameFormat).expect("expected name-format");
    assert_eq!(d.severity, Severity::Error);
    assert!(d.human_message.contains("lowercase"));
}

#[test]
fn name_format_too_long() {
    let long_name = "a".repeat(65);
    let fm = make_frontmatter(&[("name", &long_name), ("description", "A desc")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir(&long_name), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::NameFormat));
}

#[test]
fn name_format_valid_passes() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::NameMissing));
    assert!(!has_check(&diags, CheckName::NameFormat));
    assert!(!has_check(&diags, CheckName::NameDirectoryMatch));
}

#[test]
fn name_directory_mismatch() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "A desc")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("other-skill"), &ctx, &default_config()).unwrap();
    let d =
        find_diag(&diags, CheckName::NameDirectoryMatch).expect("expected name-directory-match");
    assert_eq!(d.severity, Severity::Error);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Frontmatter — description checks
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn description_missing_emits_error() {
    let fm = make_frontmatter(&[("name", "my-skill")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::DescriptionMissing).expect("expected description-missing");
    assert_eq!(d.severity, Severity::Error);
}

#[test]
fn description_over_250_chars_is_error() {
    let long_desc = "x".repeat(251);
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", &long_desc)]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::DescriptionLength).expect("expected description-length");
    assert_eq!(d.severity, Severity::Error);
    assert!(d.doc_url.is_some());
    assert!(d.human_message.contains("truncates"));
}

#[test]
fn description_exactly_250_chars_passes() {
    let desc = "x".repeat(250);
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", &desc)]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::DescriptionLength));
}

#[test]
fn description_trigger_language_present_no_diagnostic() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when deploying containers"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::DescriptionTriggerLanguage));
}

#[test]
fn description_trigger_language_missing_escalates_by_sizeyness() {
    for (sizeyness, expected_severity) in [
        (Sizeyness::Simple, Severity::Suggestion),
        (Sizeyness::Moderate, Severity::Warning),
        (Sizeyness::Hefty, Severity::Error),
    ] {
        let fm = make_frontmatter(&[
            ("name", "my-skill"),
            ("description", "A regular description without triggers"),
        ]);
        let mut ctx = make_ctx(fm);
        ctx.sizeyness = sizeyness;
        let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
        let d = find_diag(&diags, CheckName::DescriptionTriggerLanguage)
            .unwrap_or_else(|| panic!("expected diagnostic for {:?}", sizeyness));
        assert_eq!(
            d.severity, expected_severity,
            "wrong severity for {:?}",
            sizeyness
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Frontmatter — field checks
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn unknown_field_emits_warning() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("bogus-field", "value"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::UnknownField).expect("expected unknown-field");
    assert_eq!(d.severity, Severity::Warning);
    assert!(d.human_message.contains("bogus-field"));
}

#[test]
fn extension_field_emits_compatibility_suggestion() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("model", "claude-opus-4-6"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::ExtensionFieldCompatibility)
        .expect("expected extension-field-compatibility");
    assert_eq!(d.severity, Severity::Suggestion);
    assert!(d.human_message.contains("model"));
}

#[test]
fn spec_fields_do_not_trigger_unknown() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("license", "MIT"),
        ("compatibility", "claude-code"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::UnknownField));
    assert!(!has_check(&diags, CheckName::ExtensionFieldCompatibility));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Extension semantics
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn context_must_be_fork() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("context", "spawn"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::ContextValidValue).expect("expected context-valid-value");
    assert_eq!(d.severity, Severity::Error);
}

#[test]
fn context_fork_is_valid() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("context", "fork"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::ContextValidValue));
}

#[test]
fn agent_without_context_emits_warning() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("agent", "true"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::AgentWithContext).expect("expected agent-with-context");
    assert_eq!(d.severity, Severity::Warning);
}

#[test]
fn agent_with_context_fork_no_warning() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("agent", "true"),
        ("context", "fork"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::AgentWithContext));
}

#[test]
fn model_recognized_known_model_no_diagnostic() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("model", "claude-opus-4-6"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::ModelRecognized));
}

#[test]
fn model_unrecognized_emits_suggestion() {
    let fm = make_frontmatter(&[
        ("name", "my-skill"),
        ("description", "Use when testing"),
        ("model", "gpt-4-turbo"),
    ]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::ModelRecognized).expect("expected model-recognized");
    assert_eq!(d.severity, Severity::Suggestion);
    assert!(d.human_message.contains("gpt-4-turbo"));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — trigger conditions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn trigger_conditions_in_prose() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Use when the user asks about deployment.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasTriggerConditions));
}

#[test]
fn trigger_conditions_in_heading() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.headings = vec![Heading {
        level: 2,
        text: "When to Use This Skill".to_string(),
    }];
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasTriggerConditions));
}

#[test]
fn trigger_conditions_missing_escalates() {
    for (sizeyness, expected) in [
        (Sizeyness::Simple, Severity::Suggestion),
        (Sizeyness::Moderate, Severity::Warning),
        (Sizeyness::Hefty, Severity::Error),
    ] {
        let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
        let mut ctx = make_ctx(fm);
        ctx.sizeyness = sizeyness;
        ctx.prose_text = "Just some text without trigger language.".to_string();
        let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
        let d = find_diag(&diags, CheckName::HasTriggerConditions)
            .unwrap_or_else(|| panic!("expected for {:?}", sizeyness));
        assert_eq!(d.severity, expected, "wrong for {:?}", sizeyness);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — examples
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn examples_with_code_blocks_pass() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.code_blocks = vec![CodeBlock {
        language: Some("rust".to_string()),
        content: "fn main() {}".to_string(),
    }];
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasExamples));
}

#[test]
fn examples_missing_caps_at_warning() {
    // Hefty should still be Warning, not Error
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.sizeyness = Sizeyness::Hefty;
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::HasExamples).expect("expected has-examples");
    assert_eq!(d.severity, Severity::Warning); // caps at warning, not error
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — behavioral constraints (word boundary matching)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn behavioral_constraints_never_matches_word_boundary() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Never commit secrets. Always validate input.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasBehavioralConstraints));
}

#[test]
fn behavioral_constraints_whenever_does_not_match_never() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    // "whenever" should NOT satisfy the \bnever\b check
    ctx.prose_text = "Do this whenever you want.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(
        has_check(&diags, CheckName::HasBehavioralConstraints),
        "whenever should not match \\bnever\\b"
    );
}

#[test]
fn behavioral_constraints_missing_escalates() {
    for (sizeyness, expected) in [
        (Sizeyness::Simple, Severity::Suggestion),
        (Sizeyness::Moderate, Severity::Warning),
        (Sizeyness::Hefty, Severity::Warning), // caps at warning
    ] {
        let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
        let mut ctx = make_ctx(fm);
        ctx.sizeyness = sizeyness;
        ctx.prose_text = "Some text without constraint words.".to_string();
        let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
        let d = find_diag(&diags, CheckName::HasBehavioralConstraints)
            .unwrap_or_else(|| panic!("expected for {:?}", sizeyness));
        assert_eq!(d.severity, expected, "wrong for {:?}", sizeyness);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — gotchas
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn gotchas_heading_present_no_diagnostic() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.headings = vec![Heading {
        level: 2,
        text: "Common Gotchas".to_string(),
    }];
    ctx.prose_text = "Never do this.\n- gotcha item one\n- gotcha item two".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasGotchas));
}

#[test]
fn gotchas_missing_escalation_simple_moderate_suggestion_hefty_warning() {
    for (sizeyness, expected) in [
        (Sizeyness::Simple, Severity::Suggestion),
        (Sizeyness::Moderate, Severity::Suggestion),
        (Sizeyness::Hefty, Severity::Warning),
    ] {
        let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
        let mut ctx = make_ctx(fm);
        ctx.sizeyness = sizeyness;
        ctx.prose_text = "Never do wrong things. Always do right.".to_string();
        let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
        let d = find_diag(&diags, CheckName::HasGotchas)
            .unwrap_or_else(|| panic!("expected for {:?}", sizeyness));
        assert_eq!(d.severity, expected, "wrong for {:?}", sizeyness);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — body length
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn body_length_over_limit_escalates() {
    let long_prose = (0..301)
        .map(|i| format!("Line {i}"))
        .collect::<Vec<_>>()
        .join("\n");

    for (sizeyness, expected) in [
        (Sizeyness::Simple, Severity::Suggestion),
        (Sizeyness::Moderate, Severity::Warning),
        (Sizeyness::Hefty, Severity::Error),
    ] {
        let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
        let mut ctx = make_ctx(fm);
        ctx.sizeyness = sizeyness;
        ctx.prose_text = long_prose.clone();
        let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
        let d = find_diag(&diags, CheckName::BodyLength)
            .unwrap_or_else(|| panic!("expected body-length for {:?}", sizeyness));
        assert_eq!(d.severity, expected, "wrong for {:?}", sizeyness);
    }
}

#[test]
fn body_length_under_limit_no_diagnostic() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Short body.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::BodyLength));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Content quality — windows paths
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn windows_paths_detected() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = r"Place the file at C:\Users\admin\project\config.txt".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::WindowsPaths));
}

#[test]
fn posix_paths_no_windows_diagnostic() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Place the file at /home/user/project/config.txt".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::WindowsPaths));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Positive reinforcement
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn gotchas_section_with_content_emits_info() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.headings = vec![
        Heading {
            level: 2,
            text: "Gotchas".to_string(),
        },
        Heading {
            level: 3,
            text: "Specific Issue".to_string(),
        },
    ];
    ctx.prose_text = "Never ignore warnings.\n- Watch out for this\n- And this".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    let d = find_diag(&diags, CheckName::HasGotchasSection).expect("expected has-gotchas-section");
    assert_eq!(d.severity, Severity::Info);
}

#[test]
fn validation_loop_checklist_emits_info() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Never skip these steps:\n- [ ] Check output\n- [x] Run tests".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::HasValidationLoop));
}

#[test]
fn validation_loop_validate_run_emits_info() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.prose_text = "Always validate the output, then run the test suite.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::HasValidationLoop));
}

#[test]
fn progressive_disclosure_with_subdir_link() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.subdirectories = vec![PathBuf::from("agents")];
    ctx.links = vec![Link {
        text: "Agent config".to_string(),
        url: "agents/main.md".to_string(),
    }];
    ctx.prose_text = "Never skip the agent config.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::HasProgressiveDisclosure));
}

#[test]
fn progressive_disclosure_no_subdirs_no_info() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let ctx = make_ctx(fm);
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasProgressiveDisclosure));
}

#[test]
fn concrete_examples_with_heading_and_code() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.headings = vec![Heading {
        level: 2,
        text: "Examples".to_string(),
    }];
    ctx.code_blocks = vec![CodeBlock {
        language: Some("bash".to_string()),
        content: "echo hello".to_string(),
    }];
    ctx.prose_text = "Never forget to test.\n".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(has_check(&diags, CheckName::HasConcreteExamples));
}

#[test]
fn concrete_examples_without_example_heading_no_info() {
    let fm = make_frontmatter(&[("name", "my-skill"), ("description", "Use when testing")]);
    let mut ctx = make_ctx(fm);
    ctx.code_blocks = vec![CodeBlock {
        language: Some("bash".to_string()),
        content: "echo hello".to_string(),
    }];
    ctx.prose_text = "Never forget.".to_string();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    assert!(!has_check(&diags, CheckName::HasConcreteExamples));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Null frontmatter (no mapping) — should not panic
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn null_frontmatter_does_not_panic() {
    let ctx = SkillContext::default();
    let diags = content::run(&skill_dir("my-skill"), &ctx, &default_config()).unwrap();
    // Should still get content quality diagnostics but no frontmatter ones
    assert!(!has_check(&diags, CheckName::NameMissing));
}
