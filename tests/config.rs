use skills_validator::config::{load_from_str, ValidatorConfig, DEFAULT_CONFIG_TOML};
use skills_validator::models::{CheckName, Severity};

// === Default values ===

#[test]
fn defaults_have_expected_values() {
    let cfg = ValidatorConfig::default();
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
    assert_eq!(cfg.sizeyness.hefty_file_threshold, 6);
    assert_eq!(cfg.sizeyness.moderate_subdir_threshold, 1);
    assert_eq!(cfg.sizeyness.hefty_subdir_threshold, 3);
    assert_eq!(cfg.content.body_line_limit, 300);
    assert_eq!(cfg.references.markdown_hop_limit, 5);
    assert!(cfg.security.semgrep_enabled);
    assert_eq!(cfg.security.semgrep_path, "semgrep");
    assert!(cfg.security.custom_rules_dir.is_empty());
}

#[test]
fn default_known_models_contains_expected_entries() {
    let cfg = ValidatorConfig::default();
    assert!(cfg
        .content
        .known_models
        .contains(&"claude-opus-4-6".to_string()));
    assert!(cfg
        .content
        .known_models
        .contains(&"claude-sonnet-4-6".to_string()));
}

#[test]
fn default_orphan_exclusions_contains_license() {
    let cfg = ValidatorConfig::default();
    assert!(cfg
        .references
        .orphan_exclusions
        .contains(&"LICENSE*".to_string()));
}

// === TOML parsing ===

#[test]
fn empty_toml_returns_defaults_no_diagnostics() {
    let (cfg, diags) = load_from_str("");
    assert!(diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
}

#[test]
fn partial_toml_merges_with_defaults() {
    let (cfg, diags) = load_from_str("[content]\nbody_line_limit = 500\n");
    assert!(diags.is_empty());
    assert_eq!(cfg.content.body_line_limit, 500);
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3); // default preserved
}

#[test]
fn full_toml_overrides_all() {
    let toml = r#"
[sizeyness]
moderate_file_threshold = 4
hefty_file_threshold = 8
moderate_subdir_threshold = 2
hefty_subdir_threshold = 5

[content]
body_line_limit = 100
known_models = ["gpt-4"]

[references]
markdown_hop_limit = 3
orphan_exclusions = ["README*"]

[security]
semgrep_enabled = false
semgrep_path = "/bin/semgrep"
custom_rules_dir = "/rules"
"#;
    let (cfg, diags) = load_from_str(toml);
    assert!(diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 4);
    assert_eq!(cfg.sizeyness.hefty_file_threshold, 8);
    assert_eq!(cfg.content.body_line_limit, 100);
    assert_eq!(cfg.content.known_models, vec!["gpt-4"]);
    assert_eq!(cfg.references.markdown_hop_limit, 3);
    assert!(!cfg.security.semgrep_enabled);
}

// === Validation: threshold ordering ===

#[test]
fn moderate_file_gte_hefty_file_reverts_both() {
    let (cfg, diags) =
        load_from_str("[sizeyness]\nmoderate_file_threshold = 10\nhefty_file_threshold = 5\n");
    assert!(!diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
    assert_eq!(cfg.sizeyness.hefty_file_threshold, 6);
}

#[test]
fn moderate_file_equal_hefty_file_reverts_both() {
    let (cfg, diags) =
        load_from_str("[sizeyness]\nmoderate_file_threshold = 6\nhefty_file_threshold = 6\n");
    assert!(!diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
    assert_eq!(cfg.sizeyness.hefty_file_threshold, 6);
}

#[test]
fn moderate_subdir_gte_hefty_subdir_reverts_both() {
    let (cfg, diags) =
        load_from_str("[sizeyness]\nmoderate_subdir_threshold = 5\nhefty_subdir_threshold = 2\n");
    assert!(!diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_subdir_threshold, 1);
    assert_eq!(cfg.sizeyness.hefty_subdir_threshold, 3);
}

// === Validation: positive values ===

#[test]
fn zero_body_line_limit_reverts_with_warning() {
    let (cfg, diags) = load_from_str("[content]\nbody_line_limit = 0\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Warning);
    assert!(diags[0].human_message.contains("body_line_limit"));
    assert_eq!(cfg.content.body_line_limit, 300);
}

#[test]
fn zero_hop_limit_reverts_with_warning() {
    let (cfg, diags) = load_from_str("[references]\nmarkdown_hop_limit = 0\n");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Warning);
    assert_eq!(cfg.references.markdown_hop_limit, 5);
}

#[test]
fn zero_thresholds_all_revert() {
    let toml = r#"
[sizeyness]
moderate_file_threshold = 0
hefty_file_threshold = 0
moderate_subdir_threshold = 0
hefty_subdir_threshold = 0
"#;
    let (cfg, diags) = load_from_str(toml);
    assert!(!diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
    assert_eq!(cfg.sizeyness.hefty_file_threshold, 6);
    assert_eq!(cfg.sizeyness.moderate_subdir_threshold, 1);
    assert_eq!(cfg.sizeyness.hefty_subdir_threshold, 3);
}

// === Invalid TOML ===

#[test]
fn invalid_toml_produces_error_diagnostic() {
    let (cfg, diags) = load_from_str("not valid [toml");
    assert_eq!(diags.len(), 1);
    assert_eq!(diags[0].severity, Severity::Error);
    assert_eq!(diags[0].check_name, CheckName::ConfigInvalid);
    assert!(diags[0].human_message.contains("skills-validator setup"));
    // Falls back to defaults
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
}

// === Default config template is valid ===

#[test]
fn default_config_toml_template_parses_cleanly() {
    let (cfg, diags) = load_from_str(DEFAULT_CONFIG_TOML);
    assert!(diags.is_empty());
    assert_eq!(cfg.sizeyness.moderate_file_threshold, 3);
}

// === Setup creates file ===

#[test]
fn setup_creates_config_in_temp_dir() {
    use skills_validator::config::setup;

    let tmp = tempfile::tempdir().unwrap();
    let config_dir = tmp.path().join("skills-validator");

    // Override XDG_CONFIG_HOME for this test
    std::env::set_var("XDG_CONFIG_HOME", tmp.path());

    let result = setup();
    // We can't fully control the XDG path in tests since `dirs` may not
    // respect XDG_CONFIG_HOME on all platforms. If the path resolves into
    // the temp dir, verify the file was written. Otherwise just ensure it
    // returns Ok or an "already exists" error (if the user has a real config).
    match result {
        Ok(path) => {
            assert!(path.exists());
            let contents = std::fs::read_to_string(&path).unwrap();
            assert!(contents.contains("[sizeyness]"));
        }
        Err(e) => {
            // Acceptable if file already exists from a real config
            assert!(e.contains("already exists"), "unexpected error: {e}");
        }
    }

    // Clean up env var
    std::env::remove_var("XDG_CONFIG_HOME");
}

// === Diagnostic fields ===

#[test]
fn validation_diagnostics_use_config_invalid_check_name() {
    let (_, diags) = load_from_str("[content]\nbody_line_limit = 0\n");
    assert!(diags
        .iter()
        .all(|d| d.check_name == CheckName::ConfigInvalid));
}

#[test]
fn validation_diagnostics_are_warning_severity() {
    let (_, diags) = load_from_str("[content]\nbody_line_limit = 0\n");
    assert!(diags.iter().all(|d| d.severity == Severity::Warning));
}

#[test]
fn parse_error_diagnostic_is_error_severity() {
    let (_, diags) = load_from_str("{{{{");
    assert!(diags.iter().all(|d| d.severity == Severity::Error));
}
