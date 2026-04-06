// Integration tests for the formatter module.

use std::path::PathBuf;

use skills_validator::config::ValidatorConfig;
use skills_validator::formatter::{format_human, format_json};
use skills_validator::models::Severity;
use skills_validator::pipeline::run_pipeline;

fn fixture(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}

// ---------------------------------------------------------------------------
// Human output on real pipeline results
// ---------------------------------------------------------------------------

#[test]
fn human_output_on_valid_skill_has_header_and_summary() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let out = format_human(&result, &dir, Severity::Info);
    // Should have a header with the folder icon
    assert!(out.contains('\u{1f4c1}'), "Missing folder emoji in header");
    // Should have a summary line
    assert!(out.contains("Summary:"), "Missing summary line");
}

#[test]
fn human_output_severity_filter_reduces_output() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let all = format_human(&result, &dir, Severity::Info);
    let errors_only = format_human(&result, &dir, Severity::Error);

    // Errors-only output should be shorter (or equal if no non-error diagnostics)
    assert!(
        errors_only.len() <= all.len(),
        "Filtered output should not be longer than unfiltered"
    );
}

// ---------------------------------------------------------------------------
// JSON output on real pipeline results
// ---------------------------------------------------------------------------

#[test]
fn json_output_on_valid_skill_is_parseable() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let json_str = format_json(&result, &dir, Severity::Info, false);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    assert_eq!(v["schema_version"], 2);
    assert!(v["diagnostics"].is_array());
    assert!(v["summary"].is_object());
    assert!(v["sizeyness_reasons"].is_array());
}

#[test]
fn json_output_exit_code_zero_for_valid_skill() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let json_str = format_json(&result, &dir, Severity::Info, false);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(v["exit_code"], 0);
}

#[test]
fn json_output_exit_code_one_for_invalid_skill() {
    let dir = fixture("does-not-exist");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let json_str = format_json(&result, &dir, Severity::Info, false);
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(v["exit_code"], 1);
}

#[test]
fn json_severity_filter_on_real_pipeline() {
    let dir = fixture("valid/minimal/coding-standards");
    let config = ValidatorConfig::default();
    let result = run_pipeline(&dir, &config);

    let all_json = format_json(&result, &dir, Severity::Info, false);
    let errors_json = format_json(&result, &dir, Severity::Error, false);

    let all: serde_json::Value = serde_json::from_str(&all_json).unwrap();
    let errors: serde_json::Value = serde_json::from_str(&errors_json).unwrap();

    let all_count = all["diagnostics"].as_array().unwrap().len();
    let error_count = errors["diagnostics"].as_array().unwrap().len();

    assert!(
        error_count <= all_count,
        "Filtered diagnostics should not exceed unfiltered"
    );
}
