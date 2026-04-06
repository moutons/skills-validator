//! Integration tests for Pass 1 (Parse).

use std::fs;
use std::path::Path;

use skills_validator::models::{CheckName, PipelineError, Severity};
use skills_validator::passes::parse;

// ─── Helper ─────────────────────────────────────────────────────────────────

fn fixture(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}

// ─── SKILL.md casing enforcement ────────────────────────────────────────────

#[test]
fn rejects_lowercase_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("skill.md"), "---\nname: x\n---\n# Hi\n").unwrap();

    let err = parse::run(dir.path()).unwrap_err();
    match &err {
        PipelineError::ParseFailed { reason, .. } => {
            assert!(
                reason.contains("wrong casing"),
                "expected casing error, got: {reason}"
            );
        }
        other => panic!("expected ParseFailed, got: {other:?}"),
    }
}

#[test]
fn rejects_mixed_case_skill_md() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("Skill.md"), "---\nname: x\n---\n# Hi\n").unwrap();

    let err = parse::run(dir.path()).unwrap_err();
    match &err {
        PipelineError::ParseFailed { reason, .. } => {
            assert!(
                reason.contains("wrong casing"),
                "expected casing error, got: {reason}"
            );
        }
        other => panic!("expected ParseFailed, got: {other:?}"),
    }
}

#[test]
fn accepts_exact_skill_md_casing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("SKILL.md"),
        "---\nname: test\ndescription: A test skill\n---\n# Hello\n",
    )
    .unwrap();

    let (ctx, diags) = parse::run(dir.path()).unwrap();
    assert!(diags.is_empty(), "expected no diagnostics, got: {diags:?}");
    assert_eq!(ctx.headings.len(), 1);
    assert_eq!(ctx.headings[0].text, "Hello");
}

// ─── Missing SKILL.md ───────────────────────────────────────────────────────

#[test]
fn errors_when_skill_md_missing() {
    let dir = tempfile::tempdir().unwrap();
    // empty directory

    let err = parse::run(dir.path()).unwrap_err();
    match &err {
        PipelineError::ParseFailed { reason, .. } => {
            assert!(reason.contains("not found"), "got: {reason}");
        }
        other => panic!("expected ParseFailed, got: {other:?}"),
    }
}

// ─── Frontmatter extraction ────────────────────────────────────────────────

#[test]
fn extracts_frontmatter_as_yaml_value() {
    let (ctx, _) = parse::run(&fixture("valid/minimal/coding-standards")).unwrap();

    let map = ctx
        .frontmatter
        .as_mapping()
        .expect("frontmatter is a mapping");
    let name = map
        .get(&serde_yaml::Value::String("name".to_string()))
        .and_then(|v| v.as_str());
    assert_eq!(name, Some("coding-standards"));
}

#[test]
fn errors_on_missing_frontmatter() {
    let err = parse::run(&fixture("invalid/missing-frontmatter")).unwrap_err();
    match &err {
        PipelineError::ParseFailed { .. } => {}
        other => panic!("expected ParseFailed, got: {other:?}"),
    }
}

// ─── AST extraction ────────────────────────────────────────────────────────

#[test]
fn extracts_headings_from_complete_skill() {
    let (ctx, _) = parse::run(&fixture("valid/complete/debugger")).unwrap();

    assert!(!ctx.headings.is_empty(), "should find headings");
    assert_eq!(ctx.headings[0].text, "Debugger");
    assert_eq!(ctx.headings[0].level, 1);

    // Should have H2s as well
    let h2s: Vec<_> = ctx.headings.iter().filter(|h| h.level == 2).collect();
    assert!(!h2s.is_empty(), "should have level-2 headings");
}

#[test]
fn extracts_code_blocks_from_complete_skill() {
    let (ctx, _) = parse::run(&fixture("valid/minimal/coding-standards")).unwrap();

    assert!(
        !ctx.code_blocks.is_empty(),
        "coding-standards should have code blocks"
    );
    // The fixture has typescript code blocks
    let ts_blocks: Vec<_> = ctx
        .code_blocks
        .iter()
        .filter(|cb| cb.language.as_deref() == Some("typescript"))
        .collect();
    assert!(!ts_blocks.is_empty(), "should have typescript code blocks");
}

#[test]
fn extracts_links() {
    // debugger fixture has links like [link](url) in its markdown
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("SKILL.md"),
        "---\nname: test\n---\n\nSee [docs](https://example.com/docs) and [api](https://example.com/api).\n",
    )
    .unwrap();

    let (ctx, _) = parse::run(dir.path()).unwrap();
    assert_eq!(ctx.links.len(), 2);
    assert_eq!(ctx.links[0].text, "docs");
    assert_eq!(ctx.links[0].url, "https://example.com/docs");
}

// ─── Prose-only view ────────────────────────────────────────────────────────

#[test]
fn prose_strips_code_blocks_and_urls() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("SKILL.md"),
        r#"---
name: prose-test
---

# Introduction

This is some prose.

```python
print("secret code")
```

More prose with `inline code` removed.

See [reference](https://example.com) for details.
"#,
    )
    .unwrap();

    let (ctx, _) = parse::run(dir.path()).unwrap();

    assert!(ctx.prose_text.contains("This is some prose."));
    assert!(ctx.prose_text.contains("More prose with"));
    assert!(ctx.prose_text.contains("removed."));
    assert!(ctx.prose_text.contains("reference"));
    // Code block content should NOT appear
    assert!(
        !ctx.prose_text.contains("secret code"),
        "prose should not contain code block content"
    );
    // Inline code should NOT appear
    assert!(
        !ctx.prose_text.contains("inline code"),
        "prose should not contain inline code"
    );
    // URLs should NOT appear
    assert!(
        !ctx.prose_text.contains("https://example.com"),
        "prose should not contain URLs"
    );
}
