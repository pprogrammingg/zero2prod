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
    // Not async, because we use connect_lazy to connect when needed
    let connection_pool = PgPool::connect_lazy(
        configuration
            .database
            .connection_string()
            .expose_secret(),
    )
    .expect("Failed to connect to Postgres.");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
