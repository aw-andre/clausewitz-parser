use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "euiv.pest"]
pub struct EuivParser;

fn main() {}
