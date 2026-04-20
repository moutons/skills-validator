// Pass 2: Structure — file inventory, sizeyness, binary detection.
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::Path;

use walkdir::WalkDir;

use crate::config::ValidatorConfig;
use crate::models::{
    CheckName, Diagnostic, FileEntry, FileType, PipelineError, Severity, Sizeyness, SkillContext,
};

// ── Constants ───────────────────────────────────────────────────────────────

/// How many bytes to read for null-byte binary detection.
const BINARY_SNIFF_BYTES: usize = 8192;

/// Extensions that are always classified as binary, regardless of content.
const BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "so", "dylib", "wasm", "o", "a", "pyc", "class", "obj", "lib", "bin", "elf",
];

/// Extensions classified as scripts.
const SCRIPT_EXTENSIONS: &[&str] = &["py", "sh", "bash", "rb", "js", "ts", "ps1", "bat", "cmd"];

/// Extensions classified as config.
const CONFIG_EXTENSIONS: &[&str] = &["json", "yaml", "yml", "toml", "jsonc"];

/// Extensions classified as markdown.
const MARKDOWN_EXTENSIONS: &[&str] = &["md"];

// ── Public entry point ──────────────────────────────────────────────────────

/// Run Pass 2 (Structure) against `skill_dir`.
///
/// Walks the directory tree, classifies files, detects binaries, computes
/// sizeyness, and populates `ctx.file_inventory`, `ctx.subdirectories`, and
/// `ctx.sizeyness`.
pub fn run(
    skill_dir: &Path,
    ctx: &mut SkillContext,
    config: &ValidatorConfig,
) -> Result<Vec<Diagnostic>, PipelineError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut file_inventory: Vec<FileEntry> = Vec::new();
    let mut subdirectories: HashSet<std::path::PathBuf> = HashSet::new();

    // Walk directory tree
    let walker = WalkDir::new(skill_dir).min_depth(1).follow_links(false);

    for entry in walker {
        let entry = entry.map_err(|e| PipelineError::IoError {
            path: skill_dir.to_path_buf(),
            reason: format!("walkdir error: {e}"),
        })?;

        let path = entry.path();
        let relative = path.strip_prefix(skill_dir).unwrap_or(path).to_path_buf();

        if entry.file_type().is_dir() {
            subdirectories.insert(relative);
            continue;
        }

        if !entry.file_type().is_file() {
            continue;
        }

        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        let file_type = classify_file(path);

        file_inventory.push(FileEntry {
            path: relative,
            file_type,
            size_bytes,
        });
    }

    // Sort inventory for deterministic output
    file_inventory.sort_by(|a, b| a.path.cmp(&b.path));

    let mut subdir_vec: Vec<_> = subdirectories.into_iter().collect();
    subdir_vec.sort();

    // ── Binary detection diagnostics ────────────────────────────────────
    for entry in &file_inventory {
        if entry.file_type == FileType::Binary {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::BinaryDetected,
                human_message: format!(
                    "Binary file detected: `{}`. Compiled binaries in skills are a security concern.",
                    entry.path.display()
                ),
                machine_message: format!("binary:{}", entry.path.display()),
                doc_url: None,
                file_path: Some(entry.path.clone()),
                base_severity: Severity::Error,
            });
        }
    }

    // ── Compute sizeyness ───────────────────────────────────────────────
    let has_orchestration = check_orchestration_fields(&ctx.frontmatter);
    let sizeyness = compute_sizeyness(
        file_inventory.len(),
        subdir_vec.len(),
        has_orchestration,
        config,
    );

    // ── Scripts-in-root diagnostic ──────────────────────────────────────
    let root_scripts: Vec<_> = file_inventory
        .iter()
        .filter(|f| {
            f.file_type == FileType::Script && f.path.parent().is_none_or(|p| p == Path::new(""))
        })
        .collect();

    if !root_scripts.is_empty() {
        let base_severity = Severity::Suggestion;
        let severity = match sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate | Sizeyness::Hefty => Severity::Warning,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::ScriptsInRoot,
            human_message: "Scripts found in skill root — consider organizing into `scripts/`"
                .to_string(),
            machine_message: format!(
                "scripts-in-root:{}",
                root_scripts
                    .iter()
                    .map(|f| f.path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            doc_url: None,
            file_path: None,
            base_severity,
        });
    }

    // ── Sizeyness info diagnostic ───────────────────────────────────────
    let tier_label = match sizeyness {
        Sizeyness::Simple => "Simple",
        Sizeyness::Moderate => "Moderate",
        Sizeyness::Hefty => "Hefty",
    };

    let mut reasons = Vec::new();
    reasons.push(format!("{} files", file_inventory.len()));
    reasons.push(format!("{} subdirectories", subdir_vec.len()));
    if has_orchestration {
        reasons.push("has orchestration fields".to_string());
    }
    let reasons_str = reasons.join(", ");

    diagnostics.push(Diagnostic {
        severity: Severity::Info,
        check_name: CheckName::SizeynessInfo,
        human_message: format!("Skill classified as {tier_label} ({reasons_str})"),
        machine_message: format!(
            "sizeyness:{}:files={}:subdirs={}:orchestration={}",
            tier_label.to_lowercase(),
            file_inventory.len(),
            subdir_vec.len(),
            has_orchestration
        ),
        doc_url: None,
        file_path: None,
        base_severity: Severity::Info,
    });

    // ── Populate context ────────────────────────────────────────────────
    ctx.file_inventory = file_inventory;
    ctx.subdirectories = subdir_vec;
    ctx.sizeyness = sizeyness;

    Ok(diagnostics)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Classify a file based on its extension and content.
fn classify_file(path: &Path) -> FileType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Check known binary extensions first
    if BINARY_EXTENSIONS.contains(&ext.as_str()) {
        return FileType::Binary;
    }

    // Check for null bytes in content (binary sniffing)
    if has_null_bytes(path) {
        return FileType::Binary;
    }

    // Classify by extension
    if MARKDOWN_EXTENSIONS.contains(&ext.as_str()) {
        FileType::Markdown
    } else if SCRIPT_EXTENSIONS.contains(&ext.as_str()) {
        FileType::Script
    } else if CONFIG_EXTENSIONS.contains(&ext.as_str()) {
        FileType::Config
    } else {
        FileType::Other
    }
}

/// Read up to BINARY_SNIFF_BYTES and check for null bytes.
fn has_null_bytes(path: &Path) -> bool {
    use std::io::Read;
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut buf = vec![0u8; BINARY_SNIFF_BYTES];
    let Ok(n) = file.read(&mut buf) else {
        return false;
    };
    buf[..n].contains(&0)
}

/// Check if the frontmatter contains orchestration fields (hooks, agent, context).
fn check_orchestration_fields(frontmatter: &serde_yaml::Value) -> bool {
    let Some(map) = frontmatter.as_mapping() else {
        return false;
    };
    for key in ["hooks", "agent", "context"] {
        if map.contains_key(serde_yaml::Value::String(key.to_string())) {
            return true;
        }
    }
    false
}

/// Compute sizeyness tier using config thresholds.
fn compute_sizeyness(
    file_count: usize,
    subdir_count: usize,
    has_orchestration: bool,
    config: &ValidatorConfig,
) -> Sizeyness {
    if has_orchestration
        || file_count >= config.sizeyness.hefty_file_threshold
        || subdir_count >= config.sizeyness.hefty_subdir_threshold
    {
        Sizeyness::Hefty
    } else if file_count >= config.sizeyness.moderate_file_threshold
        || subdir_count >= config.sizeyness.moderate_subdir_threshold
    {
        Sizeyness::Moderate
    } else {
        Sizeyness::Simple
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_markdown() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("README.md");
        std::fs::write(&p, "# Hello").unwrap();
        assert_eq!(classify_file(&p), FileType::Markdown);
    }

    #[test]
    fn classify_script_extensions() {
        let dir = tempfile::tempdir().unwrap();
        for ext in SCRIPT_EXTENSIONS {
            let p = dir.path().join(format!("test.{ext}"));
            std::fs::write(&p, "content").unwrap();
            assert_eq!(classify_file(&p), FileType::Script, "failed for .{ext}");
        }
    }

    #[test]
    fn classify_config_extensions() {
        let dir = tempfile::tempdir().unwrap();
        for ext in CONFIG_EXTENSIONS {
            let p = dir.path().join(format!("test.{ext}"));
            std::fs::write(&p, "content").unwrap();
            assert_eq!(classify_file(&p), FileType::Config, "failed for .{ext}");
        }
    }

    #[test]
    fn classify_binary_extension() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("lib.so");
        std::fs::write(&p, "not really binary").unwrap();
        assert_eq!(classify_file(&p), FileType::Binary);
    }

    #[test]
    fn classify_binary_by_null_bytes() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("data.dat");
        std::fs::write(&p, b"hello\x00world").unwrap();
        assert_eq!(classify_file(&p), FileType::Binary);
    }

    #[test]
    fn classify_other() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("notes.txt");
        std::fs::write(&p, "notes").unwrap();
        assert_eq!(classify_file(&p), FileType::Other);
    }

    #[test]
    fn orchestration_detection() {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String("test".to_string()),
        );
        assert!(!check_orchestration_fields(&serde_yaml::Value::Mapping(
            map.clone()
        )));

        map.insert(
            serde_yaml::Value::String("hooks".to_string()),
            serde_yaml::Value::String("pre".to_string()),
        );
        assert!(check_orchestration_fields(&serde_yaml::Value::Mapping(map)));
    }

    #[test]
    fn compute_sizeyness_defaults() {
        let config = ValidatorConfig::default();
        assert_eq!(compute_sizeyness(1, 0, false, &config), Sizeyness::Simple);
        assert_eq!(compute_sizeyness(2, 0, false, &config), Sizeyness::Simple);
        assert_eq!(compute_sizeyness(3, 0, false, &config), Sizeyness::Moderate);
        assert_eq!(compute_sizeyness(5, 0, false, &config), Sizeyness::Moderate);
        assert_eq!(compute_sizeyness(6, 0, false, &config), Sizeyness::Hefty);
        assert_eq!(compute_sizeyness(1, 1, false, &config), Sizeyness::Moderate);
        assert_eq!(compute_sizeyness(1, 3, false, &config), Sizeyness::Hefty);
        assert_eq!(compute_sizeyness(1, 0, true, &config), Sizeyness::Hefty);
    }
}
