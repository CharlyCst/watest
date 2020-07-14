use std::fs;

mod engine;
mod parser;

fn main() {
    let yaml = fs::read_to_string("wasm/spec.yaml");
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
    engine::run_test(test);
}
