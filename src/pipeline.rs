// Pipeline orchestration — runs all five passes in sequence, collecting diagnostics.

use std::path::Path;

use crate::config::ValidatorConfig;
use crate::models::{CheckName, Diagnostic, PipelineError, Severity, Sizeyness, SkillContext};
use crate::passes::{content, parse, references, security, structure};

// ---------------------------------------------------------------------------
// PipelineResult
// ---------------------------------------------------------------------------

/// The result of running the full validation pipeline.
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub diagnostics: Vec<Diagnostic>,
    pub skill_name: Option<String>,
    pub sizeyness: Sizeyness,
    pub sizeyness_reasons: Vec<String>,
}

// ---------------------------------------------------------------------------
// Pipeline entry point
// ---------------------------------------------------------------------------

/// Run the full validation pipeline against `skill_dir`.
///
/// 1. Pass 1 (Parse) — fatal on error, return immediately.
/// 2. Passes 2-5 — errors are converted to diagnostics; each pass runs
///    independently even if a prior pass fails.
pub fn run_pipeline(skill_dir: &Path, config: &ValidatorConfig) -> PipelineResult {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // ── Pass 1: Parse (fatal on error) ─────────────────────────────────
    let (mut ctx, parse_diags) = match parse::run(skill_dir) {
        Ok(result) => result,
        Err(e) => {
            diagnostics.push(pipeline_error_diagnostic(&e));
            return PipelineResult {
                diagnostics,
                skill_name: None,
                sizeyness: Sizeyness::Simple,
                sizeyness_reasons: Vec::new(),
            };
        }
    };
    diagnostics.extend(parse_diags);

    // Extract skill name from frontmatter before running further passes.
    let skill_name = extract_skill_name(&ctx);

    // ── Pass 2: Structure ──────────────────────────────────────────────
    match structure::run(skill_dir, &mut ctx, config) {
        Ok(diags) => diagnostics.extend(diags),
        Err(e) => diagnostics.push(pipeline_error_diagnostic(&e)),
    }

    // Capture sizeyness info after structure pass.
    let sizeyness = ctx.sizeyness;
    let sizeyness_reasons = build_sizeyness_reasons(&ctx);

    // ── Pass 3: Content ────────────────────────────────────────────────
    match content::run(skill_dir, &ctx, config) {
        Ok(diags) => diagnostics.extend(diags),
        Err(e) => diagnostics.push(pipeline_error_diagnostic(&e)),
    }

    // ── Pass 4: References ─────────────────────────────────────────────
    match references::run(skill_dir, &mut ctx, config) {
        Ok(diags) => diagnostics.extend(diags),
        Err(e) => diagnostics.push(pipeline_error_diagnostic(&e)),
    }

    // ── Pass 5: Security ───────────────────────────────────────────────
    match security::run(skill_dir, &ctx, config) {
        Ok(diags) => diagnostics.extend(diags),
        Err(e) => diagnostics.push(pipeline_error_diagnostic(&e)),
    }

    PipelineResult {
        diagnostics,
        skill_name,
        sizeyness,
        sizeyness_reasons,
    }
}

// ---------------------------------------------------------------------------
// Exit code
// ---------------------------------------------------------------------------

/// Compute exit code from diagnostics and strict mode.
///
/// - Returns 1 if any Error-severity diagnostics exist.
/// - In strict mode, also returns 1 for Warning or Suggestion diagnostics.
/// - Otherwise returns 0.
pub fn exit_code(diagnostics: &[Diagnostic], strict: bool) -> i32 {
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return 1;
    }
    if strict
        && diagnostics
            .iter()
            .any(|d| matches!(d.severity, Severity::Warning | Severity::Suggestion))
    {
        return 1;
    }
    0
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a `PipelineError` into a system-level diagnostic.
fn pipeline_error_diagnostic(err: &PipelineError) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        check_name: CheckName::PipelineError,
        human_message: format!("Pipeline error: {err}"),
        machine_message: format!("pipeline-error:{err}"),
        doc_url: None,
        file_path: match err {
            PipelineError::ParseFailed { path, .. } | PipelineError::IoError { path, .. } => {
                Some(path.clone())
            }
            _ => None,
        },
        base_severity: Severity::Error,
    }
}

/// Extract the `name` field from parsed frontmatter, if present.
fn extract_skill_name(ctx: &SkillContext) -> Option<String> {
    ctx.frontmatter
        .as_mapping()
        .and_then(|m| m.get(serde_yaml::Value::String("name".to_string())))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

/// Build human-readable sizeyness reasons from context.
fn build_sizeyness_reasons(ctx: &SkillContext) -> Vec<String> {
    let mut reasons = Vec::new();

    let file_count = ctx.file_inventory.len();
    if file_count > 0 {
        reasons.push(format!(
            "{} {}",
            file_count,
            if file_count == 1 { "file" } else { "files" }
        ));
    }

    let subdir_count = ctx.subdirectories.len();
    if subdir_count > 0 {
        reasons.push(format!(
            "{} {}",
            subdir_count,
            if subdir_count == 1 {
                "subdirectory"
            } else {
                "subdirectories"
            }
        ));
    }

    // Check for orchestration fields in frontmatter.
    if let Some(mapping) = ctx.frontmatter.as_mapping() {
        let orchestration_keys = ["hooks", "agent", "context"];
        for key in &orchestration_keys {
            if mapping.contains_key(serde_yaml::Value::String((*key).to_string())) {
                reasons.push("has orchestration fields".to_string());
                break;
            }
        }
    }

    reasons
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_no_diagnostics() {
        assert_eq!(exit_code(&[], false), 0);
        assert_eq!(exit_code(&[], true), 0);
    }

    #[test]
    fn exit_code_info_only() {
        let diags = vec![Diagnostic {
            severity: Severity::Info,
            check_name: CheckName::SizeynessInfo,
            human_message: "info".to_string(),
            machine_message: "info".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Info,
        }];
        assert_eq!(exit_code(&diags, false), 0);
        assert_eq!(exit_code(&diags, true), 0);
    }

    #[test]
    fn exit_code_error_always_fails() {
        let diags = vec![Diagnostic {
            severity: Severity::Error,
            check_name: CheckName::PipelineError,
            human_message: "err".to_string(),
            machine_message: "err".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Error,
        }];
        assert_eq!(exit_code(&diags, false), 1);
        assert_eq!(exit_code(&diags, true), 1);
    }

    #[test]
    fn exit_code_warning_strict() {
        let diags = vec![Diagnostic {
            severity: Severity::Warning,
            check_name: CheckName::BinaryDetected,
            human_message: "warn".to_string(),
            machine_message: "warn".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Warning,
        }];
        assert_eq!(exit_code(&diags, false), 0);
        assert_eq!(exit_code(&diags, true), 1);
    }

    #[test]
    fn exit_code_suggestion_strict() {
        let diags = vec![Diagnostic {
            severity: Severity::Suggestion,
            check_name: CheckName::HasExamples,
            human_message: "sug".to_string(),
            machine_message: "sug".to_string(),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Suggestion,
        }];
        assert_eq!(exit_code(&diags, false), 0);
        assert_eq!(exit_code(&diags, true), 1);
    }

    #[test]
    fn pipeline_error_diagnostic_includes_path() {
        let err = PipelineError::ParseFailed {
            path: std::path::PathBuf::from("/some/path"),
            reason: "no SKILL.md".to_string(),
        };
        let diag = pipeline_error_diagnostic(&err);
        assert_eq!(diag.severity, Severity::Error);
        assert_eq!(diag.check_name, CheckName::PipelineError);
        assert!(diag.file_path.is_some());
    }

    #[test]
    fn pipeline_error_diagnostic_no_path_for_config() {
        let err = PipelineError::ConfigInvalid {
            reason: "bad config".to_string(),
        };
        let diag = pipeline_error_diagnostic(&err);
        assert!(diag.file_path.is_none());
    }
}
