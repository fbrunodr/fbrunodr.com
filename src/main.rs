use actix_web::{App, HttpServer};
use actix_files::Files;

mod components;
mod pages;

use pages::home;
use pages::competitive_programming_classes;
use pages::steganography;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "static"))
            .service(home::render)
            .service(competitive_programming_classes::render)
            .service(steganography::render)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
