#![allow(dead_code)]

mod cli;
mod config;
mod discovery;
mod error;
mod formatter;
mod git;
mod models;
mod parser;
mod passes;
mod paths;
mod pipeline;
mod prompt;
mod scan;
mod validator;

use cli::run;

fn main() {
    run();
}
