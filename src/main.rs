mod cli;
mod error;
mod models;
mod parser;
mod prompt;
mod validator;

use cli::run;

fn main() {
    run();
}
