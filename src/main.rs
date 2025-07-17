use actix_web::{App, HttpServer};
use actix_files::Files;

mod components;
mod pages;

use pages::home;
use pages::competitive_programming_classes;
use pages::steganography;
use pages::graduation_thesis;
use pages::who_chat;
use pages::predict_codeforces_rating;
use pages::wordle_solver;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(Files::new("/static", "static"))
            .service(home::render)
            .service(competitive_programming_classes::render)
            .service(steganography::render)
            .service(graduation_thesis::render)
            .service(who_chat::get_chat)
            .service(who_chat::post_chat)
            .service(who_chat::delete_chat)
            .service(who_chat::render)
            .service(predict_codeforces_rating::render)
            .service(predict_codeforces_rating::predict_rating)
            .service(wordle_solver::render)
            .service(wordle_solver::solve_wordle)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
