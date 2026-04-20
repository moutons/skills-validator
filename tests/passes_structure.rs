//! Integration tests for Pass 2 (Structure).

use std::fs;
use std::path::Path;

use skills_validator::config::ValidatorConfig;
use skills_validator::models::{CheckName, FileType, Severity, Sizeyness, SkillContext};
use skills_validator::passes::structure;

// ─── Helper ─────────────────────────────────────────────────────────────────

fn fixture(rel: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}

fn default_ctx() -> SkillContext {
    SkillContext::default()
}

// ─── File inventory from minimal fixture ────────────────────────────────────

#[test]
fn inventories_single_file_skill() {
    let dir = fixture("valid/minimal/coding-standards");
    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();

    let diags = structure::run(&dir, &mut ctx, &config).unwrap();

    // Should have exactly one file: SKILL.md
    assert_eq!(ctx.file_inventory.len(), 1);
    assert_eq!(ctx.file_inventory[0].file_type, FileType::Markdown);
    assert!(ctx.file_inventory[0].path.ends_with("SKILL.md"));
    assert!(ctx.file_inventory[0].size_bytes > 0);

    // No subdirectories
    assert!(ctx.subdirectories.is_empty());

    // Should be Simple sizeyness
    assert_eq!(ctx.sizeyness, Sizeyness::Simple);

    // Should have a sizeyness-info diagnostic
    assert!(diags
        .iter()
        .any(|d| d.check_name == CheckName::SizeynessInfo));
}

// ─── File inventory from multi-file fixture ─────────────────────────────────

#[test]
fn inventories_multi_file_skill() {
    let dir = fixture("valid/multi-file/security-ownership-map");
    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();

    let diags = structure::run(&dir, &mut ctx, &config).unwrap();

    // Should have multiple files
    assert!(ctx.file_inventory.len() > 1, "expected multiple files");

    // Should have subdirectories (agents, references, scripts)
    assert!(!ctx.subdirectories.is_empty(), "expected subdirectories");

    // Should have scripts
    let scripts: Vec<_> = ctx
        .file_inventory
        .iter()
        .filter(|f| f.file_type == FileType::Script)
        .collect();
    assert!(!scripts.is_empty(), "expected script files");

    // With many files + subdirs, should be at least Moderate
    assert_ne!(ctx.sizeyness, Sizeyness::Simple);

    // Should have sizeyness-info
    assert!(diags
        .iter()
        .any(|d| d.check_name == CheckName::SizeynessInfo));
}

// ─── File type classification ───────────────────────────────────────────────

#[test]
fn classifies_file_types_correctly() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("helper.py"), "print('hi')").unwrap();
    fs::write(dir.path().join("run.sh"), "#!/bin/bash").unwrap();
    fs::write(dir.path().join("config.json"), "{}").unwrap();
    fs::write(dir.path().join("config.yaml"), "key: val").unwrap();
    fs::write(dir.path().join("notes.txt"), "some notes").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    let find_type = |name: &str| -> FileType {
        ctx.file_inventory
            .iter()
            .find(|f| f.path.file_name().unwrap().to_str().unwrap() == name)
            .unwrap()
            .file_type
            .clone()
    };

    assert_eq!(find_type("SKILL.md"), FileType::Markdown);
    assert_eq!(find_type("helper.py"), FileType::Script);
    assert_eq!(find_type("run.sh"), FileType::Script);
    assert_eq!(find_type("config.json"), FileType::Config);
    assert_eq!(find_type("config.yaml"), FileType::Config);
    assert_eq!(find_type("notes.txt"), FileType::Other);
}

// ─── Binary detection ───────────────────────────────────────────────────────

#[test]
fn detects_binary_by_null_bytes() {
    let dir = fixture("binary-in-skill");
    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();

    let diags = structure::run(&dir, &mut ctx, &config).unwrap();

    // Should detect helper.dat as binary (via null-byte sniffing)
    let binary_files: Vec<_> = ctx
        .file_inventory
        .iter()
        .filter(|f| f.file_type == FileType::Binary)
        .collect();
    assert_eq!(binary_files.len(), 1, "expected one binary file");
    assert!(binary_files[0].path.ends_with("helper.dat"));

    // Should have binary-detected error
    let binary_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.check_name == CheckName::BinaryDetected)
        .collect();
    assert_eq!(binary_diags.len(), 1);
    assert_eq!(binary_diags[0].severity, Severity::Error);
    assert!(binary_diags[0].human_message.contains("helper.dat"));
}

#[test]
fn detects_binary_by_extension() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    // Write a .exe file with no null bytes (extension-based detection)
    fs::write(dir.path().join("tool.exe"), "not actually binary content").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    let diags = structure::run(dir.path(), &mut ctx, &config).unwrap();

    let binary_files: Vec<_> = ctx
        .file_inventory
        .iter()
        .filter(|f| f.file_type == FileType::Binary)
        .collect();
    assert_eq!(binary_files.len(), 1);
    assert!(binary_files[0].path.ends_with("tool.exe"));

    assert!(diags
        .iter()
        .any(|d| d.check_name == CheckName::BinaryDetected));
}

// ─── Sizeyness tiers ────────────────────────────────────────────────────────

#[test]
fn sizeyness_simple_with_two_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("notes.txt"), "notes").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    assert_eq!(ctx.sizeyness, Sizeyness::Simple);
}

#[test]
fn sizeyness_moderate_at_three_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    // 3 files -> moderate (default threshold)
    assert_eq!(ctx.sizeyness, Sizeyness::Moderate);
}

#[test]
fn sizeyness_hefty_at_six_files() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    for i in 1..=5 {
        fs::write(dir.path().join(format!("file{i}.txt")), "content").unwrap();
    }

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    // 6 files -> hefty (default threshold)
    assert_eq!(ctx.sizeyness, Sizeyness::Hefty);
}

// ─── Subdirectory counting ──────────────────────────────────────────────────

#[test]
fn counts_subdirectories() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::create_dir(dir.path().join("scripts")).unwrap();
    fs::write(dir.path().join("scripts/run.sh"), "#!/bin/bash").unwrap();
    fs::create_dir(dir.path().join("docs")).unwrap();
    fs::write(dir.path().join("docs/README.md"), "# Docs").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    assert_eq!(ctx.subdirectories.len(), 2);
    // 3 files, 2 subdirs -> at least Moderate
    assert_ne!(ctx.sizeyness, Sizeyness::Simple);
}

#[test]
fn sizeyness_moderate_with_one_subdir() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("sub/file.txt"), "content").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    // 1 subdir -> moderate (default threshold is 1)
    assert_eq!(ctx.sizeyness, Sizeyness::Moderate);
}

#[test]
fn sizeyness_hefty_with_three_subdirs() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    for name in ["a", "b", "c"] {
        fs::create_dir(dir.path().join(name)).unwrap();
        fs::write(dir.path().join(format!("{name}/f.txt")), "x").unwrap();
    }

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    // 3 subdirs -> hefty (default threshold is 3)
    assert_eq!(ctx.sizeyness, Sizeyness::Hefty);
}

// ─── Orchestration field promotion ──────────────────────────────────────────

#[test]
fn orchestration_promotes_to_hefty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();

    let mut ctx = default_ctx();
    // Simulate frontmatter with hooks field (set by parse pass)
    let mut map = serde_yaml::Mapping::new();
    map.insert(
        serde_yaml::Value::String("name".to_string()),
        serde_yaml::Value::String("test".to_string()),
    );
    map.insert(
        serde_yaml::Value::String("hooks".to_string()),
        serde_yaml::Value::String("pre-commit".to_string()),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(map);

    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    // Even with 1 file and 0 subdirs, orchestration -> hefty
    assert_eq!(ctx.sizeyness, Sizeyness::Hefty);
}

#[test]
fn agent_field_promotes_to_hefty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();

    let mut ctx = default_ctx();
    let mut map = serde_yaml::Mapping::new();
    map.insert(
        serde_yaml::Value::String("agent".to_string()),
        serde_yaml::Value::String("true".to_string()),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(map);

    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    assert_eq!(ctx.sizeyness, Sizeyness::Hefty);
}

#[test]
fn context_field_promotes_to_hefty() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();

    let mut ctx = default_ctx();
    let mut map = serde_yaml::Mapping::new();
    map.insert(
        serde_yaml::Value::String("context".to_string()),
        serde_yaml::Value::String("project".to_string()),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(map);

    let config = ValidatorConfig::default();
    structure::run(dir.path(), &mut ctx, &config).unwrap();

    assert_eq!(ctx.sizeyness, Sizeyness::Hefty);
}

// ─── Scripts-in-root diagnostic ─────────────────────────────────────────────

#[test]
fn scripts_in_root_emits_suggestion_for_simple() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("helper.py"), "print('hi')").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    let diags = structure::run(dir.path(), &mut ctx, &config).unwrap();

    let script_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.check_name == CheckName::ScriptsInRoot)
        .collect();
    assert_eq!(script_diags.len(), 1);
    assert_eq!(script_diags[0].severity, Severity::Suggestion);
}

#[test]
fn scripts_in_root_escalates_for_moderate() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("helper.py"), "print('hi')").unwrap();
    fs::write(dir.path().join("extra.txt"), "x").unwrap();

    let mut ctx = default_ctx();
    let config = ValidatorConfig::default();
    let diags = structure::run(dir.path(), &mut ctx, &config).unwrap();

    // 3 files -> moderate, scripts-in-root escalates to warning
    let script_diags: Vec<_> = diags
        .iter()
        .filter(|d| d.check_name == CheckName::ScriptsInRoot)
        .collect();
    assert_eq!(script_diags.len(), 1);
    assert_eq!(script_diags[0].severity, Severity::Warning);
}

// ─── Config thresholds are respected ────────────────────────────────────────

#[test]
fn custom_config_thresholds_respected() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("SKILL.md"), "---\nname: test\n---\n# Hi\n").unwrap();
    fs::write(dir.path().join("a.txt"), "a").unwrap();
    fs::write(dir.path().join("b.txt"), "b").unwrap();
    fs::write(dir.path().join("c.txt"), "c").unwrap();
    fs::write(dir.path().join("d.txt"), "d").unwrap();

    let mut ctx = default_ctx();
    let mut config = ValidatorConfig::default();
    // Raise thresholds so 5 files is still simple
    config.sizeyness.moderate_file_threshold = 10;
    config.sizeyness.hefty_file_threshold = 20;

    structure::run(dir.path(), &mut ctx, &config).unwrap();

    assert_eq!(ctx.sizeyness, Sizeyness::Simple);
}
