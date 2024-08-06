//! tests/health_check.rs
use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{
    Connection,
    Executor,
    PgConnection,
    PgPool,
};
use uuid::Uuid;
use zero2prod::{
    configuration::{
        get_configuration,
        DatabaseSettings,
    },
    startup::run,
    telemetry::{
        get_subscriber,
        init_subscriber,
    },
};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

// `tokio::test` is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute. //
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<- name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let TestApp {
        address,
        db_pool: _,
    } = spawn_app().await;

    // We need to bring in `reqwest`
    // to perform HTTP requests against our application.
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a200_for_valid_form_data() {
    // Arrange
    let TestApp { address, db_pool } = spawn_app().await;

    let client = reqwest::Client::new();

    // Act
    let body = "name=de%20Pprog&email=pprog%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "pprog@gmail.com");
    assert_eq!(saved.name, "de Pprog");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_name_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let long_name = "A".repeat(257);

    let long_name_email = format!("name={}&email=%40gmail.com", long_name);

    let test_cases = vec![
        (
            "name=&email=ursula_le_guin%40gmail.com".to_owned(),
            "empty name".to_owned(),
        ),
        (long_name_email, "name too long".to_owned()),
        (
            "name=U(o){&email=definitely-not-an-email".to_owned(),
            "name has forbidden characters".to_owned(),
        ),
    ];
    for (body, description) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");
        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 BAD REQUEST when the payload was {}.",
            description
        );
    }
}
#[tokio::test]
async fn subscribe_returns_a400_when_data_is_missing() {
    // Arrange
    let TestApp {
        address,
        db_pool: _db_pool,
    } = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
async fn spawn_app() -> TestApp {
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    let mut configuration = get_configuration().expect("Failed to read configuration.");

    configuration
        .database
        .database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener
        .local_addr()
        .unwrap()
        .port();
    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
// no clean up is performed intentionally, if many empty dbs become a performance issue, we can
// simply restart the db
