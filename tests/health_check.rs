//! tests/health_check.rs
use std::net::TcpListener;

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
};

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
        db_pool: _db_pool,
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
    let TestApp {
        address,
        db_pool: _db_pool,
    } = spawn_app().await;
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration
        .database
        .connection_string();
    // The `Connection` trait MUST be in scope for us to invoke
    // `PgConnection::connect` - it is not an inherent method of the struct!
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres.");

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
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "pprog@gmail.com");
    assert_eq!(saved.name, "de Pprog");
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
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    //configuration.database;
    // .database_name = Uuid::new_v4().to_string();

    let db_pool = PgPool::connect(
        &configuration
            .database
            .connection_string(),
    )
    .await
    .expect("Failed to connect to Postgres."); //configure_database(&configuration.database).await;

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // We retrieve the port assigned to us by the OS
    let port = listener
        .local_addr()
        .unwrap()
        .port();
    let server = run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    // Query to get the current database name
    // let row: (String,) = sqlx::query_as("SELECT current_database()")
    //     .fetch_one(&mut connection)
    //     .await
    //     .expect("failed to get database name");
    //
    // // The database name is in the first tuple element
    // let db_name = row.0;
    //
    // println!("Connected to database: {}", db_name);

    connection_pool
}
// no clean up is performed intentionally, if many empty dbs become a performance issue, we can
// simply restart the db
