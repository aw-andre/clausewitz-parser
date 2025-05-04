use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Initialize database
    #[arg(long, conflicts_with_all = &["game", "files"])]
    pub initialize: bool,

    /// Game name
    #[arg(long, required_if_eq("files", "true"))]
    pub game: Option<String>,

    /// Files to parse
    #[arg(long, required_if_eq("game", "true"), num_args = 1.., value_parser)]
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
