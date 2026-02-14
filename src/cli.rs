use clap::{Parser, Subcommand};
use std::path::Path;
use std::process;

use crate::parser::read_properties;
use crate::prompt::to_prompt;
use crate::validator::validate;

#[derive(Parser)]
#[command(name = "skills-validator")]
#[command(about = "Reference library for Agent Skills", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Validate {
        path: String,
    },
    ReadProperties {
        path: String,
    },
    ToPrompt {
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
                eprintln!("Validation warning: {}", warning);
            }

            if result.errors.is_empty() {
                if result.warnings.is_empty() {
                    println!("Skill is valid.");
                } else {
                    println!("Skill is valid (with warnings).");
                }
                process::exit(0);
            } else {
                for error in &result.errors {
                    eprintln!("Validation error: {}", error);
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
