#![forbid(unsafe_code)]

pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod formatter;
pub mod git;
pub mod models;
pub mod parser;
pub mod passes;
pub mod paths;
pub mod pipeline;
pub mod prompt;
pub mod scan;
pub use config::ValidatorConfig;
pub use discovery::{discover_skills, DiscoveredSkill, DiscoveryResult};
pub use formatter::{format_human, format_json};
pub use git::{find_repo_root, GitError};
pub use models::{Diagnostic, Severity};
pub use parser::{find_skill_md, parse_frontmatter, read_properties};
pub use paths::{expand_path, PathsConfig, PathsError};
pub use pipeline::{exit_code, run_pipeline, PipelineResult};
pub use prompt::to_prompt;
pub use scan::{find_duplicates, scan, ScanOptions, ScanResult, SkillValidation};
