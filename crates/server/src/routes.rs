use actix_web::{HttpResponse, Responder, get};

mod history;
mod template;
use serde::Serialize;
pub use template::{habit_tracker, outline};

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[derive(Debug, Serialize)]
struct Message {
    message: String,
}
impl Default for Message {
    fn default() -> Self {
        Self {
            message: "Printed successfully!".to_string(),
        }
    }
}
