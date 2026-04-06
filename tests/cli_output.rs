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
    fn test_output_format_json_outputs_valid_json_to_stdout() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&[
            "--output-format",
            "json",
            "validate",
            path.to_str().unwrap(),
        ]);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // New pipeline JSON goes to stdout
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        assert!(parsed.is_ok(), "stdout should be valid JSON: {}", stdout);

        let v = parsed.unwrap();
        assert_eq!(v["schema_version"], 2, "Should have schema_version 2");
        assert!(v.get("diagnostics").is_some(), "Should have diagnostics");
        assert!(v.get("summary").is_some(), "Should have summary");
    }

    #[test]
    fn test_deprecated_json_flag_emits_warning() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should have deprecation warning
        assert!(
            stderr.contains("--json is deprecated"),
            "Should contain deprecation warning in stderr: {}",
            stderr
        );
    }

    #[test]
    fn test_deprecated_json_flag_outputs_json_to_stdout() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["--json", "validate", path.to_str().unwrap()]);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // --json now outputs new pipeline JSON to stdout
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
        assert!(
            parsed.is_ok(),
            "stdout should be valid JSON when --json is used: {}",
            stdout
        );
    }

    #[test]
    fn test_validate_json_exit_code_on_errors() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "test", "---\ndescription: Missing name\n---\ncontent");
        let output = run_skill_validator(&[
            "--output-format",
            "json",
            "validate",
            path.to_str().unwrap(),
        ]);

        // Should have non-zero exit code
        assert_ne!(output.status.code(), Some(0), "Should exit with error");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(v["exit_code"], 1, "JSON exit_code should be 1");
        assert!(
            v["summary"]["errors"].as_u64().unwrap() > 0,
            "Should have errors in summary"
        );
    }

    #[test]
    fn test_validate_json_includes_diagnostics() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content without keywords",
        );
        let output = run_skill_validator(&[
            "--output-format",
            "json",
            "validate",
            path.to_str().unwrap(),
        ]);

        assert_eq!(output.status.code(), Some(0), "Should exit successfully");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        let diags = v["diagnostics"].as_array().unwrap();
        assert!(!diags.is_empty(), "Should have diagnostics");
    }
}

mod validate_text_output {
    use super::*;

    #[test]
    fn test_validate_text_outputs_to_stdout() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        let stdout = String::from_utf8_lossy(&output.stdout);

        // New pipeline outputs human format to stdout with the skill name header
        assert!(
            stdout.contains("test-skill"),
            "Should contain skill name in stdout: {}",
            stdout
        );
    }

    #[test]
    fn test_validate_text_error_exit_code() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "test", "---\ndescription: Missing name\n---\ncontent");
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        assert_ne!(output.status.code(), Some(0), "Should exit with error");
    }
}

mod strict_mode {
    use super::*;

    #[test]
    fn test_strict_exits_nonzero_on_warnings() {
        let dir = TempDir::new().unwrap();
        // A skill that is valid but has suggestions/warnings
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["--strict", "validate", path.to_str().unwrap()]);

        // Without strict this would be exit 0, with strict it should be 1
        // because there will be suggestions/warnings
        assert_ne!(
            output.status.code(),
            Some(0),
            "Strict mode should fail on warnings/suggestions"
        );
    }

    #[test]
    fn test_non_strict_exits_zero_on_warnings() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&["validate", path.to_str().unwrap()]);

        assert_eq!(
            output.status.code(),
            Some(0),
            "Non-strict should exit 0 when only warnings/suggestions"
        );
    }
}

mod severity_filter {
    use super::*;

    #[test]
    fn test_severity_filter_in_json() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nsome content",
        );
        let output = run_skill_validator(&[
            "--output-format",
            "json",
            "--severity",
            "error",
            "validate",
            path.to_str().unwrap(),
        ]);

        let stdout = String::from_utf8_lossy(&output.stdout);
        let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();

        let diags = v["diagnostics"].as_array().unwrap();
        for d in diags {
            assert_eq!(
                d["severity"].as_str().unwrap(),
                "error",
                "All diagnostics should be error severity when --severity error"
            );
        }
    }
}
