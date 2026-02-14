use tempfile::TempDir;

mod helpers;
use helpers::make_skill;

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
    let result =
        skills_validator::prompt::to_prompt(&[path1.to_str().unwrap(), path2.to_str().unwrap()]);
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
