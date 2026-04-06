// Human and JSON formatters for pipeline results.

use std::fmt::Write as _;
use std::path::Path;

use serde::Serialize;

use crate::models::{Diagnostic, Severity, Sizeyness};
use crate::pipeline::{exit_code, PipelineResult};

// ---------------------------------------------------------------------------
// Human output
// ---------------------------------------------------------------------------

/// Format pipeline results for human consumption.
///
/// Diagnostics are grouped by severity (Info first, then Suggestion, Warning,
/// Error) with emoji markers and an encouraging tone.  The `min_severity`
/// parameter filters out diagnostics below the given level.
pub fn format_human(result: &PipelineResult, skill_dir: &Path, min_severity: Severity) -> String {
    let mut out = String::new();

    // Header
    let skill_label = result.skill_name.as_deref().unwrap_or_else(|| {
        skill_dir
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("skill")
    });
    let sizeyness_label = match result.sizeyness {
        Sizeyness::Simple => "simple",
        Sizeyness::Moderate => "moderate",
        Sizeyness::Hefty => "hefty",
    };
    let reasons_joined = result.sizeyness_reasons.join(", ");
    if reasons_joined.is_empty() {
        let _ = writeln!(out, "\u{1f4c1} {skill_label}/ ({sizeyness_label})");
    } else {
        let _ = writeln!(
            out,
            "\u{1f4c1} {skill_label}/ ({sizeyness_label} \u{2014} {reasons_joined})"
        );
    }

    // Filter diagnostics
    let filtered: Vec<&Diagnostic> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity >= min_severity)
        .collect();

    if filtered.is_empty() {
        let _ = writeln!(out);
        let _ = writeln!(out, "\u{2713} No issues found");
        write_summary(&mut out, &filtered);
        return out;
    }

    // Group by severity in display order: Info, Suggestion, Warning, Error
    let severity_order = [
        Severity::Info,
        Severity::Suggestion,
        Severity::Warning,
        Severity::Error,
    ];

    for &sev in &severity_order {
        let group: Vec<&&Diagnostic> = filtered.iter().filter(|d| d.severity == sev).collect();
        if group.is_empty() {
            continue;
        }

        let _ = writeln!(out);
        for diag in group {
            let emoji = severity_emoji(sev);
            let _ = writeln!(out, "  {emoji} {}", diag.human_message);
            if let Some(ref url) = diag.doc_url {
                let _ = writeln!(out, "     \u{2192} {url}");
            }
        }
    }

    write_summary(&mut out, &filtered);
    out
}

fn severity_emoji(sev: Severity) -> &'static str {
    match sev {
        Severity::Info => "\u{2705}",
        Severity::Suggestion => "\u{1f4a1}",
        Severity::Warning => "\u{26a0}\u{fe0f}",
        Severity::Error => "\u{274c}",
    }
}

fn write_summary(out: &mut String, diagnostics: &[&Diagnostic]) {
    let errors = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .count();
    let warnings = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Warning)
        .count();
    let suggestions = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Suggestion)
        .count();
    let info = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Info)
        .count();

    let _ = writeln!(out);
    let _ = write!(
        out,
        "Summary: {} {}, {} {}, {} {}, {} passed {}",
        errors,
        if errors == 1 { "error" } else { "errors" },
        warnings,
        if warnings == 1 { "warning" } else { "warnings" },
        suggestions,
        if suggestions == 1 {
            "suggestion"
        } else {
            "suggestions"
        },
        info,
        if info == 1 { "check" } else { "checks" },
    );
}

// ---------------------------------------------------------------------------
// JSON output
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct JsonOutput {
    schema_version: u32,
    skill: String,
    path: String,
    sizeyness: String,
    sizeyness_reasons: Vec<String>,
    diagnostics: Vec<JsonDiagnostic>,
    summary: JsonSummary,
    exit_code: i32,
}

#[derive(Serialize)]
struct JsonDiagnostic {
    check: String,
    severity: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
}

#[derive(Serialize)]
struct JsonSummary {
    errors: usize,
    warnings: usize,
    suggestions: usize,
    info: usize,
}

/// Format pipeline results as JSON.
///
/// The `min_severity` parameter filters diagnostics below the given level.
/// The `strict` parameter affects the `exit_code` field.
pub fn format_json(
    result: &PipelineResult,
    skill_dir: &Path,
    min_severity: Severity,
    strict: bool,
) -> String {
    let skill_label = result.skill_name.clone().unwrap_or_else(|| {
        skill_dir
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("skill")
            .to_string()
    });

    let filtered: Vec<&Diagnostic> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity >= min_severity)
        .collect();

    let diagnostics: Vec<JsonDiagnostic> = filtered
        .iter()
        .map(|d| {
            let check = serde_json::to_value(d.check_name)
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_else(|| format!("{:?}", d.check_name));

            let severity = match d.severity {
                Severity::Info => "info",
                Severity::Suggestion => "suggestion",
                Severity::Warning => "warning",
                Severity::Error => "error",
            };

            let file = d.file_path.as_ref().map(|p| {
                p.strip_prefix(skill_dir)
                    .unwrap_or(p)
                    .to_string_lossy()
                    .to_string()
            });

            JsonDiagnostic {
                check,
                severity: severity.to_string(),
                message: d.machine_message.clone(),
                file,
            }
        })
        .collect();

    let summary = JsonSummary {
        errors: filtered
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count(),
        warnings: filtered
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count(),
        suggestions: filtered
            .iter()
            .filter(|d| d.severity == Severity::Suggestion)
            .count(),
        info: filtered
            .iter()
            .filter(|d| d.severity == Severity::Info)
            .count(),
    };

    let sizeyness_label = match result.sizeyness {
        Sizeyness::Simple => "simple",
        Sizeyness::Moderate => "moderate",
        Sizeyness::Hefty => "hefty",
    };

    // exit_code is computed from ALL diagnostics (unfiltered), per pipeline spec
    let code = exit_code(&result.diagnostics, strict);

    let output = JsonOutput {
        schema_version: 2,
        skill: skill_label,
        path: skill_dir.to_string_lossy().to_string(),
        sizeyness: sizeyness_label.to_string(),
        sizeyness_reasons: result.sizeyness_reasons.clone(),
        diagnostics,
        summary,
        exit_code: code,
    };

    serde_json::to_string_pretty(&output).expect("JSON serialization should not fail")
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CheckName, Severity, Sizeyness};
    use crate::pipeline::PipelineResult;
    use std::path::PathBuf;

    fn make_diagnostic(
        severity: Severity,
        check_name: CheckName,
        human_message: &str,
        machine_message: &str,
        doc_url: Option<&str>,
        file_path: Option<&str>,
    ) -> Diagnostic {
        Diagnostic {
            severity,
            check_name,
            human_message: human_message.to_string(),
            machine_message: machine_message.to_string(),
            doc_url: doc_url.map(|s| s.to_string()),
            file_path: file_path.map(PathBuf::from),
            base_severity: severity,
        }
    }

    fn sample_result() -> PipelineResult {
        PipelineResult {
            diagnostics: vec![
                make_diagnostic(
                    Severity::Info,
                    CheckName::HasGotchasSection,
                    "Skill includes a gotchas section with concrete content \u{2014} that's one of the highest-value things you can add.",
                    "Skill includes gotchas section with content",
                    None,
                    None,
                ),
                make_diagnostic(
                    Severity::Info,
                    CheckName::HasProgressiveDisclosure,
                    "Good use of progressive disclosure \u{2014} agents load detail on demand.",
                    "progressive-disclosure present",
                    None,
                    None,
                ),
                make_diagnostic(
                    Severity::Suggestion,
                    CheckName::DescriptionTriggerLanguage,
                    "Consider adding trigger language to your description so agents know when to activate this skill.",
                    "description lacks trigger language",
                    Some("https://code.claude.com/docs/en/skills#frontmatter-reference"),
                    None,
                ),
                make_diagnostic(
                    Severity::Warning,
                    CheckName::OrphanedFiles,
                    "2 files in this skill aren't referenced from any markdown file.",
                    "2 orphaned files",
                    None,
                    Some("/path/to/skill/scripts/unused.py"),
                ),
                make_diagnostic(
                    Severity::Error,
                    CheckName::BinaryDetected,
                    "Binary file detected: lib/helper.so\n     Compiled binaries in skills are a security concern.",
                    "binary detected: lib/helper.so",
                    Some("https://agentskills.io/skill-creation/best-practices"),
                    Some("/path/to/skill/lib/helper.so"),
                ),
            ],
            skill_name: Some("my-skill".to_string()),
            sizeyness: Sizeyness::Moderate,
            sizeyness_reasons: vec!["4 files".to_string(), "1 subdirectory".to_string()],
        }
    }

    // -----------------------------------------------------------------------
    // Human output tests
    // -----------------------------------------------------------------------

    #[test]
    fn human_header_includes_skill_name_sizeyness_and_reasons() {
        let result = sample_result();
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);
        assert!(out.contains("\u{1f4c1} my-skill/"));
        assert!(out.contains("moderate"));
        assert!(out.contains("4 files"));
        assert!(out.contains("1 subdirectory"));
    }

    #[test]
    fn human_output_has_emoji_markers() {
        let result = sample_result();
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);
        assert!(out.contains('\u{2705}'), "Missing info emoji");
        assert!(out.contains('\u{1f4a1}'), "Missing suggestion emoji");
        assert!(out.contains("\u{26a0}\u{fe0f}"), "Missing warning emoji");
        assert!(out.contains('\u{274c}'), "Missing error emoji");
    }

    #[test]
    fn human_output_includes_doc_urls() {
        let result = sample_result();
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);
        assert!(
            out.contains("\u{2192} https://code.claude.com/docs/en/skills#frontmatter-reference")
        );
        assert!(out.contains("\u{2192} https://agentskills.io/skill-creation/best-practices"));
    }

    #[test]
    fn human_output_groups_by_severity() {
        let result = sample_result();
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);

        // Info should appear before Suggestion, which should appear before Warning, etc.
        let info_pos = out.find('\u{2705}').expect("info emoji");
        let suggestion_pos = out.find('\u{1f4a1}').expect("suggestion emoji");
        let warning_pos = out.find("\u{26a0}\u{fe0f}").expect("warning emoji");
        let error_pos = out.find('\u{274c}').expect("error emoji");

        assert!(
            info_pos < suggestion_pos,
            "Info should come before suggestion"
        );
        assert!(
            suggestion_pos < warning_pos,
            "Suggestion should come before warning"
        );
        assert!(warning_pos < error_pos, "Warning should come before error");
    }

    #[test]
    fn human_output_has_summary_line() {
        let result = sample_result();
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);
        assert!(out.contains("Summary:"));
        assert!(out.contains("1 error"));
        assert!(out.contains("1 warning"));
        assert!(out.contains("1 suggestion"));
        assert!(out.contains("2 passed checks"));
    }

    #[test]
    fn human_severity_filter_hides_lower_tiers() {
        let result = sample_result();

        // Filter at Warning: should hide Info and Suggestion
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Warning);
        assert!(
            !out.contains('\u{2705}'),
            "Info should be filtered out at Warning level"
        );
        assert!(
            !out.contains('\u{1f4a1}'),
            "Suggestion should be filtered out at Warning level"
        );
        assert!(
            out.contains("\u{26a0}\u{fe0f}"),
            "Warning should still show"
        );
        assert!(out.contains('\u{274c}'), "Error should still show");
    }

    #[test]
    fn human_no_diagnostics_shows_no_issues() {
        let result = PipelineResult {
            diagnostics: vec![],
            skill_name: Some("clean-skill".to_string()),
            sizeyness: Sizeyness::Simple,
            sizeyness_reasons: vec!["1 file".to_string()],
        };
        let out = format_human(&result, Path::new("/path/to/skill"), Severity::Info);
        assert!(out.contains("\u{2713} No issues found"));
    }

    #[test]
    fn human_falls_back_to_dir_name_when_no_skill_name() {
        let result = PipelineResult {
            diagnostics: vec![],
            skill_name: None,
            sizeyness: Sizeyness::Simple,
            sizeyness_reasons: vec![],
        };
        let out = format_human(&result, Path::new("/path/to/my-dir"), Severity::Info);
        assert!(out.contains("my-dir/"));
    }

    // -----------------------------------------------------------------------
    // JSON output tests
    // -----------------------------------------------------------------------

    #[test]
    fn json_has_schema_version_2() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["schema_version"], 2);
    }

    #[test]
    fn json_has_sizeyness_reasons() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let reasons = v["sizeyness_reasons"].as_array().unwrap();
        assert_eq!(reasons.len(), 2);
        assert_eq!(reasons[0], "4 files");
        assert_eq!(reasons[1], "1 subdirectory");
    }

    #[test]
    fn json_uses_machine_message() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let diags = v["diagnostics"].as_array().unwrap();

        // First diagnostic (info) should use machine_message
        assert_eq!(
            diags[0]["message"],
            "Skill includes gotchas section with content"
        );
        // Should NOT contain human_message text
        let all_messages: Vec<_> = diags
            .iter()
            .map(|d| d["message"].as_str().unwrap())
            .collect();
        assert!(
            !all_messages.iter().any(|m| m.contains("that's one of")),
            "JSON should use machine_message, not human_message"
        );
    }

    #[test]
    fn json_severity_filter_hides_lower_tiers() {
        let result = sample_result();
        let json_str = format_json(
            &result,
            Path::new("/path/to/skill"),
            Severity::Warning,
            false,
        );
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let diags = v["diagnostics"].as_array().unwrap();

        // Should only have warning + error = 2
        assert_eq!(diags.len(), 2);
        for d in diags {
            let sev = d["severity"].as_str().unwrap();
            assert!(
                sev == "warning" || sev == "error",
                "Unexpected severity in filtered JSON: {sev}"
            );
        }
    }

    #[test]
    fn json_exit_code_reflects_errors() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["exit_code"], 1, "Should be 1 because there are errors");
    }

    #[test]
    fn json_exit_code_strict_mode() {
        // Result with only suggestions, no errors
        let result = PipelineResult {
            diagnostics: vec![make_diagnostic(
                Severity::Suggestion,
                CheckName::HasExamples,
                "Consider adding examples",
                "no examples found",
                None,
                None,
            )],
            skill_name: Some("my-skill".to_string()),
            sizeyness: Sizeyness::Simple,
            sizeyness_reasons: vec![],
        };

        let non_strict = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&non_strict).unwrap();
        assert_eq!(v["exit_code"], 0, "Non-strict with only suggestions = 0");

        let strict = format_json(&result, Path::new("/path/to/skill"), Severity::Info, true);
        let v: serde_json::Value = serde_json::from_str(&strict).unwrap();
        assert_eq!(v["exit_code"], 1, "Strict with suggestions = 1");
    }

    #[test]
    fn json_check_name_is_kebab_case() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let diags = v["diagnostics"].as_array().unwrap();
        assert_eq!(diags[0]["check"], "has-gotchas-section");
        assert_eq!(diags[4]["check"], "binary-detected");
    }

    #[test]
    fn json_file_path_relative_to_skill_dir() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        let diags = v["diagnostics"].as_array().unwrap();

        // The error diagnostic has file_path = /path/to/skill/lib/helper.so
        let error_diag = diags
            .iter()
            .find(|d| d["check"] == "binary-detected")
            .unwrap();
        assert_eq!(error_diag["file"], "lib/helper.so");
    }

    #[test]
    fn json_summary_counts() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(v["summary"]["errors"], 1);
        assert_eq!(v["summary"]["warnings"], 1);
        assert_eq!(v["summary"]["suggestions"], 1);
        assert_eq!(v["summary"]["info"], 2);
    }

    #[test]
    fn json_is_valid_parseable() {
        let result = sample_result();
        let json_str = format_json(&result, Path::new("/path/to/skill"), Severity::Info, false);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok(), "JSON output should be valid JSON");
    }
}
