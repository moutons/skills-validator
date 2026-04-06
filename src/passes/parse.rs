// Pass 1: Parse — find SKILL.md, extract frontmatter, parse body with pulldown-cmark.
//
// Returns `(SkillContext, Vec<Diagnostic>)` on success, or `PipelineError` on
// fatal failure (e.g. I/O error reading the file).
#![allow(dead_code)]

use std::path::Path;

use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};

use crate::models::{
    CheckName, CodeBlock, Diagnostic, Heading, Link, PipelineError, Severity, SkillContext,
};
use crate::parser::parse_frontmatter;

// ── public entry point ──────────────────────────────────────────────────────

/// Run Pass 1 (Parse) against `skill_dir`.
///
/// 1. Locate exactly `SKILL.md` (case-sensitive).
/// 2. Extract YAML frontmatter.
/// 3. Parse the markdown body into typed collections.
pub fn run(skill_dir: &Path) -> Result<(SkillContext, Vec<Diagnostic>), PipelineError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // ── Step 1: find SKILL.md ───────────────────────────────────────────
    // We must check exact casing by reading the directory listing, because
    // case-insensitive filesystems (macOS default) treat `skill.md` the
    // same as `SKILL.md` at the OS level.
    let skill_md_path = skill_dir.join("SKILL.md");

    match find_exact_skill_md(skill_dir) {
        SkillMdLookup::ExactMatch => { /* good, proceed */ }
        SkillMdLookup::WrongCasing(wrong_name) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::SkillFileCasing,
                human_message: format!(
                    "Found '{}' but the file must be named exactly 'SKILL.md'. Please rename it.",
                    wrong_name
                ),
                machine_message: format!("wrong-casing:{}", wrong_name),
                doc_url: None,
                file_path: Some(skill_dir.join(&wrong_name)),
                base_severity: Severity::Error,
            });
            return Err(PipelineError::ParseFailed {
                path: skill_dir.to_path_buf(),
                reason: format!(
                    "SKILL.md not found (found '{}' with wrong casing)",
                    wrong_name
                ),
            });
        }
        SkillMdLookup::NotFound => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::SkillFileExists,
                human_message: "SKILL.md not found in the skill directory.".to_string(),
                machine_message: "missing".to_string(),
                doc_url: None,
                file_path: Some(skill_dir.to_path_buf()),
                base_severity: Severity::Error,
            });
            return Err(PipelineError::ParseFailed {
                path: skill_dir.to_path_buf(),
                reason: "SKILL.md not found".to_string(),
            });
        }
    }

    // ── Step 2: read file ───────────────────────────────────────────────
    let content = std::fs::read_to_string(&skill_md_path).map_err(|e| PipelineError::IoError {
        path: skill_md_path.clone(),
        reason: e.to_string(),
    })?;

    // ── Step 3: extract frontmatter ─────────────────────────────────────
    let (frontmatter, body) = match parse_frontmatter(&content) {
        Ok(pair) => pair,
        Err(e) => {
            let msg = e.to_string();
            let check = if msg.contains("must start with") || msg.contains("not properly closed") {
                CheckName::FrontmatterPresent
            } else if msg.contains("must be a YAML mapping") {
                CheckName::FrontmatterIsMapping
            } else {
                CheckName::FrontmatterValidYaml
            };

            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: check,
                human_message: msg.clone(),
                machine_message: msg.clone(),
                doc_url: None,
                file_path: Some(skill_md_path.clone()),
                base_severity: Severity::Error,
            });
            return Err(PipelineError::ParseFailed {
                path: skill_md_path,
                reason: msg,
            });
        }
    };

    // ── Step 4: parse markdown body ─────────────────────────────────────
    let (headings, links, code_blocks, prose_text) = extract_markdown(&body);

    let ctx = SkillContext {
        frontmatter,
        headings,
        links,
        code_blocks,
        prose_text,
        ..Default::default()
    };

    Ok((ctx, diagnostics))
}

// ── helpers ─────────────────────────────────────────────────────────────────

enum SkillMdLookup {
    ExactMatch,
    WrongCasing(String),
    NotFound,
}

/// Scan the directory listing to find `SKILL.md` by exact name.
///
/// This avoids false positives on case-insensitive filesystems (macOS) where
/// `Path::exists()` would return true for any casing variant.
fn find_exact_skill_md(dir: &Path) -> SkillMdLookup {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return SkillMdLookup::NotFound,
    };

    let mut wrong_casing: Option<String> = None;

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str == "SKILL.md" {
            return SkillMdLookup::ExactMatch;
        }
        if name_str.eq_ignore_ascii_case("skill.md") {
            wrong_casing = Some(name_str.into_owned());
        }
    }

    match wrong_casing {
        Some(name) => SkillMdLookup::WrongCasing(name),
        None => SkillMdLookup::NotFound,
    }
}

/// Walk the pulldown-cmark event stream and extract headings, links, code
/// blocks, and a prose-only text view.
fn extract_markdown(body: &str) -> (Vec<Heading>, Vec<Link>, Vec<CodeBlock>, String) {
    let parser = Parser::new(body);

    let mut headings = Vec::new();
    let mut links = Vec::new();
    let mut code_blocks = Vec::new();
    let mut prose_parts: Vec<String> = Vec::new();

    // State tracking
    let mut in_heading = false;
    let mut heading_level: u8 = 0;
    let mut heading_text = String::new();

    let mut in_link = false;
    let mut link_url = String::new();
    let mut link_text = String::new();

    let mut in_code_block = false;
    let mut code_lang: Option<String> = None;
    let mut code_content = String::new();

    for event in parser {
        match event {
            // ── Headings ────────────────────────────────────────────
            Event::Start(Tag::Heading { level, .. }) => {
                in_heading = true;
                heading_level = level as u8;
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                in_heading = false;
                headings.push(Heading {
                    level: heading_level,
                    text: heading_text.clone(),
                });
                // Headings contribute to prose
                prose_parts.push(heading_text.clone());
            }

            // ── Links ───────────────────────────────────────────────
            Event::Start(Tag::Link { dest_url, .. }) => {
                in_link = true;
                link_url = dest_url.to_string();
                link_text.clear();
            }
            Event::End(TagEnd::Link) => {
                in_link = false;
                links.push(Link {
                    text: link_text.clone(),
                    url: link_url.clone(),
                });
                // Link text (not URL) contributes to prose
                prose_parts.push(link_text.clone());
            }

            // ── Fenced code blocks ─────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                in_code_block = true;
                code_lang = match &kind {
                    CodeBlockKind::Fenced(lang) => {
                        let l = lang.trim().to_string();
                        if l.is_empty() {
                            None
                        } else {
                            Some(l)
                        }
                    }
                    CodeBlockKind::Indented => None,
                };
                code_content.clear();
            }
            Event::End(TagEnd::CodeBlock) => {
                in_code_block = false;
                code_blocks.push(CodeBlock {
                    language: code_lang.take(),
                    content: code_content.clone(),
                });
                // Fenced code blocks do NOT contribute to prose.
            }

            // ── Inline code ─────────────────────────────────────────
            Event::Code(_) => {
                // Inline code is stripped from prose — intentionally ignored.
            }

            // ── Text ────────────────────────────────────────────────
            Event::Text(text) => {
                if in_code_block {
                    code_content.push_str(&text);
                } else if in_heading {
                    heading_text.push_str(&text);
                } else if in_link {
                    link_text.push_str(&text);
                } else {
                    // Regular prose text
                    prose_parts.push(text.to_string());
                }
            }

            Event::SoftBreak | Event::HardBreak => {
                if in_heading {
                    heading_text.push(' ');
                } else if !in_code_block && !in_link {
                    prose_parts.push(" ".to_string());
                }
            }

            _ => {}
        }
    }

    let prose_text = prose_parts
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    (headings, links, code_blocks, prose_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_markdown_headings() {
        let body = "# Title\n\n## Subtitle\n\nSome text.\n";
        let (headings, _, _, _) = extract_markdown(body);
        assert_eq!(headings.len(), 2);
        assert_eq!(headings[0].level, 1);
        assert_eq!(headings[0].text, "Title");
        assert_eq!(headings[1].level, 2);
        assert_eq!(headings[1].text, "Subtitle");
    }

    #[test]
    fn extract_markdown_links() {
        let body = "See [example](https://example.com) for details.\n";
        let (_, links, _, _) = extract_markdown(body);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, "example");
        assert_eq!(links[0].url, "https://example.com");
    }

    #[test]
    fn extract_markdown_code_blocks() {
        let body = "```rust\nfn main() {}\n```\n\nSome text.\n";
        let (_, _, code_blocks, _) = extract_markdown(body);
        assert_eq!(code_blocks.len(), 1);
        assert_eq!(code_blocks[0].language, Some("rust".to_string()));
        assert!(code_blocks[0].content.contains("fn main()"));
    }

    #[test]
    fn prose_strips_code_and_urls() {
        let body = "Hello world.\n\n```python\nprint('hi')\n```\n\nMore text `inline code` here.\n\nSee [link](http://x.com) too.\n";
        let (_, _, _, prose) = extract_markdown(body);
        // Code block content should not appear
        assert!(!prose.contains("print('hi')"));
        // Inline code should not appear
        assert!(!prose.contains("inline code"));
        // URL should not appear, but link text should
        assert!(!prose.contains("http://x.com"));
        assert!(prose.contains("link"));
        assert!(prose.contains("Hello world."));
        assert!(prose.contains("More text"));
    }
}
