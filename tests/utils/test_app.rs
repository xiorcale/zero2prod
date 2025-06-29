use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    config::{Config, Database},
    startup,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn run() -> TestApp {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        let mut configuration = Config::load();
        configuration.database.name = Uuid::new_v4().to_string();

        let connection_pool = TestApp::configure_database(&configuration.database).await;

        let server =
            startup::run(listener, connection_pool.clone()).expect("Failed to bind address");
        let _ = tokio::spawn(server);

        TestApp {
            address,
            db_pool: connection_pool,
        }
    }

    async fn configure_database(database: &Database) -> PgPool {
        let pool = database.pool_without_db().await;

        pool
            .execute(format!(r#"CREATE DATABASE "{}";"#, database.name).as_str())
            .await
            .expect("Failed to create database.");

        let pool = database.pool().await;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to migrate database");

        pool
    }
}
