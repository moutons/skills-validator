use clap::{Parser, Subcommand};
use std::path::Path;
use std::process;

use crate::parser::read_properties;
use crate::prompt::to_prompt;
use crate::validator::validate;

#[derive(Parser)]
#[command(name = "skills-validator")]
#[command(version = "0.1.0")]
#[command(
    about = "Validate Agent Skills and generate prompt XML",
    long_about = "
Validate Agent Skills and generate prompt XML

USAGE:
    skills-validator <COMMAND> [OPTIONS]

COMMANDS:
    validate         Validate a skill directory against the Agent Skills spec
    read-properties Parse and output skill frontmatter as YAML
    to-prompt       Generate <available_skills> XML for agent prompts

EXAMPLES:
    skills-validator validate ~/.agents/skills/my-skill
    skills-validator read-properties ~/.agents/skills/rust
    skills-validator to-prompt ~/.agents/skills/*

For pre-commit validation of all skills:
    ./scripts/validate-skills.sh ~/.agents/skills

See https://agentskills.io/specification for the full specification."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum Commands {
    /// Validate a skill directory
    ///
    /// Checks SKILL.md frontmatter against the Agent Skills specification.
    /// Returns exit code 0 if valid, 1 if errors found.
    ///
    /// Warnings are issued for:
    /// - Non-spec Claude Code extension fields
    /// - Missing content keywords (never, always, when, example)
    Validate {
        /// Path to the skill directory
        path: String,
    },

    /// Read and output skill properties as YAML
    ///
    /// Parses SKILL.md frontmatter and outputs the properties.
    /// Does not perform validation - use 'validate' command for that.
    ReadProperties {
        /// Path to the skill directory
        path: String,
    },

    /// Generate <available_skills> XML for agent prompts
    ///
    /// Creates an XML block listing all specified skills with their
    /// names, descriptions, and file locations for system prompts.
    ToPrompt {
        /// Paths to skill directories (one or more)
        #[arg(required = true)]
        paths: Vec<String>,
    },
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Validate { path } => {
            let path = Path::new(&path);
            let result = validate(path);

            for warning in &result.warnings {
                eprintln!("Warning: {}", warning);
            }

            if result.errors.is_empty() {
                if result.warnings.is_empty() {
                    println!("✓ Skill is valid");
                } else {
                    println!("✓ Skill is valid (with warnings)");
                }
                process::exit(0);
            } else {
                for error in &result.errors {
                    eprintln!("Error: {}", error);
                }
                process::exit(1);
            }
        }
        Commands::ReadProperties { path } => {
            let path = Path::new(&path);
            match read_properties(path) {
                Ok(props) => {
                    let yaml = serde_yaml::to_string(&props.to_dict()).unwrap();
                    print!("{}", yaml);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::ToPrompt { paths } => {
            let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            let prompt = to_prompt(&refs);
            println!("{}", prompt);
        }
    }
}
