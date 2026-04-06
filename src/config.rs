// Types and functions are consumed via the library crate (tests, future pipeline tasks).
// Remove this allow as consuming code is added to the binary.
#![allow(dead_code)]

use crate::models::{CheckName, Diagnostic, Severity};
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SizeynessConfig {
    pub moderate_file_threshold: usize,
    pub hefty_file_threshold: usize,
    pub moderate_subdir_threshold: usize,
    pub hefty_subdir_threshold: usize,
}

impl Default for SizeynessConfig {
    fn default() -> Self {
        Self {
            moderate_file_threshold: 3,
            hefty_file_threshold: 6,
            moderate_subdir_threshold: 1,
            hefty_subdir_threshold: 3,
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ContentConfig {
    pub body_line_limit: usize,
    pub known_models: Vec<String>,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            body_line_limit: 300,
            known_models: vec![
                "claude-opus-4-6".to_string(),
                "claude-sonnet-4-6".to_string(),
                "claude-haiku-4-5-20251001".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ReferencesConfig {
    pub markdown_hop_limit: usize,
    pub orphan_exclusions: Vec<String>,
}

impl Default for ReferencesConfig {
    fn default() -> Self {
        Self {
            markdown_hop_limit: 5,
            orphan_exclusions: vec![
                "LICENSE*".to_string(),
                "CHANGELOG*".to_string(),
                "README*".to_string(),
                ".gitignore".to_string(),
                ".*".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    pub semgrep_enabled: bool,
    pub semgrep_path: String,
    pub custom_rules_dir: String,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            semgrep_enabled: true,
            semgrep_path: "semgrep".to_string(),
            custom_rules_dir: String::new(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct ValidatorConfig {
    pub sizeyness: SizeynessConfig,
    pub content: ContentConfig,
    pub references: ReferencesConfig,
    pub security: SecurityConfig,
}

// ---------------------------------------------------------------------------
// Config path resolution
// ---------------------------------------------------------------------------

/// Returns the path to the config file: `$XDG_CONFIG_HOME/skills-validator/config.toml`
pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("skills-validator").join("config.toml"))
}

// ---------------------------------------------------------------------------
// Loading
// ---------------------------------------------------------------------------

/// Load configuration from compiled defaults, config file, and environment
/// variables. Returns the resolved config plus any diagnostics produced during
/// loading (e.g. warnings about invalid values that were reverted to defaults).
pub fn load() -> (ValidatorConfig, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();
    let mut config = ValidatorConfig::default();

    // Try to load config file
    if let Some(path) = config_path() {
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match toml::from_str::<ValidatorConfig>(&contents) {
                    Ok(file_config) => {
                        config = file_config;
                    }
                    Err(e) => {
                        diagnostics.push(Diagnostic {
                            severity: Severity::Error,
                            check_name: CheckName::ConfigInvalid,
                            human_message: format!(
                                "Your config file at {} has invalid TOML: {}. \
                                 Using defaults. You can regenerate it with \
                                 `skills-validator setup`.",
                                path.display(),
                                e
                            ),
                            machine_message: format!("config parse error: {e}"),
                            doc_url: None,
                            file_path: Some(path),
                            base_severity: Severity::Error,
                        });
                    }
                },
                Err(e) => {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        check_name: CheckName::ConfigInvalid,
                        human_message: format!(
                            "Could not read config file at {}: {e}",
                            path.display()
                        ),
                        machine_message: format!("config read error: {e}"),
                        doc_url: None,
                        file_path: Some(path),
                        base_severity: Severity::Error,
                    });
                }
            }
        }
    }

    // Apply env var overrides
    apply_env_overrides(&mut config, &mut diagnostics);

    // Validate and fix
    validate_config(&mut config, &mut diagnostics);

    (config, diagnostics)
}

/// Load configuration from a TOML string (useful for testing).
pub fn load_from_str(toml_str: &str) -> (ValidatorConfig, Vec<Diagnostic>) {
    let mut diagnostics = Vec::new();

    let mut config = match toml::from_str::<ValidatorConfig>(toml_str) {
        Ok(c) => c,
        Err(e) => {
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                check_name: CheckName::ConfigInvalid,
                human_message: format!(
                    "Invalid TOML in config: {e}. Using defaults. \
                     You can regenerate with `skills-validator setup`."
                ),
                machine_message: format!("config parse error: {e}"),
                doc_url: None,
                file_path: None,
                base_severity: Severity::Error,
            });
            ValidatorConfig::default()
        }
    };

    validate_config(&mut config, &mut diagnostics);
    (config, diagnostics)
}

// ---------------------------------------------------------------------------
// Env var overrides
// ---------------------------------------------------------------------------

fn apply_env_overrides(config: &mut ValidatorConfig, diagnostics: &mut Vec<Diagnostic>) {
    try_env_usize(
        "SKILLS_VALIDATOR_SIZEYNESS_MODERATE_FILE_THRESHOLD",
        &mut config.sizeyness.moderate_file_threshold,
        diagnostics,
    );
    try_env_usize(
        "SKILLS_VALIDATOR_SIZEYNESS_HEFTY_FILE_THRESHOLD",
        &mut config.sizeyness.hefty_file_threshold,
        diagnostics,
    );
    try_env_usize(
        "SKILLS_VALIDATOR_SIZEYNESS_MODERATE_SUBDIR_THRESHOLD",
        &mut config.sizeyness.moderate_subdir_threshold,
        diagnostics,
    );
    try_env_usize(
        "SKILLS_VALIDATOR_SIZEYNESS_HEFTY_SUBDIR_THRESHOLD",
        &mut config.sizeyness.hefty_subdir_threshold,
        diagnostics,
    );
    try_env_usize(
        "SKILLS_VALIDATOR_CONTENT_BODY_LINE_LIMIT",
        &mut config.content.body_line_limit,
        diagnostics,
    );
    try_env_usize(
        "SKILLS_VALIDATOR_REFERENCES_MARKDOWN_HOP_LIMIT",
        &mut config.references.markdown_hop_limit,
        diagnostics,
    );
    try_env_bool(
        "SKILLS_VALIDATOR_SECURITY_SEMGREP_ENABLED",
        &mut config.security.semgrep_enabled,
        diagnostics,
    );
    try_env_string(
        "SKILLS_VALIDATOR_SECURITY_SEMGREP_PATH",
        &mut config.security.semgrep_path,
    );
    try_env_string(
        "SKILLS_VALIDATOR_SECURITY_CUSTOM_RULES_DIR",
        &mut config.security.custom_rules_dir,
    );
}

fn try_env_usize(key: &str, target: &mut usize, diagnostics: &mut Vec<Diagnostic>) {
    if let Ok(val) = std::env::var(key) {
        match val.parse::<usize>() {
            Ok(n) => *target = n,
            Err(_) => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    check_name: CheckName::ConfigInvalid,
                    human_message: format!(
                        "Environment variable {key} has non-numeric value \"{val}\"; \
                         keeping previous value ({target})."
                    ),
                    machine_message: format!("env {key}=\"{val}\" is not a valid usize, ignored"),
                    doc_url: None,
                    file_path: None,
                    base_severity: Severity::Warning,
                });
            }
        }
    }
}

fn try_env_bool(key: &str, target: &mut bool, diagnostics: &mut Vec<Diagnostic>) {
    if let Ok(val) = std::env::var(key) {
        match val.to_lowercase().as_str() {
            "true" | "1" | "yes" => *target = true,
            "false" | "0" | "no" => *target = false,
            _ => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Warning,
                    check_name: CheckName::ConfigInvalid,
                    human_message: format!(
                        "Environment variable {key} has unrecognized value \"{val}\"; \
                         keeping previous value ({target}). Use true/false, 1/0, or yes/no."
                    ),
                    machine_message: format!("env {key}=\"{val}\" is not a valid bool, ignored"),
                    doc_url: None,
                    file_path: None,
                    base_severity: Severity::Warning,
                });
            }
        }
    }
}

fn try_env_string(key: &str, target: &mut String) {
    if let Ok(val) = std::env::var(key) {
        *target = val;
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn validate_config(config: &mut ValidatorConfig, diagnostics: &mut Vec<Diagnostic>) {
    let defaults = ValidatorConfig::default();

    // body_line_limit > 0
    if config.content.body_line_limit == 0 {
        diagnostics.push(warn_reverted(
            "content.body_line_limit",
            "0",
            &defaults.content.body_line_limit.to_string(),
        ));
        config.content.body_line_limit = defaults.content.body_line_limit;
    }

    // markdown_hop_limit > 0
    if config.references.markdown_hop_limit == 0 {
        diagnostics.push(warn_reverted(
            "references.markdown_hop_limit",
            "0",
            &defaults.references.markdown_hop_limit.to_string(),
        ));
        config.references.markdown_hop_limit = defaults.references.markdown_hop_limit;
    }

    // moderate_file_threshold must be positive
    if config.sizeyness.moderate_file_threshold == 0 {
        diagnostics.push(warn_reverted(
            "sizeyness.moderate_file_threshold",
            "0",
            &defaults.sizeyness.moderate_file_threshold.to_string(),
        ));
        config.sizeyness.moderate_file_threshold = defaults.sizeyness.moderate_file_threshold;
    }

    // hefty_file_threshold must be positive
    if config.sizeyness.hefty_file_threshold == 0 {
        diagnostics.push(warn_reverted(
            "sizeyness.hefty_file_threshold",
            "0",
            &defaults.sizeyness.hefty_file_threshold.to_string(),
        ));
        config.sizeyness.hefty_file_threshold = defaults.sizeyness.hefty_file_threshold;
    }

    // moderate < hefty (files)
    if config.sizeyness.moderate_file_threshold >= config.sizeyness.hefty_file_threshold {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            check_name: CheckName::ConfigInvalid,
            human_message: format!(
                "sizeyness.moderate_file_threshold ({}) must be less than \
                 hefty_file_threshold ({}); reverting both to defaults ({}, {}).",
                config.sizeyness.moderate_file_threshold,
                config.sizeyness.hefty_file_threshold,
                defaults.sizeyness.moderate_file_threshold,
                defaults.sizeyness.hefty_file_threshold,
            ),
            machine_message: format!(
                "moderate_file_threshold ({}) >= hefty_file_threshold ({}), reverted to defaults",
                config.sizeyness.moderate_file_threshold, config.sizeyness.hefty_file_threshold,
            ),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Warning,
        });
        config.sizeyness.moderate_file_threshold = defaults.sizeyness.moderate_file_threshold;
        config.sizeyness.hefty_file_threshold = defaults.sizeyness.hefty_file_threshold;
    }

    // moderate_subdir_threshold must be positive
    if config.sizeyness.moderate_subdir_threshold == 0 {
        diagnostics.push(warn_reverted(
            "sizeyness.moderate_subdir_threshold",
            "0",
            &defaults.sizeyness.moderate_subdir_threshold.to_string(),
        ));
        config.sizeyness.moderate_subdir_threshold = defaults.sizeyness.moderate_subdir_threshold;
    }

    // hefty_subdir_threshold must be positive
    if config.sizeyness.hefty_subdir_threshold == 0 {
        diagnostics.push(warn_reverted(
            "sizeyness.hefty_subdir_threshold",
            "0",
            &defaults.sizeyness.hefty_subdir_threshold.to_string(),
        ));
        config.sizeyness.hefty_subdir_threshold = defaults.sizeyness.hefty_subdir_threshold;
    }

    // moderate < hefty (subdirs)
    if config.sizeyness.moderate_subdir_threshold >= config.sizeyness.hefty_subdir_threshold {
        diagnostics.push(Diagnostic {
            severity: Severity::Warning,
            check_name: CheckName::ConfigInvalid,
            human_message: format!(
                "sizeyness.moderate_subdir_threshold ({}) must be less than \
                 hefty_subdir_threshold ({}); reverting both to defaults ({}, {}).",
                config.sizeyness.moderate_subdir_threshold,
                config.sizeyness.hefty_subdir_threshold,
                defaults.sizeyness.moderate_subdir_threshold,
                defaults.sizeyness.hefty_subdir_threshold,
            ),
            machine_message: format!(
                "moderate_subdir_threshold ({}) >= hefty_subdir_threshold ({}), reverted to defaults",
                config.sizeyness.moderate_subdir_threshold,
                config.sizeyness.hefty_subdir_threshold,
            ),
            doc_url: None,
            file_path: None,
            base_severity: Severity::Warning,
        });
        config.sizeyness.moderate_subdir_threshold = defaults.sizeyness.moderate_subdir_threshold;
        config.sizeyness.hefty_subdir_threshold = defaults.sizeyness.hefty_subdir_threshold;
    }
}

fn warn_reverted(key: &str, bad_value: &str, default_value: &str) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        check_name: CheckName::ConfigInvalid,
        human_message: format!(
            "{key} has invalid value ({bad_value}); reverted to default ({default_value})."
        ),
        machine_message: format!("{key}={bad_value} invalid, reverted to {default_value}"),
        doc_url: None,
        file_path: None,
        base_severity: Severity::Warning,
    }
}

// ---------------------------------------------------------------------------
// Setup — write commented default config
// ---------------------------------------------------------------------------

/// The default config file content with all values commented out.
pub const DEFAULT_CONFIG_TOML: &str = r#"# skills-validator configuration
# Override order: compiled defaults -> this file -> env vars -> CLI flags
# Env var naming: SKILLS_VALIDATOR_<SECTION>_<KEY> (uppercase)

# [sizeyness]
# moderate_file_threshold = 3
# hefty_file_threshold = 6
# moderate_subdir_threshold = 1
# hefty_subdir_threshold = 3

# [content]
# body_line_limit = 300
# known_models = ["claude-opus-4-6", "claude-sonnet-4-6", "claude-haiku-4-5-20251001"]

# [references]
# markdown_hop_limit = 5
# orphan_exclusions = ["LICENSE*", "CHANGELOG*", "README*", ".gitignore", ".*"]

# [security]
# semgrep_enabled = true
# semgrep_path = "semgrep"
# custom_rules_dir = ""
"#;

/// Run the `setup` subcommand: create the config directory and write commented
/// defaults. Returns `Ok(path)` on success or an error message.
pub fn setup() -> Result<PathBuf, String> {
    let path = config_path().ok_or_else(|| {
        "Could not determine config directory (XDG_CONFIG_HOME not set and \
         no home directory found)."
            .to_string()
    })?;

    if path.exists() {
        return Err(format!(
            "Config file already exists at {}. Remove it first if you want to regenerate.",
            path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Could not create config directory {}: {e}",
                parent.display()
            )
        })?;
    }

    std::fs::write(&path, DEFAULT_CONFIG_TOML)
        .map_err(|e| format!("Could not write config file {}: {e}", path.display()))?;

    Ok(path)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        let config = ValidatorConfig::default();
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
        assert_eq!(config.sizeyness.hefty_file_threshold, 6);
        assert_eq!(config.sizeyness.moderate_subdir_threshold, 1);
        assert_eq!(config.sizeyness.hefty_subdir_threshold, 3);
        assert_eq!(config.content.body_line_limit, 300);
        assert_eq!(config.content.known_models.len(), 3);
        assert_eq!(config.references.markdown_hop_limit, 5);
        assert_eq!(config.references.orphan_exclusions.len(), 5);
        assert!(config.security.semgrep_enabled);
        assert_eq!(config.security.semgrep_path, "semgrep");
        assert!(config.security.custom_rules_dir.is_empty());
    }

    #[test]
    fn test_parse_empty_toml_gives_defaults() {
        let (config, diags) = load_from_str("");
        assert!(diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
        assert_eq!(config.content.body_line_limit, 300);
    }

    #[test]
    fn test_parse_partial_toml() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_file_threshold = 5
"#,
        );
        assert!(diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 5);
        // Other fields get defaults
        assert_eq!(config.sizeyness.hefty_file_threshold, 6);
        assert_eq!(config.content.body_line_limit, 300);
    }

    #[test]
    fn test_parse_full_toml() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_file_threshold = 4
hefty_file_threshold = 8
moderate_subdir_threshold = 2
hefty_subdir_threshold = 5

[content]
body_line_limit = 500
known_models = ["gpt-4", "claude-opus-4-6"]

[references]
markdown_hop_limit = 10
orphan_exclusions = ["LICENSE*"]

[security]
semgrep_enabled = false
semgrep_path = "/usr/local/bin/semgrep"
custom_rules_dir = "/rules"
"#,
        );
        assert!(diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 4);
        assert_eq!(config.sizeyness.hefty_file_threshold, 8);
        assert_eq!(config.content.body_line_limit, 500);
        assert_eq!(config.content.known_models.len(), 2);
        assert_eq!(config.references.markdown_hop_limit, 10);
        assert!(!config.security.semgrep_enabled);
        assert_eq!(config.security.semgrep_path, "/usr/local/bin/semgrep");
    }

    #[test]
    fn test_invalid_toml_gives_error_diagnostic() {
        let (config, diags) = load_from_str("this is [not valid toml");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Error);
        assert_eq!(diags[0].check_name, CheckName::ConfigInvalid);
        assert!(diags[0].human_message.contains("skills-validator setup"));
        // Should fall back to defaults
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
    }

    #[test]
    fn test_validation_body_line_limit_zero() {
        let (config, diags) = load_from_str(
            r#"
[content]
body_line_limit = 0
"#,
        );
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].severity, Severity::Warning);
        assert!(diags[0].human_message.contains("body_line_limit"));
        assert_eq!(config.content.body_line_limit, 300); // reverted
    }

    #[test]
    fn test_validation_hop_limit_zero() {
        let (config, diags) = load_from_str(
            r#"
[references]
markdown_hop_limit = 0
"#,
        );
        assert_eq!(diags.len(), 1);
        assert_eq!(config.references.markdown_hop_limit, 5); // reverted
    }

    #[test]
    fn test_validation_moderate_gte_hefty_files() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_file_threshold = 6
hefty_file_threshold = 6
"#,
        );
        // Should get a warning and revert both
        assert!(!diags.is_empty());
        assert!(diags
            .iter()
            .any(|d| d.human_message.contains("moderate_file_threshold")));
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
        assert_eq!(config.sizeyness.hefty_file_threshold, 6);
    }

    #[test]
    fn test_validation_moderate_gt_hefty_files() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_file_threshold = 10
hefty_file_threshold = 5
"#,
        );
        assert!(!diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
        assert_eq!(config.sizeyness.hefty_file_threshold, 6);
    }

    #[test]
    fn test_validation_moderate_gte_hefty_subdirs() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_subdir_threshold = 5
hefty_subdir_threshold = 3
"#,
        );
        assert!(!diags.is_empty());
        assert_eq!(config.sizeyness.moderate_subdir_threshold, 1);
        assert_eq!(config.sizeyness.hefty_subdir_threshold, 3);
    }

    #[test]
    fn test_validation_zero_thresholds_revert() {
        let (config, diags) = load_from_str(
            r#"
[sizeyness]
moderate_file_threshold = 0
hefty_file_threshold = 0
moderate_subdir_threshold = 0
hefty_subdir_threshold = 0
"#,
        );
        // Should get warnings for zeros, then ordering check may or may not fire
        // depending on revert order, but values should be defaults
        assert!(!diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
        assert_eq!(config.sizeyness.hefty_file_threshold, 6);
        assert_eq!(config.sizeyness.moderate_subdir_threshold, 1);
        assert_eq!(config.sizeyness.hefty_subdir_threshold, 3);
    }

    #[test]
    fn test_default_config_toml_is_valid_commented() {
        // The default config is all comments, so parsing it should give defaults
        let (config, diags) = load_from_str(DEFAULT_CONFIG_TOML);
        assert!(diags.is_empty());
        assert_eq!(config.sizeyness.moderate_file_threshold, 3);
    }
}
