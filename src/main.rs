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
use pages::lucasodon;

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
            .service(lucasodon::render)
            .service(lucasodon::login)
            .service(lucasodon::logout)
            .service(lucasodon::list)
            .service(lucasodon::create)
            .service(lucasodon::update)
            .service(lucasodon::delete)
            .service(lucasodon::categorias_list)
            .service(lucasodon::categoria_create)
            .service(lucasodon::categoria_update)
            .service(lucasodon::categoria_delete)
            .service(lucasodon::recorrentes_list)
            .service(lucasodon::recorrente_create)
            .service(lucasodon::recorrente_update)
            .service(lucasodon::recorrente_delete)
            .service(lucasodon::despesas_list)
            .service(lucasodon::despesa_create)
            .service(lucasodon::despesa_update)
            .service(lucasodon::despesa_delete)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
