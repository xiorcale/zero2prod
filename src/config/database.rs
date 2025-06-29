use sqlx::{PgPool, Pool, Postgres};

#[derive(serde::Deserialize)]
pub struct Database {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub name: String,
}

impl Database {
    pub async fn pool(&self) -> Pool<Postgres> {
        PgPool::connect(&self.connection_string())
            .await
            .expect("Failed to get pool")
    }

    pub async fn pool_without_db(&self) -> Pool<Postgres> {
        PgPool::connect(&self.connection_string_without_db())
            .await
            .expect("Failed to get pool without db")
    }

    fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }
}
