use tempfile::TempDir;

mod helpers;
use helpers::make_skill;

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
    let result = skills_validator::parser::parse_frontmatter("---name: test\ndescription: test");
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
    let path = make_skill(
        &dir,
        "meta-skill",
        "---\nname: meta-skill\ndescription: Skill with metadata\nmetadata:\n  author: testuser\n---\nbody",
    );
    let props = skills_validator::parser::read_properties(&path).unwrap();
    assert_eq!(props.metadata.get("author"), Some(&"testuser".to_string()));
}
