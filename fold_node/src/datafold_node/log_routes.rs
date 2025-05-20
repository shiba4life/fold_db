use actix_web::{web, HttpResponse, Responder};
use futures_util::stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use crate::web_logger;

pub async fn list_logs() -> impl Responder {
    HttpResponse::Ok().json(web_logger::get_logs())
}

pub async fn stream_logs() -> impl Responder {
    let rx = match web_logger::subscribe() {
        Some(r) => r,
        None => return HttpResponse::InternalServerError().finish(),
    };
    let stream = BroadcastStream::new(rx).filter_map(|msg| async move {
        match msg {
            Ok(line) => Some(Ok(web::Bytes::from(format!("data: {}\n\n", line)))),
            Err(_) => None,
        }
    });
    HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .streaming(stream)
}
