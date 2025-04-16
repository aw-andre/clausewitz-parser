use super::parser::*;
use sqlx::{Pool, Postgres};

pub async fn initialize(pool: &Pool<Postgres>) -> Result<(), sqlx::Error> {
    sqlx::query!("DROP TABLE IF EXISTS euiv")
        .execute(pool)
        .await?;
    sqlx::query!(
        "
            CREATE TABLE euiv (
                primary_id INT PRIMARY KEY,
                group_id INT,
                key VARCHAR(255) NOT NULL,
                value VARCHAR(255),
                parent_id INT,
                child_id INT
            )
        ",
    )
    .execute(pool)
    .await?;
    sqlx::query!(
        "
            ALTER TABLE euiv
                ADD CONSTRAINT fk_parent FOREIGN KEY (parent_id) REFERENCES euiv(primary_id) ON DELETE RESTRICT ON UPDATE CASCADE
        ",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub fn insert_file(file: String, pool: &Pool<Postgres>) {
    // let parsedfile = UnparsedFile::new(&file).process();
}
