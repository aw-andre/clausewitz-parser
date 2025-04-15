use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Database URL
    #[arg(short, long, value_parser)]
    pub database_url: String,

    /// Files to parse
    #[arg(num_args = 1, value_parser)]
    pub files: Vec<String>,
}

impl Cli {
    pub fn validate(&self) {
        for file in &self.files {
            match fs::metadata(file) {
                Ok(_) => (),
                Err(_) => panic!("Error: {} is not a valid file.", file),
            }
        }
    }
}
