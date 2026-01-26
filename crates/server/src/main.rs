use std::env;

use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    http::Method,
    middleware::{Compress, Logger},
    web,
};

mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a valid number");

    log::info!("Starting server at http://{}:{}", host, port);

    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_methods([Method::POST, Method::GET]);

        App::new()
            // ~~~ Global Middleware
            .wrap(Logger::default())
            .wrap(Compress::default())
            .wrap(cors)
            // ~~~ Routes
            .service(routes::health)
            .service(
                web::scope("/template")
                    .service(routes::outline)
                    .service(routes::habit_tracker),
            )
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
