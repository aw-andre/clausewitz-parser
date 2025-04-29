use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Game name
    #[arg(long, required = true)]
    pub game: String,

    /// Initialize database
    #[arg(long, required_unless_present_any = ["add", "delete", "finalize"])]
    pub initialize: bool,

    /// Add files to database
    #[arg(long, required_unless_present_any = ["initialize", "delete", "finalize"])]
    pub add: bool,

    /// Files to parse
    #[arg(long, required_if_eq("add", "true"), conflicts_with_all = ["initialize", "delete", "finalize"], num_args = 1, value_parser)]
    pub files: Vec<String>,

    /// Delete database
    #[arg(long, required_unless_present_any = ["initialize", "add", "finalize"])]
    pub delete: bool,

    /// Finalize database
    #[arg(long, required_unless_present_any = ["initialize", "add", "delete"])]
    pub finalize: bool,
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
