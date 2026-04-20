// Pass 4: References — markdown chain walking, orphan detection, path safety.
#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag};
use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::config::ValidatorConfig;
use crate::models::{CheckName, Diagnostic, PipelineError, Severity, Sizeyness, SkillContext};

// ── Public entry point ────────────────────────────────────────────────────

/// Run Pass 4 (References) against `skill_dir`.
///
/// 1. Extract references from `ctx.links` and backtick paths in `ctx.prose_text`.
/// 2. Walk markdown chain up to `config.references.markdown_hop_limit` hops.
/// 3. Detect broken references, orphans, circular references, path traversal.
/// 4. Check hooks scripts from frontmatter.
pub fn run(
    skill_dir: &Path,
    ctx: &mut SkillContext,
    config: &ValidatorConfig,
) -> Result<Vec<Diagnostic>, PipelineError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Canonicalize skill_dir for boundary checks.
    // Fall back to the original path if canonicalization fails (e.g. in tests).
    let canon_skill_dir = skill_dir
        .canonicalize()
        .unwrap_or_else(|_| skill_dir.to_path_buf());

    // ── Collect initial references from ctx.links ──────────────────────
    let mut initial_refs: Vec<String> = Vec::new();
    for link in &ctx.links {
        if !is_external_ref(&link.url) {
            initial_refs.push(link.url.clone());
        }
    }

    // ── Extract backtick-quoted file paths from prose_text ─────────────
    let backtick_re = Regex::new(r"`([^`]+\.\w+)`").unwrap();
    for cap in backtick_re.captures_iter(&ctx.prose_text) {
        let path_str = &cap[1];
        // Only include if it looks like a relative path (not a URL, not a command)
        if !path_str.contains("://") && !path_str.starts_with('#') {
            initial_refs.push(path_str.to_string());
        }
    }

    // ── Chain walking state ────────────────────────────────────────────
    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut referenced_files: HashSet<PathBuf> = HashSet::new();

    // SKILL.md is always reachable (hop 0)
    referenced_files.insert(PathBuf::from("SKILL.md"));
    visited.insert(PathBuf::from("SKILL.md"));

    // Process initial references at hop 0 (from SKILL.md)
    let skill_md_dir = skill_dir; // SKILL.md is in the root
    process_references(
        skill_dir,
        &canon_skill_dir,
        skill_md_dir,
        &initial_refs,
        0,
        config.references.markdown_hop_limit,
        &mut visited,
        &mut referenced_files,
        &mut diagnostics,
        ctx.sizeyness,
    );

    // ── Hooks script check ─────────────────────────────────────────────
    check_hooks_scripts(skill_dir, &canon_skill_dir, ctx, &mut diagnostics);

    // ── Orphan detection ───────────────────────────────────────────────
    detect_orphans(
        ctx,
        &referenced_files,
        &config.references.orphan_exclusions,
        &mut diagnostics,
    );

    // ── Populate context ───────────────────────────────────────────────
    ctx.referenced_files = referenced_files;

    Ok(diagnostics)
}

// ── Reference processing (recursive chain walker) ─────────────────────────

#[allow(clippy::too_many_arguments)]
fn process_references(
    skill_dir: &Path,
    canon_skill_dir: &Path,
    containing_dir: &Path,
    refs: &[String],
    current_hop: usize,
    max_hops: usize,
    visited: &mut HashSet<PathBuf>,
    referenced_files: &mut HashSet<PathBuf>,
    diagnostics: &mut Vec<Diagnostic>,
    sizeyness: Sizeyness,
) {
    for ref_path_str in refs {
        // Strip fragment identifiers (e.g., "file.md#section")
        let ref_path_str = ref_path_str.split('#').next().unwrap_or(ref_path_str);
        if ref_path_str.is_empty() {
            continue;
        }

        // NFC normalize the path
        let normalized: String = ref_path_str.nfc().collect();

        // Resolve relative to the containing directory
        let resolved = containing_dir.join(&normalized);

        // Canonicalize to resolve .. and symlinks
        let canonical = match resolved.canonicalize() {
            Ok(p) => p,
            Err(_) => {
                // File doesn't exist — check if it's a traversal attempt
                // by doing a manual path normalization
                let manual = normalize_path(&resolved);
                if !manual.starts_with(canon_skill_dir) && !manual.starts_with(skill_dir) {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Warning,
                        check_name: CheckName::PathTraversalBlocked,
                        human_message: format!(
                            "Reference '{}' resolves outside the skill directory.",
                            ref_path_str
                        ),
                        machine_message: format!("path-traversal:{}", ref_path_str),
                        doc_url: None,
                        file_path: None,
                        base_severity: Severity::Warning,
                    });
                } else {
                    // File genuinely doesn't exist
                    let severity = match sizeyness {
                        Sizeyness::Simple => Severity::Warning,
                        Sizeyness::Moderate | Sizeyness::Hefty => Severity::Error,
                    };
                    diagnostics.push(Diagnostic {
                        severity,
                        check_name: CheckName::BrokenReference,
                        human_message: format!(
                            "Referenced file '{}' does not exist.",
                            ref_path_str
                        ),
                        machine_message: format!("broken:{}", ref_path_str),
                        doc_url: None,
                        file_path: None,
                        base_severity: Severity::Warning,
                    });
                }
                continue;
            }
        };

        // Boundary check: canonical path must be within skill_dir
        if !canonical.starts_with(canon_skill_dir) {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::PathTraversalBlocked,
                human_message: format!(
                    "Reference '{}' resolves outside the skill directory.",
                    ref_path_str
                ),
                machine_message: format!("path-traversal:{}", ref_path_str),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            continue;
        }

        // Get relative path within skill dir
        let relative = canonical
            .strip_prefix(canon_skill_dir)
            .unwrap_or(&canonical)
            .to_path_buf();

        referenced_files.insert(relative.clone());

        // If it's a markdown file, follow the chain
        if is_markdown(&relative) {
            if visited.contains(&relative) {
                // Circular reference
                diagnostics.push(Diagnostic {
                    severity: Severity::Info,
                    check_name: CheckName::CircularReference,
                    human_message: format!(
                        "Circular reference detected involving '{}'.",
                        relative.display()
                    ),
                    machine_message: format!("circular:{}", relative.display()),
                    doc_url: None,
                    file_path: Some(relative),
                    base_severity: Severity::Info,
                });
                continue;
            }

            if current_hop >= max_hops {
                diagnostics.push(Diagnostic {
                    severity: Severity::Info,
                    check_name: CheckName::HopLimitReached,
                    human_message: format!(
                        "Reference chain exceeded {} hops at '{}'.",
                        max_hops,
                        relative.display()
                    ),
                    machine_message: format!("hop-limit:{}:{}", max_hops, relative.display()),
                    doc_url: None,
                    file_path: Some(relative),
                    base_severity: Severity::Info,
                });
                continue;
            }

            visited.insert(relative.clone());

            // Read and parse the markdown file for more links
            if let Ok(content) = std::fs::read_to_string(&canonical) {
                let child_links = extract_links_from_markdown(&content);
                let child_dir = canonical.parent().unwrap_or(canon_skill_dir);

                process_references(
                    skill_dir,
                    canon_skill_dir,
                    child_dir,
                    &child_links,
                    current_hop + 1,
                    max_hops,
                    visited,
                    referenced_files,
                    diagnostics,
                    sizeyness,
                );
            }
        }
    }
}

// ── Hooks script checking ─────────────────────────────────────────────────

fn check_hooks_scripts(
    skill_dir: &Path,
    canon_skill_dir: &Path,
    ctx: &SkillContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(mapping) = ctx.frontmatter.as_mapping() else {
        return;
    };

    let hooks_key = serde_yaml::Value::String("hooks".to_string());
    let Some(hooks_val) = mapping.get(&hooks_key) else {
        return;
    };

    let script_paths = extract_hook_paths(hooks_val);

    for script_path in script_paths {
        let resolved = skill_dir.join(&script_path);
        let exists = resolved
            .canonicalize()
            .map(|p| p.starts_with(canon_skill_dir))
            .unwrap_or(false);

        if !exists {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::HooksScriptMissing,
                human_message: format!(
                    "Hooks reference script '{}' but the file does not exist.",
                    script_path
                ),
                machine_message: format!("hooks-missing:{}", script_path),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Error,
            });
        }
    }
}

/// Extract script paths from a hooks YAML value (string or mapping).
fn extract_hook_paths(val: &serde_yaml::Value) -> Vec<String> {
    let mut paths = Vec::new();
    match val {
        serde_yaml::Value::String(s) => {
            paths.push(s.clone());
        }
        serde_yaml::Value::Mapping(map) => {
            for (_key, v) in map {
                if let serde_yaml::Value::String(s) = v {
                    paths.push(s.clone());
                }
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                if let serde_yaml::Value::String(s) = item {
                    paths.push(s.clone());
                }
            }
        }
        _ => {}
    }
    paths
}

// ── Orphan detection ──────────────────────────────────────────────────────

fn detect_orphans(
    ctx: &SkillContext,
    referenced_files: &HashSet<PathBuf>,
    exclusions: &[String],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut orphans: Vec<String> = Vec::new();

    for entry in &ctx.file_inventory {
        let path = &entry.path;

        // SKILL.md is always reachable
        if path == Path::new("SKILL.md") {
            continue;
        }

        // Check if referenced
        if referenced_files.contains(path) {
            continue;
        }

        // Check exclusion patterns
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let path_str = path.to_string_lossy();

        if exclusions
            .iter()
            .any(|pat| glob_match(pat, filename) || glob_match(pat, &path_str))
        {
            continue;
        }

        orphans.push(path.display().to_string());
    }

    if !orphans.is_empty() {
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate | Sizeyness::Hefty => Severity::Warning,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::OrphanedFiles,
            human_message: format!(
                "These files aren't referenced from any markdown file: {}. \
                 They may still be used by scripts, but the validator can't verify that.",
                orphans.join(", ")
            ),
            machine_message: format!("orphans:{}", orphans.join(",")),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Check if a URL is an external reference (http, mailto, anchor).
fn is_external_ref(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || url.starts_with("mailto:")
        || url.starts_with('#')
}

/// Check if a path has a markdown extension.
fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md"))
}

/// Extract link URLs from markdown content using pulldown-cmark.
fn extract_links_from_markdown(content: &str) -> Vec<String> {
    let parser = Parser::new(content);
    let mut links = Vec::new();

    for event in parser {
        if let Event::Start(Tag::Link { dest_url, .. }) = event {
            let url = dest_url.to_string();
            if !is_external_ref(&url) {
                links.push(url);
            }
        }
    }

    links
}

/// Simple glob matching supporting only `*` as wildcard.
fn glob_match(pattern: &str, text: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if !pattern.contains('*') {
        return pattern == text;
    }

    // Split pattern by `*` and match parts
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 2 {
        let prefix = parts[0];
        let suffix = parts[1];
        return text.starts_with(prefix)
            && text.ends_with(suffix)
            && text.len() >= prefix.len() + suffix.len();
    }

    // General case: greedy matching of parts in order
    let mut remaining = text;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            if !remaining.starts_with(part) {
                return false;
            }
            remaining = &remaining[part.len()..];
        } else if i == parts.len() - 1 {
            if !remaining.ends_with(part) {
                return false;
            }
            return true;
        } else if let Some(pos) = remaining.find(part) {
            remaining = &remaining[pos + part.len()..];
        } else {
            return false;
        }
    }
    true
}

/// Normalize a path by resolving `.` and `..` components without touching the filesystem.
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => {
                components.push(c);
            }
        }
    }
    components.iter().collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_external_ref() {
        assert!(is_external_ref("https://example.com"));
        assert!(is_external_ref("http://example.com"));
        assert!(is_external_ref("mailto:foo@bar.com"));
        assert!(is_external_ref("#section"));
        assert!(!is_external_ref("docs/setup.md"));
        assert!(!is_external_ref("../other.md"));
    }

    #[test]
    fn test_is_markdown() {
        assert!(is_markdown(Path::new("README.md")));
        assert!(is_markdown(Path::new("docs/setup.MD")));
        assert!(!is_markdown(Path::new("script.sh")));
        assert!(!is_markdown(Path::new("data.json")));
    }

    #[test]
    fn test_extract_links_from_markdown() {
        let content = "# Hello\n\nSee [setup](docs/setup.md) and [site](https://example.com).\n";
        let links = extract_links_from_markdown(content);
        assert_eq!(links, vec!["docs/setup.md"]);
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("LICENSE*", "LICENSE"));
        assert!(glob_match("LICENSE*", "LICENSE.txt"));
        assert!(glob_match("LICENSE*", "LICENSE.md"));
        assert!(!glob_match("LICENSE*", "NOLICENSE"));
        assert!(glob_match(".*", ".gitignore"));
        assert!(glob_match(".*", ".env"));
        assert!(glob_match(".gitignore", ".gitignore"));
        assert!(!glob_match(".gitignore", ".env"));
        assert!(glob_match("*", "anything"));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(
            normalize_path(Path::new("/a/b/../c")),
            PathBuf::from("/a/c")
        );
        assert_eq!(normalize_path(Path::new("/a/./b")), PathBuf::from("/a/b"));
    }

    #[test]
    fn test_extract_hook_paths_string() {
        let val = serde_yaml::Value::String("scripts/pre.sh".to_string());
        assert_eq!(extract_hook_paths(&val), vec!["scripts/pre.sh"]);
    }

    #[test]
    fn test_extract_hook_paths_mapping() {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("pre".to_string()),
            serde_yaml::Value::String("scripts/pre.sh".to_string()),
        );
        map.insert(
            serde_yaml::Value::String("post".to_string()),
            serde_yaml::Value::String("scripts/post.sh".to_string()),
        );
        let val = serde_yaml::Value::Mapping(map);
        let paths = extract_hook_paths(&val);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"scripts/pre.sh".to_string()));
        assert!(paths.contains(&"scripts/post.sh".to_string()));
    }
}
