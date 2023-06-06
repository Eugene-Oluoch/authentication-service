//MODS
pub mod models;
pub mod utils;
pub mod views;
pub mod shared_state;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use models::db::create_connection;
use shared_state::SharedState;
use utils::jwt::generate_key;
use views::{refresh::refresh_tokens, logout::logout,login::login};


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // GENERATE SIGNING KEY
    let key = generate_key().await;

    // CREATE A MANAGED STATE
    let mongo_client = create_connection().await;
    let shared_state = SharedState::new(mongo_client, key);


    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_state.clone()))
            .service(health_check)
            .service(logout)
            .service(refresh_tokens)
            .service(login)
    })
    .bind(("0.0.0.0", 8050))?
    .run()
    .await
}


#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}