use sqlx::Sqlite;

#[derive(Debug, Clone)]
pub struct Db(sqlx::Pool<Sqlite>);

#[derive(Debug)]
pub struct MappingValue {
    pub merchant_private_key: String,
    pub token: String,
}

impl Db {
    pub async fn connect() -> sqlx::Result<Self> {
        sqlx::Pool::connect("sqlite://database.sqlite")
            .await
            .map(Self)
    }

    pub async fn insert_mapping(
        &self,
        merchant_private_key: &str,
        token: &str,
        gateway_token: &str,
    ) -> sqlx::Result<()> {
        sqlx::query!(
            "INSERT INTO gateway_id_mapping (token, merchant_private_key, gateway_id) VALUES (?, ?, ?)",
            token,
            merchant_private_key,
            gateway_token
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }

    pub async fn get_mapping(&self, gateway_token: &str) -> sqlx::Result<Option<MappingValue>> {
        sqlx::query_as!(
            MappingValue,
            "SELECT merchant_private_key, token FROM gateway_id_mapping WHERE gateway_id = ?",
            gateway_token
        )
        .fetch_optional(&self.0)
        .await
    }
}
