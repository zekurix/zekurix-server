use anyhow::Result;
use sqlx::PgPool;

#[derive(Default)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn connect(&self) -> Result<(Self)> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:password@localhost/test")
            .await?;

        Ok(Self { pool })
    }
}
