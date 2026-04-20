use std::path::Path;

use skills_validator::config::ValidatorConfig;
use skills_validator::{run_pipeline, Severity};

mod helpers;

#[test]
fn test_validate_fixture_valid_skill() {
    let path = Path::new("tests/fixtures/valid-skill");
    let config = ValidatorConfig::default();
    let result = run_pipeline(path, &config);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "Errors: {:?}", errors);
}

#[test]
fn test_validate_fixture_invalid_name() {
    let path = Path::new("tests/fixtures/invalid-name");
    let config = ValidatorConfig::default();
    let result = run_pipeline(path, &config);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(!errors.is_empty(), "Errors: {:?}", errors);
    let has_name_error = errors
        .iter()
        .any(|e| e.human_message.contains("lowercase") || e.human_message.contains("name"));
    assert!(has_name_error, "Errors: {:?}", errors);
}

#[test]
fn test_validate_fixture_missing_description() {
    let path = Path::new("tests/fixtures/missing-description");
    let config = ValidatorConfig::default();
    let result = run_pipeline(path, &config);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(!errors.is_empty());
    assert!(errors
        .iter()
        .any(|e| e.human_message.contains("description")));
}
