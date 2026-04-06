mod cli;
mod config;
mod discovery;
mod error;
mod git;
mod models;
mod parser;
mod paths;
mod prompt;
mod scan;
mod validator;

use cli::run;

fn main() {
    run();
}
