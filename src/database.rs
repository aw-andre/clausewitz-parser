use super::parser::*;
use sqlx::{Pool, Postgres};

pub async fn initialize(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query!("DROP SEQUENCE IF EXISTS euiv_childseq")
        .execute(pool)
        .await?;
    sqlx::query!("CREATE SEQUENCE euiv_childseq")
        .execute(pool)
        .await?;
    sqlx::query!("DROP TABLE IF EXISTS euiv")
        .execute(pool)
        .await?;
    sqlx::query!(
        "
            CREATE TABLE euiv (
                primary_id SERIAL PRIMARY KEY,
                group_id INT REFERENCES euiv(child_id) ON DELETE RESTRICT ON UPDATE CASCADE,
                key VARCHAR(255) NOT NULL,
                value VARCHAR(255),
                parent_id INT REFERENCES euiv(primary_id) ON DELETE RESTRICT ON UPDATE CASCADE,
                child_id INT UNIQUE DEFAULT nextval('euiv_childseq')
            )
        ",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub fn insert_file(file: String, pool: &Pool<Postgres>) {
    // let parsedfile = UnparsedFile::new(&file).process();
}
