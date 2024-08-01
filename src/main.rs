//! main.rs

use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{
        get_subscriber,
        init_subscriber,
    },
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Redirect all `log`'s events to our subscriber
    // todo: why Actix-Web does not produce a log when endpoint first invoked and
    // todo: only receiving actix-web log when end point exits
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(
        &configuration
            .database
            .connection_string()
            .expose_secret(),
    )
    .await
    .expect("Failed to connect to Postgres.");

    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
