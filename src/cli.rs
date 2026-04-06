use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use log::LevelFilter;
use owo_colors::OwoColorize;
use std::io::{IsTerminal, Write};
use std::path::Path;
use std::process;

use crate::formatter::{format_human, format_json};
use crate::models::Severity;
use crate::parser::read_properties;
use crate::paths::PathsConfig;
use crate::pipeline::{exit_code, run_pipeline};
use crate::prompt::to_prompt;
use crate::scan::{find_duplicates, scan, ScanOptions};

#[derive(Parser)]
#[command(name = "skills-validator")]
#[command(version = "0.1.7")]
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
    scan            Scan for skills across multiple tool directories

EXAMPLES:
    skills-validator validate ~/.agents/skills/my-skill
    skills-validator --output-format json validate ~/.agents/skills/my-skill
    skills-validator --strict validate ~/.agents/skills/my-skill
    skills-validator read-properties ~/.agents/skills/rust
    skills-validator to-prompt ~/.agents/skills/*
    skills-validator scan --all

LOG LEVELS:
    error   - Show only errors
    warn    - Show warnings and errors (default)
    info    - Show informational messages and above
    debug   - Show all messages including detailed debug info

OUTPUT:
    stdout  - Data/results (validation results, YAML, XML)
    stderr  - All log messages (INFO, WARN, DEBUG, errors)

    Use --output-format json for structured JSON validation output to stdout.

See https://agentskills.io/specification for the full specification."
)]
struct Cli {
    /// Set log level (error, warn, info, debug)
    #[arg(short, long, value_name = "LEVEL", default_value = "info")]
    log_level: LevelFilter,

    /// Output logs as JSON to stderr (deprecated: use --output-format json)
    #[arg(long)]
    json: bool,

    /// Output format for validate command: human (default) or json
    #[arg(long, value_name = "FORMAT")]
    output_format: Option<String>,

    /// Minimum severity to display: info (default), suggestion, warning, error
    #[arg(long, value_name = "LEVEL", default_value = "info")]
    severity: Severity,

    /// Promote warnings and suggestions to exit code 1
    #[arg(long)]
    strict: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[command(arg_required_else_help = true)]
enum Commands {
    /// Validate a skill directory
    Validate {
        /// Path to the skill directory
        path: String,
    },

    /// Read and output skill properties as YAML
    ReadProperties {
        /// Path to the skill directory
        path: String,
    },

    /// Generate <available_skills> XML for agent prompts
    ToPrompt {
        /// Paths to skill directories (one or more)
        #[arg(required = true)]
        paths: Vec<String>,
    },

    /// Write default config to XDG config directory
    Setup,

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for (bash, zsh, fish, elvish, powershell)
        shell: Shell,
    },

    /// Scan for skills across multiple tool directories
    Scan {
        /// Scan all locations (default: $CWD->repo root + $HOME)
        #[arg(long, group = "scan_scope")]
        all: bool,

        /// Scan $HOME for all tool directories
        #[arg(long, group = "scan_scope")]
        user: bool,

        /// Scan $CWD->repo root (requires git repo)
        #[arg(long, group = "scan_scope")]
        repo: bool,

        /// Comma-separated tool names to scan
        #[arg(long, value_delimiter = ',')]
        tool: Vec<String>,

        /// Discover paths without validating
        #[arg(long)]
        dry_run: bool,

        /// Show detailed output per skill
        #[arg(long)]
        verbose: bool,
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

/// Resolve the effective output format from CLI flags.
///
/// Returns the format string ("human" or "json") and whether `--json` (deprecated)
/// was used for log formatting on non-validate commands.
fn resolve_output_format(cli: &Cli) -> (String, bool) {
    let mut use_json_logs = cli.json;

    let format = if let Some(ref fmt) = cli.output_format {
        fmt.clone()
    } else if cli.json {
        "json".to_string()
    } else {
        "human".to_string()
    };

    // If --json was used, emit deprecation warning
    if cli.json {
        eprintln!(
            "Warning: --json is deprecated. Use --output-format json instead. \
             Note: --output-format json writes structured validation JSON to stdout, \
             whereas --json wrote JSON log lines to stderr."
        );
        use_json_logs = true;
    }

    (format, use_json_logs)
}

pub fn run() {
    let cli = Cli::parse();

    let (output_format, use_json_logs) = resolve_output_format(&cli);
    let strict = cli.strict;
    let min_severity = cli.severity;

    // Initialize logger - all logs go to stderr
    init_logger(cli.log_level, use_json_logs);

    match cli.command {
        Commands::Validate { path } => {
            let path = Path::new(&path);
            let (config, config_diags) = crate::config::load();

            // Log any config diagnostics
            for diag in &config_diags {
                match diag.severity {
                    Severity::Error => log::error!("{}", diag.human_message),
                    Severity::Warning => log::warn!("{}", diag.human_message),
                    _ => log::info!("{}", diag.human_message),
                }
            }

            let result = run_pipeline(path, &config);

            if output_format == "json" {
                let json = format_json(&result, path, min_severity, strict);
                println!("{}", json);
            } else {
                let human = format_human(&result, path, min_severity);
                print!("{}", human);
            }

            process::exit(exit_code(&result.diagnostics, strict));
        }
        Commands::ReadProperties { path } => {
            let path = Path::new(&path);
            match read_properties(path) {
                Ok(props) => {
                    log::debug!("Read properties from {:?}", path);
                    let yaml = serde_yaml::to_string(&props.to_dict()).unwrap();
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
            println!("{}", prompt);
        }
        Commands::Setup => match crate::config::setup() {
            Ok(path) => {
                println!("Config file written to {}", path.display());
            }
            Err(e) => {
                log::error!("{}", e);
                process::exit(1);
            }
        },
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut std::io::stdout());
        }
        Commands::Scan {
            all,
            user,
            repo,
            tool,
            dry_run,
            verbose,
        } => {
            // Validate tool names if specified
            let config = match PathsConfig::load() {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to load paths configuration: {}", e);
                    process::exit(2);
                }
            };

            if !tool.is_empty() {
                for t in &tool {
                    if !config.has_tool(t) {
                        log::error!("Unknown tool: {}", t);
                        log::info!("Available tools: {}", config.tool_names().join(", "));
                        process::exit(2);
                    }
                }
            }

            if dry_run {
                println!(
                    "Dry run - would scan with options: all={}, user={}, repo={}, tools={:?}",
                    all, user, repo, tool
                );
                return;
            }

            // Perform the scan
            let result = scan(&ScanOptions {
                all,
                user,
                repo,
                tools: tool,
                verbose,
            });

            // Output results
            println!("\n=== Scan Results ===");
            println!("Total skills found: {}", result.total_skills);
            println!("Valid: {}", result.valid_count);
            println!("Invalid: {}", result.invalid_count);
            println!("Warnings: {}", result.warning_count);

            // Check for duplicates
            let duplicates = find_duplicates(&result);
            if !duplicates.is_empty() {
                println!("\n=== Duplicate Skills Found ===");
                for dup_group in &duplicates {
                    let name = dup_group[0]
                        .skill
                        .directory
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    println!("Duplicate: {}", name);
                    for dup in dup_group {
                        println!("  - {:?}", dup.skill.directory);
                    }
                }
            }

            // Set exit code based on results
            if result.invalid_count > 0 {
                process::exit(1);
            }
        }
    }
}
