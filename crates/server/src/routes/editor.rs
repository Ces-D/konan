use crate::routes::Message;
use actix_web::{Result, post, web::Json};
use designs::tiptap_adapter::TipTapJsonAdapter;
use rongta::PrintBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PrintableMessage {
    content: tiptap::JSONContent,
    rows: Option<u32>,
}

#[post("/message")]
async fn message(Json(form): Json<PrintableMessage>) -> Result<Json<Message>> {
    let builder = PrintBuilder::new(true);
    let adapter = TipTapJsonAdapter::new(builder);
    adapter
        .print(form.content, form.rows)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to print"))?;
    Ok(Json(Message::default()))
}
