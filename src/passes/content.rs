// Pass 3: Content — frontmatter field checks, quality, positive reinforcement.
#![allow(dead_code)]

use std::path::Path;

use regex::Regex;
use unicode_normalization::UnicodeNormalization;

use crate::config::ValidatorConfig;
use crate::models::{CheckName, Diagnostic, PipelineError, Severity, Sizeyness, SkillContext};

// ── Constants ──────────────────────────────────────────────────────────────────

const MAX_SKILL_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 250;

/// Spec-defined fields.
const SPEC_FIELDS: &[&str] = &[
    "name",
    "description",
    "license",
    "allowed-tools",
    "metadata",
    "compatibility",
];

/// Claude Code extension fields.
const EXTENSION_FIELDS: &[&str] = &[
    "argument-hint",
    "disable-model-invocation",
    "user-invocable",
    "model",
    "context",
    "agent",
    "hooks",
];

// ── Public entry point ─────────────────────────────────────────────────────────

/// Run Pass 3 (Content) against a parsed skill context.
///
/// Checks frontmatter fields, content quality, and emits positive reinforcement.
/// The `skill_dir` is used for the name-directory-match check.
pub fn run(
    skill_dir: &Path,
    ctx: &SkillContext,
    config: &ValidatorConfig,
) -> Result<Vec<Diagnostic>, PipelineError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Only run checks if frontmatter is a mapping
    if let Some(map) = ctx.frontmatter.as_mapping() {
        check_frontmatter(skill_dir, map, ctx, config, &mut diagnostics);
    }

    check_content_quality(ctx, config, &mut diagnostics);
    check_positive_reinforcement(ctx, &mut diagnostics);

    Ok(diagnostics)
}

// ── Frontmatter checks ────────────────────────────────────────────────────────

fn check_frontmatter(
    skill_dir: &Path,
    map: &serde_yaml::Mapping,
    ctx: &SkillContext,
    config: &ValidatorConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_name(skill_dir, map, diagnostics);
    check_description(map, ctx, diagnostics);
    check_fields(map, diagnostics);
    check_extension_semantics(map, config, diagnostics);
}

// ── Name checks ────────────────────────────────────────────────────────────────

fn check_name(skill_dir: &Path, map: &serde_yaml::Mapping, diagnostics: &mut Vec<Diagnostic>) {
    let name_val = map.get(serde_yaml::Value::String("name".to_string()));

    let name_str = match name_val.and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::NameMissing,
                human_message: "Frontmatter must include a `name` field.".to_string(),
                machine_message: "name:missing".to_string(),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Error,
            });
            return;
        }
    };

    let normalized: String = name_str.nfkc().collect();
    let name = normalized.trim();

    // Format: lowercase, hyphens, 1-64 chars
    let valid_format = !name.is_empty()
        && name.len() <= MAX_SKILL_NAME_LENGTH
        && name == name.to_lowercase()
        && name.chars().all(|c| c.is_alphanumeric() || c == '-')
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--");

    if !valid_format {
        let mut reasons = Vec::new();
        if name.is_empty() || name.len() > MAX_SKILL_NAME_LENGTH {
            reasons.push(format!("must be 1-{MAX_SKILL_NAME_LENGTH} characters"));
        }
        if name != name.to_lowercase() {
            reasons.push("must be lowercase".to_string());
        }
        if !name.chars().all(|c| c.is_alphanumeric() || c == '-') {
            reasons.push("only letters, digits, and hyphens allowed".to_string());
        }
        if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
            reasons.push("no leading/trailing/consecutive hyphens".to_string());
        }

        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            check_name: CheckName::NameFormat,
            human_message: format!(
                "Skill name `{name}` has invalid format: {}.",
                reasons.join("; ")
            ),
            machine_message: format!("name-format:{name}"),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Error,
        });
    }

    // Directory match
    let dir_name: String = skill_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .nfkc()
        .collect();

    if dir_name != name {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            check_name: CheckName::NameDirectoryMatch,
            human_message: format!(
                "Skill name `{name}` does not match directory name `{dir_name}`."
            ),
            machine_message: format!("name-dir-mismatch:{name}:{dir_name}"),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Error,
        });
    }
}

// ── Description checks ─────────────────────────────────────────────────────────

fn check_description(
    map: &serde_yaml::Mapping,
    ctx: &SkillContext,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let desc_val = map.get(serde_yaml::Value::String("description".to_string()));

    let desc = match desc_val.and_then(|v| v.as_str()) {
        Some(s) if !s.trim().is_empty() => s,
        _ => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::DescriptionMissing,
                human_message: "Frontmatter must include a `description` field.".to_string(),
                machine_message: "description:missing".to_string(),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Error,
            });
            return;
        }
    };

    // Length check
    if desc.len() > MAX_DESCRIPTION_LENGTH {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            check_name: CheckName::DescriptionLength,
            human_message: format!(
                "Description is {} characters — exceeds the {MAX_DESCRIPTION_LENGTH}-character limit. \
                 Claude Code truncates long descriptions in tool listings. \
                 See: https://docs.anthropic.com/s/claude-code-skills",
                desc.len()
            ),
            machine_message: format!("description-length:{}:{MAX_DESCRIPTION_LENGTH}", desc.len()),
            doc_url: Some(
                "https://docs.anthropic.com/s/claude-code-skills".to_string(),
            ),
            file_path: None,
            base_severity: Severity::Error,
        });
    }

    // Trigger language in description
    let trigger_re = Regex::new(r"(?i)\b(use when|trigger when|activate when)\b").unwrap();
    if !trigger_re.is_match(desc) {
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate => Severity::Warning,
            Sizeyness::Hefty => Severity::Error,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::DescriptionTriggerLanguage,
            human_message:
                "Description should contain trigger language like \"use when\", \
                 \"trigger when\", or \"activate when\" so the model knows when to invoke this skill."
                    .to_string(),
            machine_message: "description-trigger-language:missing".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

// ── Field checks ───────────────────────────────────────────────────────────────

fn check_fields(map: &serde_yaml::Mapping, diagnostics: &mut Vec<Diagnostic>) {
    for (key, _) in map {
        let Some(key_str) = key.as_str() else {
            continue;
        };

        if SPEC_FIELDS.contains(&key_str) {
            continue;
        }

        if EXTENSION_FIELDS.contains(&key_str) {
            diagnostics.push(Diagnostic {
                severity: Severity::Suggestion,
                check_name: CheckName::ExtensionFieldCompatibility,
                human_message: format!(
                    "Field `{key_str}` is recognized by Claude Code but may not be used by other tools."
                ),
                machine_message: format!("extension-field:{key_str}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Suggestion,
            });
            continue;
        }

        // Unknown field
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            check_name: CheckName::UnknownField,
            human_message: format!(
                "Unknown frontmatter field `{key_str}`. \
                 Spec fields: {}. Claude Code extensions: {}.",
                SPEC_FIELDS.join(", "),
                EXTENSION_FIELDS.join(", ")
            ),
            machine_message: format!("unknown-field:{key_str}"),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Warning,
        });
    }
}

// ── Extension semantic checks ──────────────────────────────────────────────────

fn check_extension_semantics(
    map: &serde_yaml::Mapping,
    config: &ValidatorConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let get_str = |key: &str| -> Option<&str> {
        map.get(serde_yaml::Value::String(key.to_string()))
            .and_then(|v| v.as_str())
    };

    let has_key =
        |key: &str| -> bool { map.contains_key(serde_yaml::Value::String(key.to_string())) };

    // context must be "fork" if present
    if let Some(context_val) = get_str("context") {
        if context_val != "fork" {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::ContextValidValue,
                human_message: format!(
                    "Field `context` has value `{context_val}` but must be `fork` if set."
                ),
                machine_message: format!("context-invalid:{context_val}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Error,
            });
        }
    }

    // agent without context: fork
    if has_key("agent") {
        let context_is_fork = get_str("context").is_some_and(|v| v == "fork");
        if !context_is_fork {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::AgentWithContext,
                human_message: "Field `agent` has no effect without `context: fork`. \
                     Add `context: fork` to enable agent delegation."
                    .to_string(),
                machine_message: "agent-without-context-fork".to_string(),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
        }
    }

    // model recognition
    if let Some(model_val) = get_str("model") {
        if !config.content.known_models.iter().any(|m| m == model_val) {
            diagnostics.push(Diagnostic {
                severity: Severity::Suggestion,
                check_name: CheckName::ModelRecognized,
                human_message: format!(
                    "Model `{model_val}` is not in the known models list ({}). \
                     This may be intentional if you're targeting a newer model.",
                    config.content.known_models.join(", ")
                ),
                machine_message: format!("model-unrecognized:{model_val}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Suggestion,
            });
        }
    }
}

// ── Content quality checks ─────────────────────────────────────────────────────

fn check_content_quality(
    ctx: &SkillContext,
    config: &ValidatorConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    check_trigger_conditions(ctx, diagnostics);
    check_examples(ctx, diagnostics);
    check_behavioral_constraints(ctx, diagnostics);
    check_gotchas(ctx, diagnostics);
    check_body_length(ctx, config, diagnostics);
    check_windows_paths(ctx, diagnostics);
}

fn check_trigger_conditions(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let trigger_re = Regex::new(r"(?i)\b(use when|trigger when|activate when)\b").unwrap();
    let heading_re = Regex::new(r"(?i)when to use").unwrap();

    let has_trigger = trigger_re.is_match(&ctx.prose_text)
        || ctx.headings.iter().any(|h| heading_re.is_match(&h.text));

    if !has_trigger {
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate => Severity::Warning,
            Sizeyness::Hefty => Severity::Error,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::HasTriggerConditions,
            human_message: "No trigger conditions found. Add \"use when\", \"trigger when\", \
                 \"activate when\" in the body, or a heading containing \"When to Use\"."
                .to_string(),
            machine_message: "no-trigger-conditions".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

fn check_examples(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let has_code_blocks = !ctx.code_blocks.is_empty();
    let has_example_heading = ctx
        .headings
        .iter()
        .any(|h| h.text.to_lowercase().contains("example"));

    if !has_code_blocks && !has_example_heading {
        // Simple→suggestion, Moderate→warning, Hefty→warning (caps at warning)
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate | Sizeyness::Hefty => Severity::Warning,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::HasExamples,
            human_message:
                "No examples found. Add fenced code blocks or a heading containing \"Example\"."
                    .to_string(),
            machine_message: "no-examples".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

fn check_behavioral_constraints(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let never_re = Regex::new(r"(?i)\bnever\b").unwrap();
    let always_re = Regex::new(r"(?i)\balways\b").unwrap();

    let has_never = never_re.is_match(&ctx.prose_text);
    let has_always = always_re.is_match(&ctx.prose_text);

    if !has_never && !has_always {
        // Simple→suggestion, Moderate→warning, Hefty→warning
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate | Sizeyness::Hefty => Severity::Warning,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::HasBehavioralConstraints,
            human_message:
                "No behavioral constraints found. Consider adding \"never\" or \"always\" \
                 statements to set clear boundaries."
                    .to_string(),
            machine_message: "no-behavioral-constraints".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

fn check_gotchas(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let has_gotcha_heading = ctx.headings.iter().any(|h| is_gotcha_heading(&h.text));

    if !has_gotcha_heading {
        // Simple→suggestion, Moderate→suggestion, Hefty→warning
        let severity = match ctx.sizeyness {
            Sizeyness::Simple | Sizeyness::Moderate => Severity::Suggestion,
            Sizeyness::Hefty => Severity::Warning,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::HasGotchas,
            human_message: "No gotchas section found. Consider adding a heading with \
                 \"Gotchas\", \"Caveats\", \"Pitfalls\", or \"Common Mistakes\"."
                .to_string(),
            machine_message: "no-gotchas".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

fn check_body_length(
    ctx: &SkillContext,
    config: &ValidatorConfig,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let line_count = ctx.prose_text.lines().count();
    if line_count > config.content.body_line_limit {
        let severity = match ctx.sizeyness {
            Sizeyness::Simple => Severity::Suggestion,
            Sizeyness::Moderate => Severity::Warning,
            Sizeyness::Hefty => Severity::Error,
        };
        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::BodyLength,
            human_message: format!(
                "Body is {line_count} lines, exceeding the {}-line limit. \
                 Consider splitting into referenced files.",
                config.content.body_line_limit
            ),
            machine_message: format!(
                "body-length:{}:{}",
                line_count, config.content.body_line_limit
            ),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

fn check_windows_paths(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    // Match common Windows path patterns in prose text (not code blocks)
    let win_path_re = Regex::new(r"[A-Z]:\\[\w\\]+").unwrap();
    if win_path_re.is_match(&ctx.prose_text) {
        diagnostics.push(Diagnostic {
            severity: Severity::Suggestion,
            check_name: CheckName::WindowsPaths,
            human_message: "Windows-style paths detected in prose. Consider using POSIX paths \
                 or platform-agnostic references."
                .to_string(),
            machine_message: "windows-paths-detected".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        });
    }
}

// ── Positive reinforcement ─────────────────────────────────────────────────────

fn check_positive_reinforcement(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    check_gotchas_section(ctx, diagnostics);
    check_validation_loop(ctx, diagnostics);
    check_progressive_disclosure(ctx, diagnostics);
    check_concrete_examples(ctx, diagnostics);
}

fn check_gotchas_section(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    // Find a gotcha heading and check for content after it
    let gotcha_idx = ctx.headings.iter().position(|h| is_gotcha_heading(&h.text));

    if let Some(idx) = gotcha_idx {
        let gotcha_level = ctx.headings[idx].level;
        // Check if there's content after this heading by looking at prose_text
        // for list markers or paragraph content. Also check that the next heading
        // of same-or-higher level isn't immediately following (meaning there's content between).
        let has_content = if idx + 1 < ctx.headings.len() {
            // There are more headings — assume there's content between
            // (approximation: if the next heading is at deeper level, content exists)
            ctx.headings[idx + 1].level > gotcha_level || has_list_markers(&ctx.prose_text)
        } else {
            // Last heading — check for any content indicators
            has_list_markers(&ctx.prose_text) || ctx.prose_text.lines().count() > ctx.headings.len()
        };

        if has_content {
            diagnostics.push(Diagnostic {
                severity: Severity::Info,
                check_name: CheckName::HasGotchasSection,
                human_message:
                    "Nice! Gotchas section with content helps users avoid common mistakes."
                        .to_string(),
                machine_message: "has-gotchas-section".to_string(),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Info,
            });
        }
    }
}

fn check_validation_loop(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let has_checklist = ctx.prose_text.contains("- [ ]") || ctx.prose_text.contains("- [x]");

    let validate_re = Regex::new(r"(?i)\bvalidat").unwrap();
    let run_re = Regex::new(r"(?i)\brun\b").unwrap();
    let has_validate_run =
        validate_re.is_match(&ctx.prose_text) && run_re.is_match(&ctx.prose_text);

    if has_checklist || has_validate_run {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            check_name: CheckName::HasValidationLoop,
            human_message:
                "Good practice! Validation steps help ensure the skill's output is correct."
                    .to_string(),
            machine_message: "has-validation-loop".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Info,
        });
    }
}

fn check_progressive_disclosure(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    if ctx.subdirectories.is_empty() {
        return;
    }

    // Check if any link points to a file in a subdirectory
    let has_subdir_link = ctx.links.iter().any(|link| {
        let url = &link.url;
        // Relative link containing a slash (i.e., pointing into a subdirectory)
        !url.starts_with("http://")
            && !url.starts_with("https://")
            && !url.starts_with('#')
            && url.contains('/')
    });

    if has_subdir_link {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            check_name: CheckName::HasProgressiveDisclosure,
            human_message:
                "Well structured! SKILL.md references files in subdirectories for progressive disclosure."
                    .to_string(),
            machine_message: "has-progressive-disclosure".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Info,
        });
    }
}

fn check_concrete_examples(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let has_example_heading = ctx
        .headings
        .iter()
        .any(|h| h.text.to_lowercase().contains("example"));

    let has_code_blocks_with_content = ctx
        .code_blocks
        .iter()
        .any(|cb| !cb.content.trim().is_empty());

    if has_example_heading && has_code_blocks_with_content {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            check_name: CheckName::HasConcreteExamples,
            human_message: "Excellent! Concrete examples with code blocks near example headings \
                 make the skill easier to understand."
                .to_string(),
            machine_message: "has-concrete-examples".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Info,
        });
    }
}

// ── Utility helpers ────────────────────────────────────────────────────────────

fn is_gotcha_heading(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("gotcha")
        || lower.contains("caveat")
        || lower.contains("pitfall")
        || lower.contains("common mistake")
}

fn has_list_markers(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("+ ")
            || (trimmed.len() > 2
                && trimmed.as_bytes()[0].is_ascii_digit()
                && trimmed.contains(". "))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_gotcha_heading_matches() {
        assert!(is_gotcha_heading("Gotchas"));
        assert!(is_gotcha_heading("Common Gotchas"));
        assert!(is_gotcha_heading("Caveats"));
        assert!(is_gotcha_heading("Known Pitfalls"));
        assert!(is_gotcha_heading("Common Mistakes to Avoid"));
        assert!(!is_gotcha_heading("Examples"));
    }

    #[test]
    fn has_list_markers_detects() {
        assert!(has_list_markers("- item one\n- item two"));
        assert!(has_list_markers("* bullet"));
        assert!(has_list_markers("1. numbered"));
        assert!(!has_list_markers("just prose text"));
    }

    #[test]
    fn word_boundary_never_does_not_match_whenever() {
        let re = Regex::new(r"(?i)\bnever\b").unwrap();
        assert!(re.is_match("never do this"));
        assert!(!re.is_match("whenever you want"));
    }

    #[test]
    fn trigger_language_regex_matches() {
        let re = Regex::new(r"(?i)\b(use when|trigger when|activate when)\b").unwrap();
        assert!(re.is_match("Use when deploying"));
        assert!(re.is_match("trigger when the user asks"));
        assert!(re.is_match("Activate when needed"));
        assert!(!re.is_match("misuse whenever"));
    }
}
