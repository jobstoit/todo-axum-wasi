mod users;
pub mod todos;
pub use users::User;
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct Repo {
    db_name: String,
    db: PgPool,
}

impl Repo {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let host = std::env::var("DB_HOST").unwrap_or("localhost".to_string());
        let port = std::env::var("DB_PORT").unwrap_or("5432".to_string());
        let user = std::env::var("DB_USER").unwrap_or("todo".to_string());
        let password = std::env::var("DB_PASSWORD").unwrap_or_default();
        let dbname = std::env::var("DB_NAME").unwrap_or("todo".to_string());
        let sslmode = std::env::var("DB_SSLMODE").unwrap_or("prefer".to_string());

        let db = PgPool::connect(
            format!("postgres://{user}:{password}@{host}:{port}/{dbname}?sslmode={sslmode}")
                .as_str(),
        )
        .await?;

        log::info!("initialising database...");
        sqlx::migrate!("./migrations").run(&db).await?;

        Ok(Self {
            db_name: dbname,
            db,
        })
    }

    pub async fn active_queries(&self) -> Result<i64, sqlx::Error> {
        let count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM pg_stat_activity WHERE datname = $1;")
                .bind(&self.db_name)
                .fetch_one(&self.db)
                .await?;

        Ok(count.0)
    }
}

/// DateTime is a helper function since the sqlx::types::chrono::NaiveDateTime for wasm
/// doesn't implement all the same traits as the main package including serialize
#[derive(Debug, serde::Serialize)]
pub struct DateTime(chrono::NaiveDateTime);

impl From<sqlx::types::chrono::NaiveDateTime> for DateTime {
    fn from(value: sqlx::types::chrono::NaiveDateTime) -> DateTime {
        DateTime(chrono::NaiveDateTime::from_timestamp_opt(value.timestamp(), 0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::Repo;

    #[tokio::test]
    async fn initialization() {
        Repo::new().await.unwrap();
    }
}
