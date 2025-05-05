use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::fs;

#[derive(Parser)]
#[grammar = "pdx.pest"]
struct PdxParser;

pub struct UnparsedFile<'a> {
    pub filename: &'a str,
    pub unparsed: String,
}

fn latin1_to_utf8(input: &[u8]) -> String {
    input.iter().map(|&b| b as char).collect()
}

impl UnparsedFile<'_> {
    pub fn new(filename: &str) -> UnparsedFile {
        UnparsedFile {
            filename,
            unparsed: latin1_to_utf8(
                &fs::read(filename)
                    .unwrap_or_else(|e| panic!("Error: {} could not be read: {}", filename, e)),
            ),
        }
    }

    pub fn process(&self) -> ParsedFile {
        let filename = self.filename;
        let parsed = PdxParser::parse(Rule::file, &self.unparsed)
            .unwrap_or_else(|e| panic!("Error: {} could not be parsed: {}", filename, e))
            .next()
            .unwrap();
        ParsedFile { filename, parsed }
    }
}

pub struct ParsedFile<'a> {
    pub filename: &'a str,
    pub parsed: Pair<'a, Rule>,
}
