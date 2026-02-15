use clap::{Parser, Subcommand};
use log::LevelFilter;
use owo_colors::OwoColorize;
use std::io::{IsTerminal, Write};
use std::path::Path;
use std::process;

use crate::parser::read_properties;
use crate::prompt::to_prompt;
use crate::validator::validate;

#[derive(Parser)]
#[command(name = "skills-validator")]
#[command(version = "0.1.2")]
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

LOG LEVELS:
    error   - Show only errors
    warn    - Show warnings and errors (default)
    info    - Show informational messages and above
    debug   - Show all messages including detailed debug info

OUTPUT:
    stdout  - Data/results (validation results, YAML, XML)
    stderr  - All log messages (INFO, WARN, DEBUG, errors)

    Use --json for JSON-formatted log output to stderr

See https://agentskills.io/specification for the full specification."
)]
struct Cli {
    /// Set log level (error, warn, info, debug)
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    log_level: LevelFilter,

    /// Output logs as JSON to stderr
    #[arg(long)]
    json: bool,

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

struct LogFormatter {
    use_colors: bool,
    use_json: bool,
}

impl LogFormatter {
    fn new(use_json: bool) -> Self {
        Self {
            use_colors: std::io::stderr().is_terminal(),
            use_json,
        }
    }

    fn format(
        &self,
        buf: &mut env_logger::fmt::Formatter,
        record: &log::Record,
    ) -> std::io::Result<()> {
        if self.use_json {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            let level = record.level();
            let target = record.target();
            let args = record.args();
            writeln!(
                buf,
                r#"{{"time":{},"level":"{}","target":"{}","message":"{}"}}"#,
                timestamp, level, target, args
            )
        } else {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| {
                    let secs = d.as_secs();
                    let hours = (secs / 3600) % 24;
                    let mins = (secs / 60) % 60;
                    let s = secs % 60;
                    format!("{:02}:{:02}:{:02}", hours, mins, s)
                })
                .unwrap_or_else(|_| "00:00:00".to_string());
            let level = record.level();
            let args = record.args();

            if self.use_colors {
                let level_str = match level {
                    log::Level::Error => format!("{}", "ERROR".red()),
                    log::Level::Warn => format!("{}", "WARN".yellow()),
                    log::Level::Info => format!("{}", "INFO".green()),
                    log::Level::Debug => format!("{}", "DEBUG".cyan()),
                    log::Level::Trace => format!("{}", "TRACE".dimmed()),
                };
                writeln!(buf, "{} {} {}", timestamp.dimmed(), level_str, args)
            } else {
                writeln!(buf, "{} {} - {}", timestamp, level, args)
            }
        }
    }
}

fn init_logger(level: LevelFilter, use_json: bool) {
    let formatter = LogFormatter::new(use_json);

    env_logger::Builder::new()
        .filter_level(level)
        .format(move |buf, record| formatter.format(buf, record))
        .init();
}

pub fn run() {
    let cli = Cli::parse();

    // Initialize logger - all logs go to stderr
    init_logger(cli.log_level, cli.json);

    match cli.command {
        Commands::Validate { path } => {
            let path = Path::new(&path);
            let result = validate(path);

            // When --json is passed, output result as JSON to stderr
            if cli.json {
                let json_result = serde_json::json!({
                    "valid": result.errors.is_empty(),
                    "errors": result.errors,
                    "warnings": result.warnings
                });
                eprintln!("{}", json_result);

                if result.errors.is_empty() {
                    process::exit(0);
                } else {
                    process::exit(1);
                }
            }

            // Text output mode
            for warning in &result.warnings {
                log::warn!("{}", warning);
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
                    log::error!("{}", error);
                }
                process::exit(1);
            }
        }
        Commands::ReadProperties { path } => {
            let path = Path::new(&path);
            match read_properties(path) {
                Ok(props) => {
                    log::debug!("Read properties from {:?}", path);
                    let yaml = serde_yaml::to_string(&props.to_dict()).unwrap();
                    // YAML output goes to stdout (data)
                    print!("{}", yaml);
                }
                Err(e) => {
                    log::error!("Failed to read properties: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::ToPrompt { paths } => {
            log::debug!("Generating prompt for {} skills", paths.len());
            let refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            let prompt = to_prompt(&refs);
            // XML output goes to stdout (data)
            println!("{}", prompt);
        }
    }
}
