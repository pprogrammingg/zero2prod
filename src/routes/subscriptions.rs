use actix_web::{
    web,
    HttpResponse,
};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// Let's start simple: we always return a 200 OK
pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
