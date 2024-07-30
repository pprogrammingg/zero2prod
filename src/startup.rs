use std::net::TcpListener;

use actix_web::{
    dev::Server,
    middleware::Logger,
    web,
    App,
    HttpServer,
};
use sqlx::PgPool;

use crate::routes::{
    health_check::health_check,
    subscriptions::subscribe,
};

// Notice the different signature!
// We return `Server` on the happy path and we dropped the `async` keyword
// We have no .await call, so it is not needed anymore.
pub fn run(listener: TcpListener, db_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_pool = web::Data::new(db_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .app_data(db_pool.clone()) //used to hold state
    })
    .listen(listener)?
    .run();

    Ok(server)
}
