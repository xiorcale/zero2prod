use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::{config::Config, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = Config::load();
    let pool = config.database.pool().await;

    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address)?;

    run(listener, pool)?.await
}
