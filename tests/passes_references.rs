//! Integration tests for Pass 4 (References).

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use skills_validator::config::ValidatorConfig;
use skills_validator::models::{
    CheckName, FileEntry, FileType, Link, Severity, Sizeyness, SkillContext,
};
use skills_validator::passes::references;

// ─── Helpers ───────────────────────────────────────────────────────────────

fn fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/skills")
        .join(rel)
}

fn default_ctx() -> SkillContext {
    SkillContext::default()
}

fn ctx_with_links(links: Vec<Link>) -> SkillContext {
    let mut ctx = default_ctx();
    ctx.links = links;
    ctx
}

fn link(text: &str, url: &str) -> Link {
    Link {
        text: text.to_string(),
        url: url.to_string(),
    }
}

fn has_check(diags: &[skills_validator::models::Diagnostic], check: CheckName) -> bool {
    diags.iter().any(|d| d.check_name == check)
}

fn count_check(diags: &[skills_validator::models::Diagnostic], check: CheckName) -> usize {
    diags.iter().filter(|d| d.check_name == check).count()
}

// ─── Link extraction ───────────────────────────────────────────────────────

#[test]
fn extracts_markdown_links_from_ctx() {
    let dir = fixture("broken-ref");
    let mut ctx = default_ctx();
    ctx.links = vec![link("setup guide", "docs/setup.md")];
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 100,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    // docs/setup.md doesn't exist, so we should get a broken-reference
    assert!(has_check(&diags, CheckName::BrokenReference));
}

// ─── Backtick path extraction ──────────────────────────────────────────────

#[test]
fn extracts_backtick_paths_from_prose() {
    let dir = fixture("broken-ref");
    let mut ctx = default_ctx();
    ctx.prose_text = "Check `scripts/missing.sh` for the script.".to_string();
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 100,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    // scripts/missing.sh doesn't exist
    assert!(has_check(&diags, CheckName::BrokenReference));
}

// ─── Path traversal blocked ────────────────────────────────────────────────

#[test]
fn rejects_path_traversal() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test").unwrap();

    let mut ctx = default_ctx();
    ctx.links = vec![link("evil", "../../etc/passwd")];
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 50,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    assert!(has_check(&diags, CheckName::PathTraversalBlocked));
}

// ─── Broken references ────────────────────────────────────────────────────

#[test]
fn broken_ref_fixture() {
    let dir = fixture("broken-ref");
    let mut ctx = default_ctx();
    ctx.links = vec![link("setup guide", "docs/setup.md")];
    ctx.prose_text = "Also check `scripts/missing.sh` for the script.".to_string();
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 100,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    let broken_refs: Vec<_> = diags
        .iter()
        .filter(|d| d.check_name == CheckName::BrokenReference)
        .collect();
    assert!(broken_refs.len() >= 2, "expected at least 2 broken refs");
}

#[test]
fn broken_ref_severity_escalates_with_sizeyness() {
    let dir = fixture("broken-ref");

    // Simple sizeyness: warning
    let mut ctx = default_ctx();
    ctx.sizeyness = Sizeyness::Simple;
    ctx.links = vec![link("setup guide", "docs/setup.md")];
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 100,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();
    let broken = diags
        .iter()
        .find(|d| d.check_name == CheckName::BrokenReference)
        .unwrap();
    assert_eq!(broken.severity, Severity::Warning);

    // Moderate sizeyness: error
    let mut ctx2 = default_ctx();
    ctx2.sizeyness = Sizeyness::Moderate;
    ctx2.links = vec![link("setup guide", "docs/setup.md")];
    ctx2.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 100,
    }];

    let diags2 = references::run(&dir, &mut ctx2, &config).unwrap();
    let broken2 = diags2
        .iter()
        .find(|d| d.check_name == CheckName::BrokenReference)
        .unwrap();
    assert_eq!(broken2.severity, Severity::Error);
}

// ─── Orphan detection ──────────────────────────────────────────────────────

#[test]
fn orphaned_files_fixture() {
    let dir = fixture("orphaned-files");
    let mut ctx = default_ctx();
    ctx.links = vec![link("notes", "notes.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("notes.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("orphan.txt"),
            file_type: FileType::Other,
            size_bytes: 40,
        },
        FileEntry {
            path: PathBuf::from("LICENSE"),
            file_type: FileType::Other,
            size_bytes: 12,
        },
    ];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    // orphan.txt should be flagged, LICENSE should be excluded
    assert!(has_check(&diags, CheckName::OrphanedFiles));
    let orphan_diag = diags
        .iter()
        .find(|d| d.check_name == CheckName::OrphanedFiles)
        .unwrap();
    assert!(orphan_diag.machine_message.contains("orphan.txt"));
    assert!(!orphan_diag.machine_message.contains("LICENSE"));
}

#[test]
fn license_excluded_from_orphan_detection() {
    let dir = fixture("orphaned-files");
    let mut ctx = default_ctx();
    ctx.links = vec![link("notes", "notes.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("notes.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("LICENSE"),
            file_type: FileType::Other,
            size_bytes: 12,
        },
    ];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    // No orphan diagnostic since LICENSE is excluded
    assert!(!has_check(&diags, CheckName::OrphanedFiles));
}

// ─── Circular reference detection ──────────────────────────────────────────

#[test]
fn circular_ref_fixture() {
    let dir = fixture("circular-ref");
    let mut ctx = default_ctx();
    ctx.links = vec![link("page A", "a.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("a.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("b.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
    ];
    let config = ValidatorConfig::default();

    let diags = references::run(&dir, &mut ctx, &config).unwrap();

    assert!(has_check(&diags, CheckName::CircularReference));
}

#[test]
fn circular_ref_does_not_infinite_loop() {
    let dir = fixture("circular-ref");
    let mut ctx = default_ctx();
    ctx.links = vec![link("page A", "a.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("a.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("b.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
    ];
    let config = ValidatorConfig::default();

    // If this returns at all, infinite loop is avoided
    let _diags = references::run(&dir, &mut ctx, &config).unwrap();

    // All markdown files should still be reachable
    assert!(ctx.referenced_files.contains(&PathBuf::from("a.md")));
    assert!(ctx.referenced_files.contains(&PathBuf::from("b.md")));
}

// ─── Hop limit ─────────────────────────────────────────────────────────────

#[test]
fn hop_limit_reached() {
    // Create a chain: SKILL.md -> 1.md -> 2.md -> 3.md with hop_limit=2
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();

    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test\n---\n# Test\nSee [one](1.md)",
    )
    .unwrap();
    fs::write(skill_dir.join("1.md"), "# One\nSee [two](2.md)").unwrap();
    fs::write(skill_dir.join("2.md"), "# Two\nSee [three](3.md)").unwrap();
    fs::write(skill_dir.join("3.md"), "# Three\nDone.").unwrap();

    let mut ctx = default_ctx();
    ctx.links = vec![link("one", "1.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("1.md"),
            file_type: FileType::Markdown,
            size_bytes: 30,
        },
        FileEntry {
            path: PathBuf::from("2.md"),
            file_type: FileType::Markdown,
            size_bytes: 30,
        },
        FileEntry {
            path: PathBuf::from("3.md"),
            file_type: FileType::Markdown,
            size_bytes: 20,
        },
    ];

    let mut config = ValidatorConfig::default();
    config.references.markdown_hop_limit = 2;

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    assert!(has_check(&diags, CheckName::HopLimitReached));
}

// ─── Hooks script missing ──────────────────────────────────────────────────

#[test]
fn hooks_script_missing_string() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: test\nhooks: scripts/pre-run.sh\n---\n# Test",
    )
    .unwrap();

    let mut ctx = default_ctx();
    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::String("hooks".to_string()),
        serde_yaml::Value::String("scripts/pre-run.sh".to_string()),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(fm);
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 50,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    assert!(has_check(&diags, CheckName::HooksScriptMissing));
    let hook_diag = diags
        .iter()
        .find(|d| d.check_name == CheckName::HooksScriptMissing)
        .unwrap();
    assert_eq!(hook_diag.severity, Severity::Error);
}

#[test]
fn hooks_script_missing_mapping() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test").unwrap();

    let mut ctx = default_ctx();
    let mut hooks_map = serde_yaml::Mapping::new();
    hooks_map.insert(
        serde_yaml::Value::String("pre".to_string()),
        serde_yaml::Value::String("scripts/pre.sh".to_string()),
    );
    hooks_map.insert(
        serde_yaml::Value::String("post".to_string()),
        serde_yaml::Value::String("scripts/post.sh".to_string()),
    );
    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::String("hooks".to_string()),
        serde_yaml::Value::Mapping(hooks_map),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(fm);
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 50,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    // Both scripts are missing
    assert_eq!(count_check(&diags, CheckName::HooksScriptMissing), 2);
}

#[test]
fn hooks_script_exists_no_diagnostic() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test").unwrap();
    fs::create_dir_all(skill_dir.join("scripts")).unwrap();
    fs::write(skill_dir.join("scripts/pre.sh"), "#!/bin/bash\necho hi").unwrap();

    let mut ctx = default_ctx();
    let mut fm = serde_yaml::Mapping::new();
    fm.insert(
        serde_yaml::Value::String("hooks".to_string()),
        serde_yaml::Value::String("scripts/pre.sh".to_string()),
    );
    ctx.frontmatter = serde_yaml::Value::Mapping(fm);
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
        FileEntry {
            path: PathBuf::from("scripts/pre.sh"),
            file_type: FileType::Script,
            size_bytes: 20,
        },
    ];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    assert!(!has_check(&diags, CheckName::HooksScriptMissing));
}

// ─── Symlink boundary check ───────────────────────────────────────────────

#[cfg(unix)]
#[test]
fn symlink_outside_skill_dir_is_broken() {
    use std::os::unix::fs::symlink;

    let outside = tempfile::tempdir().unwrap();
    fs::write(outside.path().join("secret.md"), "# Secret").unwrap();

    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test").unwrap();
    symlink(
        outside.path().join("secret.md"),
        skill_dir.join("secret.md"),
    )
    .unwrap();

    let mut ctx = default_ctx();
    ctx.links = vec![link("secret", "secret.md")];
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 50,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    // Symlink pointing outside should be caught as path traversal
    assert!(has_check(&diags, CheckName::PathTraversalBlocked));
}

// ─── Skips external URLs and anchors ───────────────────────────────────────

#[test]
fn skips_http_and_anchor_links() {
    let dir = tempfile::tempdir().unwrap();
    let skill_dir = dir.path();
    fs::write(skill_dir.join("SKILL.md"), "---\nname: test\n---\n# Test").unwrap();

    let mut ctx = default_ctx();
    ctx.links = vec![
        link("site", "https://example.com"),
        link("mailto", "mailto:foo@bar.com"),
        link("anchor", "#section"),
    ];
    ctx.file_inventory = vec![FileEntry {
        path: PathBuf::from("SKILL.md"),
        file_type: FileType::Markdown,
        size_bytes: 50,
    }];
    let config = ValidatorConfig::default();

    let diags = references::run(skill_dir, &mut ctx, &config).unwrap();

    // None of these should produce broken-reference or path-traversal
    assert!(!has_check(&diags, CheckName::BrokenReference));
    assert!(!has_check(&diags, CheckName::PathTraversalBlocked));
}

// ─── Referenced files populated ────────────────────────────────────────────

#[test]
fn referenced_files_populated() {
    let dir = fixture("orphaned-files");
    let mut ctx = default_ctx();
    ctx.links = vec![link("notes", "notes.md")];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("notes.md"),
            file_type: FileType::Markdown,
            size_bytes: 50,
        },
    ];
    let config = ValidatorConfig::default();

    references::run(&dir, &mut ctx, &config).unwrap();

    assert!(ctx.referenced_files.contains(&PathBuf::from("SKILL.md")));
    assert!(ctx.referenced_files.contains(&PathBuf::from("notes.md")));
}

// ─── Orphan severity escalation ────────────────────────────────────────────

#[test]
fn orphan_severity_escalates() {
    let dir = fixture("orphaned-files");

    // Simple: suggestion
    let mut ctx = default_ctx();
    ctx.sizeyness = Sizeyness::Simple;
    ctx.links = vec![];
    ctx.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("orphan.txt"),
            file_type: FileType::Other,
            size_bytes: 40,
        },
    ];
    let config = ValidatorConfig::default();
    let diags = references::run(&dir, &mut ctx, &config).unwrap();
    let orphan = diags
        .iter()
        .find(|d| d.check_name == CheckName::OrphanedFiles)
        .unwrap();
    assert_eq!(orphan.severity, Severity::Suggestion);

    // Moderate: warning
    let mut ctx2 = default_ctx();
    ctx2.sizeyness = Sizeyness::Moderate;
    ctx2.links = vec![];
    ctx2.file_inventory = vec![
        FileEntry {
            path: PathBuf::from("SKILL.md"),
            file_type: FileType::Markdown,
            size_bytes: 100,
        },
        FileEntry {
            path: PathBuf::from("orphan.txt"),
            file_type: FileType::Other,
            size_bytes: 40,
        },
    ];
    let diags2 = references::run(&dir, &mut ctx2, &config).unwrap();
    let orphan2 = diags2
        .iter()
        .find(|d| d.check_name == CheckName::OrphanedFiles)
        .unwrap();
    assert_eq!(orphan2.severity, Severity::Warning);
}
