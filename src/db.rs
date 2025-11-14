use std::path::Path;

use sqlx::{Sqlite, migrate::Migrator};

static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

#[derive(Debug, Clone)]
pub struct Db(sqlx::Pool<Sqlite>);

#[derive(Debug, sqlx::FromRow)]
pub struct MappingValue {
    pub merchant_private_key: String,
    pub token: String,
}

impl Db {
    pub async fn connect() -> sqlx::Result<Self> {
        let database_url = std::env::var("DATABASE_URL").expect("database url to be defined");
        tracing::debug!(%database_url);
        let path = Path::new(
            database_url
                .strip_prefix("sqlite://")
                .expect("url sqlite prefix"),
        );
        {
            if let Some(parent) = path.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .expect("directory is initialized");
            }
            tokio::fs::OpenOptions::new()
                .write(true)
                .truncate(false)
                .create(true)
                .open(path)
                .await
                .expect("open database file");
        }
        let pool = sqlx::Pool::connect(&database_url).await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self(pool))
    }

    pub async fn insert_mapping(
        &self,
        merchant_private_key: &str,
        token: &str,
        gateway_token: &str,
    ) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO gateway_id_mapping (token, merchant_private_key, gateway_id) VALUES (?, ?, ?)")
            .bind(token)
            .bind(merchant_private_key)
            .bind(gateway_token)
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn get_mapping(&self, gateway_token: &str) -> sqlx::Result<Option<MappingValue>> {
        sqlx::query_as(
            "SELECT merchant_private_key, token FROM gateway_id_mapping WHERE gateway_id = ?",
        )
        .bind(gateway_token)
        .fetch_optional(&self.0)
        .await
    }
}
