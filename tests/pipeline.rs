// Integration tests for the pipeline orchestrator.

use std::path::PathBuf;

use skills_validator::config::ValidatorConfig;
use skills_validator::models::{CheckName, Severity, Sizeyness};
use skills_validator::pipeline::{exit_code, run_pipeline};

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}

// ---------------------------------------------------------------------------
// Full pipeline on valid simple skill
// ---------------------------------------------------------------------------

#[test]
fn valid_minimal_skill_produces_no_errors() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    // No errors — only Info/Suggestion expected.
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        errors.is_empty(),
        "Expected no errors for valid skill, got: {errors:?}"
    );

    assert_eq!(result.sizeyness, Sizeyness::Simple);
    assert_eq!(exit_code(&result.diagnostics, false), 0);
}

#[test]
fn valid_minimal_skill_has_skill_name() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    // The skill should have a name extracted from frontmatter.
    assert!(
        result.skill_name.is_some(),
        "Expected skill_name to be set for valid skill"
    );
}

// ---------------------------------------------------------------------------
// Invalid skill — parse failures
// ---------------------------------------------------------------------------

#[test]
fn nonexistent_dir_returns_pipeline_error() {
    let dir = fixture("does-not-exist");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.check_name == CheckName::PipelineError
                || d.check_name == CheckName::SkillFileExists),
        "Expected a pipeline or parse error for nonexistent dir"
    );
    assert_eq!(exit_code(&result.diagnostics, false), 1);
    assert!(result.skill_name.is_none());
}

#[test]
fn missing_frontmatter_is_fatal_parse_error() {
    // missing-frontmatter has a SKILL.md but no frontmatter block.
    // Parse returns Err, so the pipeline stops immediately with a PipelineError.
    let dir = fixture("invalid/missing-frontmatter");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    // Should have a pipeline error diagnostic (parse failed).
    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.check_name == CheckName::PipelineError),
        "Expected pipeline-error diagnostic for missing frontmatter, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.check_name))
            .collect::<Vec<_>>()
    );

    // Downstream passes should NOT have run — no sizeyness info.
    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.check_name == CheckName::SizeynessInfo),
        "Structure pass should not have run after fatal parse error"
    );

    assert_eq!(exit_code(&result.diagnostics, false), 1);
    assert!(result.skill_name.is_none());
}

// ---------------------------------------------------------------------------
// Multi-file skill — moderate/hefty sizeyness
// ---------------------------------------------------------------------------

#[test]
fn multi_file_skill_classified_above_simple() {
    // multi-file fixtures have scripts/subdirs, so should be Moderate or Hefty.
    let dir = fixture("valid/multi-file/webapp-testing");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    assert!(
        result.sizeyness != Sizeyness::Simple,
        "Expected multi-file skill to be Moderate or Hefty, got {:?}",
        result.sizeyness
    );
    assert!(!result.sizeyness_reasons.is_empty());
}

// ---------------------------------------------------------------------------
// Strict mode exit code
// ---------------------------------------------------------------------------

#[test]
fn strict_mode_fails_on_suggestions() {
    // Use a skill that produces suggestions but no errors.
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let has_suggestions = result
        .diagnostics
        .iter()
        .any(|d| d.severity == Severity::Suggestion);

    if has_suggestions {
        assert_eq!(
            exit_code(&result.diagnostics, true),
            1,
            "Strict mode should fail when suggestions exist"
        );
    }
    // Non-strict should pass.
    assert_eq!(exit_code(&result.diagnostics, false), 0);
}

// ---------------------------------------------------------------------------
// Broken references — pass 4 produces diagnostics
// ---------------------------------------------------------------------------

#[test]
fn broken_ref_detected() {
    let dir = fixture("broken-ref");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.check_name == CheckName::BrokenReference),
        "Expected broken-reference diagnostic, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.check_name))
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Orphaned files — pass 4 produces diagnostics
// ---------------------------------------------------------------------------

#[test]
fn orphaned_files_detected() {
    let dir = fixture("orphaned-files");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.check_name == CheckName::OrphanedFiles),
        "Expected orphaned-files diagnostic, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| format!("{:?}", d.check_name))
            .collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// Sizeyness reasons populated
// ---------------------------------------------------------------------------

#[test]
fn sizeyness_reasons_populated_for_valid_skill() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    // Even simple skills should have file count in reasons.
    assert!(
        result.sizeyness_reasons.iter().any(|r| r.contains("file")),
        "Expected sizeyness reasons to mention files, got: {:?}",
        result.sizeyness_reasons
    );
}
