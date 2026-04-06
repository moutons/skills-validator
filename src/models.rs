// New pipeline types are defined ahead of their consumers (Tasks 2-10).
// Remove this allow as consuming code is added.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

// ---------------------------------------------------------------------------
// Severity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Suggestion,
    Warning,
    Error,
}

/// Promote a severity by `levels` steps (capped at Error).
pub fn escalate(base: Severity, levels: u8) -> Severity {
    let idx = base as u8;
    let max = Severity::Error as u8;
    let new = idx.saturating_add(levels).min(max);
    match new {
        0 => Severity::Info,
        1 => Severity::Suggestion,
        2 => Severity::Warning,
        _ => Severity::Error,
    }
}

// ---------------------------------------------------------------------------
// Sizeyness
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sizeyness {
    #[default]
    Simple,
    Moderate,
    Hefty,
}

impl Sizeyness {
    /// Determine sizeyness from file count, subdirectory count, and whether
    /// orchestration frontmatter fields (hooks, agent, context) are present.
    pub fn from_counts(files: usize, subdirs: usize, has_orchestration: bool) -> Self {
        if has_orchestration || files >= 6 || subdirs >= 3 {
            Self::Hefty
        } else if files >= 3 || subdirs >= 1 {
            Self::Moderate
        } else {
            Self::Simple
        }
    }
}

// ---------------------------------------------------------------------------
// CheckName
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CheckName {
    // Parse
    #[serde(rename = "skill-file-exists")]
    SkillFileExists,
    #[serde(rename = "skill-file-casing")]
    SkillFileCasing,
    #[serde(rename = "frontmatter-present")]
    FrontmatterPresent,
    #[serde(rename = "frontmatter-valid-yaml")]
    FrontmatterValidYaml,
    #[serde(rename = "frontmatter-is-mapping")]
    FrontmatterIsMapping,

    // Structure
    #[serde(rename = "binary-detected")]
    BinaryDetected,
    #[serde(rename = "scripts-in-root")]
    ScriptsInRoot,
    #[serde(rename = "sizeyness-info")]
    SizeynessInfo,

    // Content — frontmatter
    #[serde(rename = "name-missing")]
    NameMissing,
    #[serde(rename = "name-format")]
    NameFormat,
    #[serde(rename = "name-directory-match")]
    NameDirectoryMatch,
    #[serde(rename = "description-missing")]
    DescriptionMissing,
    #[serde(rename = "description-length")]
    DescriptionLength,
    #[serde(rename = "description-trigger-language")]
    DescriptionTriggerLanguage,
    #[serde(rename = "unknown-field")]
    UnknownField,
    #[serde(rename = "extension-field-compatibility")]
    ExtensionFieldCompatibility,
    #[serde(rename = "context-valid-value")]
    ContextValidValue,
    #[serde(rename = "agent-with-context")]
    AgentWithContext,
    #[serde(rename = "model-recognized")]
    ModelRecognized,

    // Content — quality
    #[serde(rename = "has-trigger-conditions")]
    HasTriggerConditions,
    #[serde(rename = "has-examples")]
    HasExamples,
    #[serde(rename = "has-behavioral-constraints")]
    HasBehavioralConstraints,
    #[serde(rename = "has-gotchas")]
    HasGotchas,
    #[serde(rename = "body-length")]
    BodyLength,
    #[serde(rename = "windows-paths")]
    WindowsPaths,

    // Content — positive reinforcement
    #[serde(rename = "has-gotchas-section")]
    HasGotchasSection,
    #[serde(rename = "has-validation-loop")]
    HasValidationLoop,
    #[serde(rename = "has-progressive-disclosure")]
    HasProgressiveDisclosure,
    #[serde(rename = "has-concrete-examples")]
    HasConcreteExamples,

    // References
    #[serde(rename = "broken-reference")]
    BrokenReference,
    #[serde(rename = "orphaned-files")]
    OrphanedFiles,
    #[serde(rename = "hooks-script-missing")]
    HooksScriptMissing,
    #[serde(rename = "circular-reference")]
    CircularReference,
    #[serde(rename = "hop-limit-reached")]
    HopLimitReached,
    #[serde(rename = "path-traversal-blocked")]
    PathTraversalBlocked,

    // Security
    #[serde(rename = "scripts-detected-no-semgrep")]
    ScriptsDetectedNoSemgrep,
    #[serde(rename = "script-detected")]
    ScriptDetected,
    #[serde(rename = "remote-execution-pattern")]
    RemoteExecutionPattern,
    #[serde(rename = "semgrep-execution-failed")]
    SemgrepExecutionFailed,

    // Config
    #[serde(rename = "config-invalid")]
    ConfigInvalid,

    // Pipeline
    #[serde(rename = "pipeline-error")]
    PipelineError,
}

// ---------------------------------------------------------------------------
// Diagnostic
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub check_name: CheckName,
    pub human_message: String,
    pub machine_message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<PathBuf>,
    pub base_severity: Severity,
}

// ---------------------------------------------------------------------------
// PipelineError
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, thiserror::Error)]
pub enum PipelineError {
    #[error("parse failed for {path}: {reason}")]
    ParseFailed { path: PathBuf, reason: String },
    #[error("I/O error for {path}: {reason}")]
    IoError { path: PathBuf, reason: String },
    #[error("semgrep failed: {reason}")]
    SemgrepFailed { reason: String },
    #[error("invalid configuration: {reason}")]
    ConfigInvalid { reason: String },
}

// ---------------------------------------------------------------------------
// AST helper structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heading {
    pub level: u8,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub text: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeBlock {
    pub language: Option<String>,
    pub content: String,
}

// ---------------------------------------------------------------------------
// FileEntry / FileType
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Markdown,
    Script,
    Binary,
    Config,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub file_type: FileType,
    pub size_bytes: u64,
}

// ---------------------------------------------------------------------------
// SkillContext
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContext {
    // Pass 1: Parse — raw YAML value; will be refined to a typed Frontmatter struct in Task 3
    pub frontmatter: serde_yaml::Value,
    pub headings: Vec<Heading>,
    pub links: Vec<Link>,
    pub code_blocks: Vec<CodeBlock>,
    pub prose_text: String,

    // Pass 2: Structure
    pub sizeyness: Sizeyness,
    pub file_inventory: Vec<FileEntry>,
    pub subdirectories: Vec<PathBuf>,

    // Pass 4: accumulated from markdown chain
    pub referenced_files: HashSet<PathBuf>,
}

impl Default for SkillContext {
    fn default() -> Self {
        Self {
            frontmatter: serde_yaml::Value::Null,
            headings: Vec::new(),
            links: Vec::new(),
            code_blocks: Vec::new(),
            prose_text: String::new(),
            sizeyness: Sizeyness::default(),
            file_inventory: Vec::new(),
            subdirectories: Vec::new(),
            referenced_files: HashSet::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Legacy types (deprecated, kept for backward compatibility)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProperties {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "allowed-tools")]
    pub allowed_tools: Option<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub metadata: std::collections::HashMap<String, String>,
}

impl SkillProperties {
    pub fn to_dict(&self) -> serde_yaml::Value {
        let mut map = serde_yaml::Mapping::new();
        map.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String(self.name.clone()),
        );
        map.insert(
            serde_yaml::Value::String("description".to_string()),
            serde_yaml::Value::String(self.description.clone()),
        );

        if let Some(ref license) = self.license {
            map.insert(
                serde_yaml::Value::String("license".to_string()),
                serde_yaml::Value::String(license.clone()),
            );
        }

        if let Some(ref compatibility) = self.compatibility {
            map.insert(
                serde_yaml::Value::String("compatibility".to_string()),
                serde_yaml::Value::String(compatibility.clone()),
            );
        }

        if let Some(ref allowed_tools) = self.allowed_tools {
            map.insert(
                serde_yaml::Value::String("allowed-tools".to_string()),
                serde_yaml::Value::String(allowed_tools.clone()),
            );
        }

        if !self.metadata.is_empty() {
            let mut meta_map = serde_yaml::Mapping::new();
            for (k, v) in &self.metadata {
                meta_map.insert(
                    serde_yaml::Value::String(k.clone()),
                    serde_yaml::Value::String(v.clone()),
                );
            }
            map.insert(
                serde_yaml::Value::String("metadata".to_string()),
                serde_yaml::Value::Mapping(meta_map),
            );
        }

        serde_yaml::Value::Mapping(map)
    }
}
