use std::fs;

use colored::Colorize;

mod engine;
mod parser;
mod cli;
mod error;

fn main() {
    let config = cli::parse();
    let yaml = fs::read_to_string(config.path);
    let yaml = match yaml {
        Ok(s) => s,
        Err(err) => {
            println!("Parsing error: {}", err);
            std::process::exit(1);
        }
    };
    let test = match parser::parse(&yaml) {
        Ok(test) => test,
        Err(err) => {
            println!("Validation error: {}", err);
            std::process::exit(1);
        }
    };

    let handler = engine::run_test(test);
    handler.print();
    if handler.has_error() {
        std::process::exit(1);
    } else {
        println!("{}", "Success".bold().green());
        std::process::exit(0);
    }
}

