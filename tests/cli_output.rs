use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

fn make_skill(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("SKILL.md"), content).unwrap();
    path
}

fn run_skill_validator(args: &[&str]) -> Output {
    Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .current_dir(".")
        .output()
        .expect("Failed to run skills-validator")
}

mod validate_json_output {
    use super::*;

    #[test]
    fn test_validate_json_outputs_valid_json() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should have at least one valid JSON line (the result)
        let json_lines: Vec<&str> = stderr
            .lines()
            .filter(|line| line.starts_with('{') && line.ends_with('}'))
            .collect();

        assert!(
            !json_lines.is_empty(),
            "No JSON output found in stderr: {}",
            stderr
        );

        // Try to parse the last JSON line as an object
        let last_json = json_lines.last().unwrap();
        assert!(
            serde_json::from_str::<serde_json::Value>(last_json).is_ok(),
            "Invalid JSON: {}",
            last_json
        );
    }

    #[test]
    fn test_validate_json_includes_valid_field() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Find the result JSON line
        let json_lines: Vec<&str> = stderr
            .lines()
            .filter(|line| line.contains("\"valid\""))
            .collect();

        assert!(
            !json_lines.is_empty(),
            "No result JSON found in stderr: {}",
            stderr
        );

        let json_str = json_lines.last().unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert!(value.get("valid").is_some(), "Missing 'valid' field");
        assert!(
            value.get("valid").unwrap().as_bool().is_some(),
            "'valid' should be a boolean"
        );
    }

    #[test]
    fn test_validate_json_includes_errors() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "test", "---\ndescription: Missing name\n---\ncontent");
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should have non-zero exit code
        assert_ne!(output.status.code(), Some(0), "Should exit with error");

        // Find the result JSON line with errors
        let json_lines: Vec<&str> = stderr
            .lines()
            .filter(|line| line.contains("\"errors\""))
            .collect();

        assert!(!json_lines.is_empty(), "No result JSON with errors found");

        let json_str = json_lines.last().unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert!(value.get("errors").is_some(), "Missing 'errors' field");
        let errors = value.get("errors").unwrap().as_array().unwrap();
        assert!(!errors.is_empty(), "Errors should not be empty");
    }

    #[test]
    fn test_validate_json_includes_warnings() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content without keywords",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should exit successfully but have warnings
        assert_eq!(output.status.code(), Some(0), "Should exit successfully");

        // Find the result JSON line with warnings
        let json_lines: Vec<&str> = stderr
            .lines()
            .filter(|line| line.contains("\"warnings\""))
            .collect();

        assert!(!json_lines.is_empty(), "No result JSON with warnings found");

        let json_str = json_lines.last().unwrap();
        let value: serde_json::Value = serde_json::from_str(json_str).unwrap();

        assert!(value.get("warnings").is_some(), "Missing 'warnings' field");
        let warnings = value.get("warnings").unwrap().as_array().unwrap();
        assert!(!warnings.is_empty(), "Warnings should not be empty");
    }

    #[test]
    fn test_validate_json_stderr_only() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\ncontent",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let _stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // stdout should be empty (result goes to stderr when --json)
        // The result should contain JSON in stderr
        let has_json_stderr = stderr
            .lines()
            .any(|line| line.starts_with('{') && line.ends_with('}'));

        assert!(
            has_json_stderr,
            "JSON should be in stderr when --json is used"
        );
    }
}

mod validate_text_output {
    use super::*;

    #[test]
    fn test_validate_text_outputs_plain_text() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should have the success message
        assert!(
            stdout.contains("✓ Skill is valid"),
            "Should contain '✓ Skill is valid' in stdout: {}",
            stdout
        );
    }

    #[test]
    fn test_validate_text_with_warnings() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should have the success message with warnings
        assert!(
            stdout.contains("✓ Skill is valid (with warnings)"),
            "Should contain '✓ Skill is valid (with warnings)' in stdout: {}",
            stdout
        );
    }

    #[test]
    fn test_validate_text_error_to_stderr() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "test", "---\ndescription: Missing name\n---\ncontent");
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Error messages should go to stderr
        assert!(stderr.contains("name"), "Error should mention 'name' field");
        assert_ne!(output.status.code(), Some(0), "Should exit with error");
    }
}
