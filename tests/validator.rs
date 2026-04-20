#![allow(deprecated)]

use std::path::PathBuf;
use tempfile::TempDir;

mod helpers;
use helpers::make_skill;

#[test]
fn test_validate_valid_skill() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "valid-skill",
        "---\nname: valid-skill\ndescription: A valid skill\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(result.errors.is_empty(), "Errors: {:?}", result.errors);
}

#[test]
fn test_validate_missing_name() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test-skill",
        "---\ndescription: Missing name\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("name")));
}

#[test]
fn test_validate_missing_description() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(&dir, "test-skill", "---\nname: test-skill\n---\ncontent");
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("description")));
}

#[test]
fn test_validate_name_too_long() {
    let long_name = "a".repeat(65);
    let content = format!("---\nname: {}\ndescription: Test\n---\ncontent", long_name);
    let dir = TempDir::new().unwrap();
    let path = make_skill(&dir, "test", &content);
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("exceeds")));
}

#[test]
fn test_validate_name_uppercase() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test",
        "---\nname: Invalid-Name\ndescription: Test\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("lowercase")));
}

#[test]
fn test_validate_name_starts_with_hyphen() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test",
        "---\nname: -invalid\ndescription: Test\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("hyphen")));
}

#[test]
fn test_validate_name_consecutive_hyphens() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test",
        "---\nname: invalid--name\ndescription: Test\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("consecutive")));
}

#[test]
fn test_validate_name_mismatch_directory() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "correct-name",
        "---\nname: wrong-name\ndescription: Test\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("Directory name")));
}

#[test]
fn test_validate_description_too_long() {
    let long_desc = "a".repeat(1025);
    let content = format!(
        "---\nname: test-skill\ndescription: {}\n---\ncontent",
        long_desc
    );
    let dir = TempDir::new().unwrap();
    let path = make_skill(&dir, "test-skill", &content);
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result
        .errors
        .iter()
        .any(|e| e.contains("Description exceeds")));
}

#[test]
fn test_validate_compatibility_too_long() {
    let long_compat = "a".repeat(501);
    let content = format!(
        "---\nname: test-skill\ndescription: Test\ncompatibility: {}\n---\ncontent",
        long_compat
    );
    let dir = TempDir::new().unwrap();
    let path = make_skill(&dir, "test-skill", &content);
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result
        .errors
        .iter()
        .any(|e| e.contains("Compatibility exceeds")));
}

#[test]
fn test_validate_unknown_field() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test-skill",
        "---\nname: test-skill\ndescription: Test\nunknown-field: value\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("Unexpected field")));
}

#[test]
fn test_validate_claude_code_extension_warning() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test-skill",
        "---\nname: test-skill\ndescription: Test\nargument-hint: [arg]\n---\ncontent",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(result.errors.is_empty());
    assert!(!result.warnings.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Claude Code extension")));
}

#[test]
fn test_validate_keyword_missing() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test-skill",
        "---\nname: test-skill\ndescription: Test skill\n---\nsome content without keywords",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("'never' not found")));
}

#[test]
fn test_validate_missing_skill_md() {
    use std::fs;
    let dir = TempDir::new().unwrap();
    let empty_path = dir.path().join("empty");
    fs::create_dir_all(&empty_path).unwrap();
    let result = skills_validator::validator::validate(&empty_path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("SKILL.md")));
}

#[test]
fn test_validate_path_not_exists() {
    let result =
        skills_validator::validator::validate(PathBuf::from("/nonexistent/path").as_path());
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("does not exist")));
}

#[test]
fn test_validate_not_a_directory() {
    use std::fs;
    let file = TempDir::new().unwrap();
    let file_path = file.path().join("file.txt");
    fs::write(&file_path, "content").unwrap();
    let result = skills_validator::validator::validate(&file_path);
    assert!(!result.errors.is_empty());
    assert!(result.errors.iter().any(|e| e.contains("Not a directory")));
}

#[test]
fn test_validate_body_too_long_warning() {
    let long_body = (0..=501)
        .map(|i| format!("Line {}\n", i))
        .collect::<String>();
    let content = format!(
        "---\nname: test-skill\ndescription: Test\n---\n{}",
        long_body
    );
    let dir = TempDir::new().unwrap();
    let path = make_skill(&dir, "test-skill", &content);
    let result = skills_validator::validator::validate(&path);
    assert!(result.errors.is_empty());
    assert!(result.warnings.iter().any(|w| w.contains("502 lines")));
}

#[test]
fn test_validate_windows_path_warning() {
    let dir = TempDir::new().unwrap();
    let path = make_skill(
        &dir,
        "test-skill",
        "---\nname: test-skill\ndescription: Test\n---\nC:\\Users\\test\\file.md",
    );
    let result = skills_validator::validator::validate(&path);
    assert!(result.errors.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Windows-style path")));
}

#[test]
fn test_validate_script_in_root_warning() {
    use std::fs;
    let dir = TempDir::new().unwrap();
    let skill_path = dir.path().join("test-skill");
    fs::create_dir_all(&skill_path).unwrap();
    fs::write(
        skill_path.join("SKILL.md"),
        "---\nname: test-skill\ndescription: Test\n---\ncontent",
    )
    .unwrap();
    fs::write(skill_path.join("script.sh"), "#!/bin/bash\necho hello").unwrap();

    let result = skills_validator::validator::validate(&skill_path);
    assert!(result.errors.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|w| w.contains("Script file") && w.contains("script.sh")));
}
