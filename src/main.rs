mod create_tables;
pub mod domain;

use create_tables::create_tables;
use sqlx::postgres::PgPoolOptions;

#[macro_use]
extern crate dotenv_codegen;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(dotenv!("DATABASE_URL"))
        .await?;

    create_tables(&pool, true).await?;

    Ok(())
}
