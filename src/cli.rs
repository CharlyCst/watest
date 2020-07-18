extern crate clap;

use clap::Clap;
use std::path::PathBuf;

/// A wasm test tool.
#[derive(Clap, Debug)]
#[clap(version = "0.1.0")]
pub struct Config {
    /// Test specification path.
    #[clap(default_value = ".", parse(from_os_str))]
    pub path: PathBuf,
}

/// Parse CLI args, may terminate the program.
pub fn parse() -> Config {
    Config::parse()
}
