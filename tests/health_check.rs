use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    startup::run,
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // given
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    // when
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // then
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // given
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // when
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // then
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT name, email FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.name, "le guin");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
}

#[tokio::test]
async fn subscribe_reurns_a_400_when_data_is_missing() {
    // given
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // when
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // then
        assert_eq!(
            400,
            response.status().as_u16(),
            // additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        )
    }
}
