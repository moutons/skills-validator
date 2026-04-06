// Pass 5: Security — semgrep integration, remote execution detection.

use std::path::Path;

use regex::Regex;

use crate::config::ValidatorConfig;
use crate::models::SkillContext;
use crate::models::{CheckName, Diagnostic, FileEntry, FileType, PipelineError, Severity};

// ── Bundled semgrep rules (embedded at compile time) ──────────────────────

const _RULE_SHELL_INJECTION: &str = include_str!("../../rules/shell-injection.yaml");
const _RULE_PYTHON_EXEC: &str = include_str!("../../rules/python-exec.yaml");
const _RULE_ENV_EXFILTRATION: &str = include_str!("../../rules/env-exfiltration.yaml");
const _RULE_HARDCODED_URLS: &str = include_str!("../../rules/hardcoded-urls.yaml");
const _RULE_FILESYSTEM_ESCAPE: &str = include_str!("../../rules/filesystem-escape.yaml");

const BUNDLED_RULES: &[(&str, &str)] = &[
    ("shell-injection.yaml", _RULE_SHELL_INJECTION),
    ("python-exec.yaml", _RULE_PYTHON_EXEC),
    ("env-exfiltration.yaml", _RULE_ENV_EXFILTRATION),
    ("hardcoded-urls.yaml", _RULE_HARDCODED_URLS),
    ("filesystem-escape.yaml", _RULE_FILESYSTEM_ESCAPE),
];

// ── Remote execution patterns ─────────────────────────────────────────────

/// Patterns that indicate piping remote content into a shell.
const REMOTE_EXEC_PATTERNS: &[&str] = &[
    r"curl\s+[^|]*\|\s*bash",
    r"curl\s+[^|]*\|\s*sh",
    r"wget\s+[^|]*\|\s*bash",
    r"wget\s+[^|]*\|\s*sh",
    r"bash\s+<\(\s*curl",
    r"sh\s+<\(\s*curl",
];

// ── Public entry point ────────────────────────────────────────────────────

/// Run Pass 5 (Security) against a parsed skill context.
///
/// Checks for remote execution patterns in prose and code blocks, and
/// optionally runs semgrep if available.
pub fn run(
    skill_dir: &Path,
    ctx: &SkillContext,
    config: &ValidatorConfig,
) -> Result<Vec<Diagnostic>, PipelineError> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // Always: scan for remote execution patterns
    check_remote_execution_patterns(ctx, &mut diagnostics);

    // Collect script files from inventory
    let scripts: Vec<&FileEntry> = ctx
        .file_inventory
        .iter()
        .filter(|f| f.file_type == FileType::Script)
        .collect();

    // Determine whether to use semgrep
    let semgrep_available =
        config.security.semgrep_enabled && which_semgrep(&config.security.semgrep_path).is_some();

    if !scripts.is_empty() {
        if semgrep_available {
            run_semgrep(skill_dir, ctx, config, &scripts, &mut diagnostics);
        } else {
            emit_no_semgrep_diagnostics(&scripts, &mut diagnostics);
        }
    }

    Ok(diagnostics)
}

// ── Remote execution pattern scanning ─────────────────────────────────────

fn check_remote_execution_patterns(ctx: &SkillContext, diagnostics: &mut Vec<Diagnostic>) {
    let patterns: Vec<Regex> = REMOTE_EXEC_PATTERNS
        .iter()
        .map(|p| Regex::new(p).unwrap())
        .collect();

    // Scan prose text
    scan_text_for_remote_exec(&ctx.prose_text, &patterns, diagnostics);

    // Scan code block contents
    for block in &ctx.code_blocks {
        scan_text_for_remote_exec(&block.content, &patterns, diagnostics);
    }
}

fn scan_text_for_remote_exec(text: &str, patterns: &[Regex], diagnostics: &mut Vec<Diagnostic>) {
    for pattern in patterns {
        for mat in pattern.find_iter(text) {
            let matched = mat.as_str();
            // Truncate long matches for display
            let display = if matched.len() > 60 {
                format!("{}...", &matched[..57])
            } else {
                matched.to_string()
            };
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::RemoteExecutionPattern,
                human_message: format!("Skill may direct execution of remote code (`{display}`)."),
                machine_message: format!("remote-exec:{display}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
        }
    }
}

// ── Semgrep integration ───────────────────────────────────────────────────

/// Check if semgrep binary exists at the given path.
fn which_semgrep(path: &str) -> Option<std::path::PathBuf> {
    // If it's an absolute path, check directly
    if Path::new(path).is_absolute() {
        if Path::new(path).exists() {
            return Some(PathBuf::from(path));
        }
        return None;
    }

    // Otherwise, search PATH
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|dir| dir.join(path))
            .find(|p| p.exists())
    })
}

fn run_semgrep(
    skill_dir: &Path,
    ctx: &SkillContext,
    config: &ValidatorConfig,
    scripts: &[&FileEntry],
    diagnostics: &mut Vec<Diagnostic>,
) {
    use std::io::Write;

    // Write bundled rules to a temp directory
    let rules_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::SemgrepExecutionFailed,
                human_message: format!("Could not create temp dir for semgrep rules: {e}"),
                machine_message: format!("semgrep-tempdir-failed:{e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            emit_no_semgrep_diagnostics(scripts, diagnostics);
            return;
        }
    };

    for (name, content) in BUNDLED_RULES {
        let rule_path = rules_dir.path().join(name);
        if let Err(e) = std::fs::write(&rule_path, content) {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::SemgrepExecutionFailed,
                human_message: format!("Could not write semgrep rule {name}: {e}"),
                machine_message: format!("semgrep-rule-write-failed:{name}:{e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            return;
        }
    }

    // Write code blocks to temp files
    let temp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::SemgrepExecutionFailed,
                human_message: format!("Could not create temp dir for code blocks: {e}"),
                machine_message: format!("semgrep-code-tempdir-failed:{e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            return;
        }
    };

    let mut file_paths: Vec<std::path::PathBuf> = Vec::new();

    // Add actual script files
    for script in scripts {
        let full_path = skill_dir.join(&script.path);
        if full_path.exists() {
            file_paths.push(full_path);
        }
    }

    // Extract code blocks with known languages to temp files
    for (i, block) in ctx.code_blocks.iter().enumerate() {
        if let Some(ref lang) = block.language {
            if let Some(ext) = lang_to_extension(lang) {
                let temp_path = temp_dir.path().join(format!("codeblock_{i}{ext}"));
                match std::fs::File::create(&temp_path) {
                    Ok(mut f) => {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let _ = f.set_permissions(std::fs::Permissions::from_mode(0o600));
                        }
                        if let Err(e) = f.write_all(block.content.as_bytes()) {
                            diagnostics.push(Diagnostic {
                                severity: Severity::Warning,
                                check_name: CheckName::SemgrepExecutionFailed,
                                human_message: format!(
                                    "Could not write code block {i} to temp file: {e}"
                                ),
                                machine_message: format!("semgrep-write-failed:block_{i}:{e}"),
                                doc_url: None,
                                file_path: None,
                                base_severity: Severity::Warning,
                            });
                            continue;
                        }
                        file_paths.push(temp_path);
                    }
                    Err(e) => {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Warning,
                            check_name: CheckName::SemgrepExecutionFailed,
                            human_message: format!(
                                "Could not create temp file for code block {i}: {e}"
                            ),
                            machine_message: format!("semgrep-create-failed:block_{i}:{e}"),
                            doc_url: None,
                            file_path: None,
                            base_severity: Severity::Warning,
                        });
                        continue;
                    }
                }
            }
        }
    }

    if file_paths.is_empty() {
        return;
    }

    // Build semgrep command
    let semgrep_path = &config.security.semgrep_path;
    let mut cmd = std::process::Command::new(semgrep_path);
    cmd.arg("--json");
    cmd.arg("--config").arg(rules_dir.path());

    // Add custom rules dir if configured
    if !config.security.custom_rules_dir.is_empty() {
        let custom_dir = Path::new(&config.security.custom_rules_dir);
        if custom_dir.exists() {
            cmd.arg("--config").arg(custom_dir);
        }
    }

    for path in &file_paths {
        cmd.arg(path);
    }

    // Execute
    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::SemgrepExecutionFailed,
                human_message: format!("Semgrep execution failed: {e}. Security analysis skipped."),
                machine_message: format!("semgrep-exec-failed:{e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            return;
        }
    };

    // semgrep returns 0 for no findings, 1 for findings, other for errors
    if !output.status.success() && output.status.code() != Some(1) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            check_name: CheckName::SemgrepExecutionFailed,
            human_message: format!(
                "Semgrep returned exit code {}. {}",
                output.status.code().unwrap_or(-1),
                if stderr.is_empty() {
                    "No error output."
                } else {
                    "Check semgrep installation."
                }
            ),
            machine_message: format!("semgrep-exit:{}", output.status.code().unwrap_or(-1)),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Warning,
        });
        return;
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_semgrep_output(&stdout, diagnostics);
}

/// Parse semgrep JSON output and map findings to diagnostics.
fn parse_semgrep_output(json_str: &str, diagnostics: &mut Vec<Diagnostic>) {
    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(e) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                check_name: CheckName::SemgrepExecutionFailed,
                human_message: format!("Could not parse semgrep JSON output: {e}"),
                machine_message: format!("semgrep-json-parse-failed:{e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Warning,
            });
            return;
        }
    };

    let results = match parsed.get("results").and_then(|r| r.as_array()) {
        Some(arr) => arr,
        None => return,
    };

    for result in results {
        let check_id = result
            .get("check_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let message = result
            .get("extra")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("semgrep finding");
        let semgrep_severity = result
            .get("extra")
            .and_then(|e| e.get("severity"))
            .and_then(|s| s.as_str())
            .unwrap_or("WARNING");
        let file_path = result
            .get("path")
            .and_then(|p| p.as_str())
            .map(PathBuf::from);

        let severity = match semgrep_severity {
            "ERROR" => Severity::Error,
            "WARNING" => Severity::Warning,
            "INFO" => Severity::Suggestion,
            _ => Severity::Warning,
        };

        diagnostics.push(Diagnostic {
            severity,
            check_name: CheckName::RemoteExecutionPattern,
            human_message: format!("[{check_id}] {message}"),
            machine_message: format!("semgrep:{check_id}"),
            doc_url: None,
            file_path,
            base_severity: severity,
        });
    }
}

// ── No-semgrep fallback ───────────────────────────────────────────────────

fn emit_no_semgrep_diagnostics(scripts: &[&FileEntry], diagnostics: &mut Vec<Diagnostic>) {
    let script_names: Vec<String> = scripts
        .iter()
        .map(|s| s.path.display().to_string())
        .collect();

    diagnostics.push(Diagnostic {
        severity: Severity::Suggestion,
        check_name: CheckName::ScriptsDetectedNoSemgrep,
        human_message: format!(
            "This skill contains script files ({}). Install semgrep for automated security analysis.",
            script_names.join(", ")
        ),
        machine_message: format!("scripts-no-semgrep:{}", script_names.join(",")),
        doc_url: None,
        file_path: None,
        base_severity: Severity::Suggestion,
    });

    for script in scripts {
        diagnostics.push(Diagnostic {
            severity: Severity::Info,
            check_name: CheckName::ScriptDetected,
            human_message: format!("Script file detected: {}", script.path.display()),
            machine_message: format!("script:{}", script.path.display()),
            doc_url: None,
            file_path: Some(script.path.clone()),
            base_severity: Severity::Info,
        });
    }
}

// ── Language → extension mapping ──────────────────────────────────────────

/// Map a language tag to a file extension. Public for testing.
pub fn lang_to_extension(lang: &str) -> Option<&'static str> {
    match lang.to_lowercase().as_str() {
        "python" | "py" => Some(".py"),
        "bash" | "sh" | "shell" | "zsh" => Some(".sh"),
        "ruby" | "rb" => Some(".rb"),
        "javascript" | "js" => Some(".js"),
        "typescript" | "ts" => Some(".ts"),
        _ => None,
    }
}

use std::path::PathBuf;

// ── Unit tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_exec_patterns_compile() {
        for pattern in REMOTE_EXEC_PATTERNS {
            Regex::new(pattern).expect("pattern should compile");
        }
    }

    #[test]
    fn lang_extension_known() {
        assert_eq!(lang_to_extension("python"), Some(".py"));
        assert_eq!(lang_to_extension("py"), Some(".py"));
        assert_eq!(lang_to_extension("bash"), Some(".sh"));
        assert_eq!(lang_to_extension("sh"), Some(".sh"));
        assert_eq!(lang_to_extension("shell"), Some(".sh"));
        assert_eq!(lang_to_extension("zsh"), Some(".sh"));
        assert_eq!(lang_to_extension("ruby"), Some(".rb"));
        assert_eq!(lang_to_extension("rb"), Some(".rb"));
        assert_eq!(lang_to_extension("javascript"), Some(".js"));
        assert_eq!(lang_to_extension("js"), Some(".js"));
        assert_eq!(lang_to_extension("typescript"), Some(".ts"));
        assert_eq!(lang_to_extension("ts"), Some(".ts"));
    }

    #[test]
    fn lang_extension_unknown() {
        assert_eq!(lang_to_extension("rust"), None);
        assert_eq!(lang_to_extension("go"), None);
        assert_eq!(lang_to_extension("unknown"), None);
    }

    #[test]
    fn lang_extension_case_insensitive() {
        assert_eq!(lang_to_extension("Python"), Some(".py"));
        assert_eq!(lang_to_extension("BASH"), Some(".sh"));
        assert_eq!(lang_to_extension("JavaScript"), Some(".js"));
    }

    #[test]
    fn bundled_rules_are_valid_yaml() {
        for (name, content) in BUNDLED_RULES {
            let parsed: serde_yaml::Value = serde_yaml::from_str(content)
                .unwrap_or_else(|e| panic!("{name} is invalid YAML: {e}"));
            assert!(
                parsed.get("rules").is_some(),
                "{name} must have a 'rules' key"
            );
        }
    }

    #[test]
    fn which_semgrep_nonexistent_returns_none() {
        assert!(which_semgrep("/definitely/nonexistent/semgrep").is_none());
    }

    #[test]
    fn parse_semgrep_empty_results() {
        let json = r#"{"results": [], "errors": []}"#;
        let mut diags = Vec::new();
        parse_semgrep_output(json, &mut diags);
        assert!(diags.is_empty());
    }

    #[test]
    fn parse_semgrep_with_findings() {
        let json = r#"{
            "results": [
                {
                    "check_id": "shell-eval",
                    "path": "/tmp/test.sh",
                    "extra": {
                        "message": "Use of eval detected",
                        "severity": "WARNING"
                    }
                },
                {
                    "check_id": "python-exec",
                    "path": "/tmp/test.py",
                    "extra": {
                        "message": "exec() call detected",
                        "severity": "ERROR"
                    }
                }
            ]
        }"#;
        let mut diags = Vec::new();
        parse_semgrep_output(json, &mut diags);
        assert_eq!(diags.len(), 2);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert_eq!(diags[1].severity, Severity::Error);
    }

    #[test]
    fn parse_semgrep_info_maps_to_suggestion() {
        let json = r#"{
            "results": [
                {
                    "check_id": "hardcoded-url",
                    "path": "/tmp/test.sh",
                    "extra": {
                        "message": "Hardcoded URL",
                        "severity": "INFO"
                    }
                }
            ]
        }"#;
        let mut diags = Vec::new();
        parse_semgrep_output(json, &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Suggestion);
    }

    #[test]
    fn parse_semgrep_invalid_json() {
        let mut diags = Vec::new();
        parse_semgrep_output("not json", &mut diags);
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].check_name, CheckName::SemgrepExecutionFailed);
    }
}
