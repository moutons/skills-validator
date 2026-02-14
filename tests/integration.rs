use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn make_skill(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::create_dir_all(&path).unwrap();
    fs::write(path.join("SKILL.md"), content).unwrap();
    path
}

mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_basic() {
        let content = "---\nname: test-skill\ndescription: A test skill\n---\n# Body";
        let (metadata, body) = skills_validator::parser::parse_frontmatter(content).unwrap();
        let map = metadata.as_mapping().unwrap();
        assert_eq!(
            map.get(&serde_yaml::Value::String("name".to_string()))
                .unwrap()
                .as_str(),
            Some("test-skill")
        );
        assert_eq!(
            map.get(&serde_yaml::Value::String("description".to_string()))
                .unwrap()
                .as_str(),
            Some("A test skill")
        );
        assert_eq!(body, "# Body");
    }

    #[test]
    fn test_parse_frontmatter_missing_frontmatter() {
        let result = skills_validator::parser::parse_frontmatter("No frontmatter here");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_frontmatter_unclosed() {
        let result =
            skills_validator::parser::parse_frontmatter("---name: test\ndescription: test");
        assert!(result.is_err());
    }

    #[test]
    fn test_find_skill_md() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "test-skill", "content");
        let found = skills_validator::parser::find_skill_md(&path);
        assert!(found.is_some());
    }

    #[test]
    fn test_find_skill_md_not_found() {
        let dir = TempDir::new().unwrap();
        let found = skills_validator::parser::find_skill_md(dir.path());
        assert!(found.is_none());
    }

    #[test]
    fn test_read_properties() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-prop",
            "---\nname: test-prop\ndescription: Testing properties\nlicense: MIT\n---\nbody",
        );
        let props = skills_validator::parser::read_properties(&path).unwrap();
        assert_eq!(props.name, "test-prop");
        assert_eq!(props.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_read_properties_with_metadata() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(&dir, "meta-skill", "---\nname: meta-skill\ndescription: Skill with metadata\nmetadata:\n  author: testuser\n---\nbody");
        let props = skills_validator::parser::read_properties(&path).unwrap();
        assert_eq!(props.metadata.get("author"), Some(&"testuser".to_string()));
    }
}

mod validator_tests {
    use super::*;

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
    fn test_validate_keyword_found() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "test-skill",
            "---\nname: test-skill\ndescription: Test skill\n---\nnever do this in your responses",
        );
        let result = skills_validator::validator::validate(&path);
        assert!(result
            .warnings
            .iter()
            .any(|w| w.contains("Good: Found 'never'")));
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
            .any(|w| w.contains("Warning: 'never' not found")));
    }

    #[test]
    fn test_validate_missing_skill_md() {
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
        let file = TempDir::new().unwrap();
        let file_path = file.path().join("file.txt");
        fs::write(&file_path, "content").unwrap();
        let result = skills_validator::validator::validate(&file_path);
        assert!(!result.errors.is_empty());
        assert!(result.errors.iter().any(|e| e.contains("Not a directory")));
    }
}

mod prompt_tests {
    use super::*;

    #[test]
    fn test_to_prompt_empty() {
        let result = skills_validator::prompt::to_prompt(&[]);
        assert!(result.contains("<available_skills>"));
        assert!(result.contains("</available_skills>"));
    }

    #[test]
    fn test_to_prompt_single_skill() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "prompt-test",
            "---\nname: prompt-test\ndescription: Testing prompt\n---\ncontent",
        );
        let result = skills_validator::prompt::to_prompt(&[path.to_str().unwrap()]);
        assert!(result.contains("<name>"));
        assert!(result.contains("prompt-test"));
        assert!(result.contains("<description>"));
        assert!(result.contains("<location>"));
    }

    #[test]
    fn test_to_prompt_multiple_skills() {
        let dir1 = TempDir::new().unwrap();
        let path1 = make_skill(
            &dir1,
            "skill-one",
            "---\nname: skill-one\ndescription: First\n---\ncontent",
        );
        let dir2 = TempDir::new().unwrap();
        let path2 = make_skill(
            &dir2,
            "skill-two",
            "---\nname: skill-two\ndescription: Second\n---\ncontent",
        );
        let result = skills_validator::prompt::to_prompt(&[
            path1.to_str().unwrap(),
            path2.to_str().unwrap(),
        ]);
        assert!(result.contains("skill-one"));
        assert!(result.contains("skill-two"));
        assert!(result.matches("<skill>").count() == 2);
    }

    #[test]
    fn test_to_prompt_html_escaping() {
        let dir = TempDir::new().unwrap();
        let path = make_skill(
            &dir,
            "escape-test",
            "---\nname: escape-test\ndescription: Test <script>\n---\ncontent",
        );
        let result = skills_validator::prompt::to_prompt(&[path.to_str().unwrap()]);
        assert!(result.contains("&lt;script&gt;"));
        assert!(!result.contains("<script>"));
    }
}

mod models_tests {
    use super::*;

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
}
