use actix_web::{App, HttpServer};
use actix_files::Files;

mod components;
mod pages;

use pages::home;
use pages::competitive_programming_classes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "static").show_files_listing())
            .service(home::render)
            .service(competitive_programming_classes::render)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
