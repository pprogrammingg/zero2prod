//! src/routes/subscriptions_confirm.rs
use actix_web::{
    web,
    HttpResponse,
};
use tracing::info;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
pub async fn confirm(parameters: web::Query<Parameters>) -> HttpResponse {
    info!("Params are {}", parameters.subscription_token);
    HttpResponse::Ok().finish()
}
