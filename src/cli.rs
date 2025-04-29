use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Initialize database
    #[arg(long, required_unless_present_any = ["add", "delete", "finalize"])]
    pub initialize: bool,

    /// Add files to database
    #[arg(long, required_unless_present_any = ["initialize", "delete", "finalize"])]
    pub add: bool,

    /// Delete database
    #[arg(long, required_unless_present_any = ["initialize", "add", "finalize"])]
    pub delete: bool,

    /// Finalize database
    #[arg(long, required_unless_present_any = ["initialize", "add", "delete"])]
    pub finalize: bool,

    /// Game name
    #[arg(long, conflicts_with_all = ["initialize", "delete", "finalize"], required_unless_present_any = ["initialize", "finalize"])]
    pub game: String,

    /// Files to parse
    #[arg(long, required_if_eq("add", "true"), conflicts_with_all = ["initialize", "delete", "finalize"], num_args = 1, value_parser)]
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
