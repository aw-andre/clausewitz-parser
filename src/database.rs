use super::ParsedFile;
use sqlx::{Pool, Postgres};

pub fn insert_file(parsedfile: ParsedFile, pool: &Pool<Postgres>) {}
