use std::path::Path;

mod helpers;

#[test]
fn test_validate_fixture_valid_skill() {
    let path = Path::new("tests/fixtures/valid-skill");
    let result = skills_validator::validator::validate(path);
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
}

#[test]
fn test_validate_fixture_invalid_name() {
    let path = Path::new("tests/fixtures/invalid-name");
    let result = skills_validator::validator::validate(path);
    assert!(!result.errors.is_empty(), "Errors: {:?}", result.errors);
    let has_error = result
        .errors
        .iter()
        .any(|e| e.contains("lowercase") || e.contains("name"));
    assert!(has_error, "Errors: {:?}", result.errors);
}

#[test]
fn test_validate_fixture_missing_description() {
    let path = Path::new("tests/fixtures/missing-description");
    let result = skills_validator::validator::validate(path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("description")));
}
