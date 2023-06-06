//MODS
pub mod models;
pub mod utils;
pub mod views;
pub mod shared_state;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use models::db::create_connection;
use shared_state::SharedState;
use tonic::transport::Server;
use utils::jwt::generate_key;
use views::{refresh::refresh_tokens, logout::logout,login::login, verify_token::{TokenService, verify::verify_token_server::VerifyTokenServer}};

use crate::views::verify_user::verify_user_link;


#[tokio::main]
async fn main() {

    // GENERATE SIGNING KEY
    let key = generate_key().await;
    let key_grpc = key.clone();

    // CREATE A MANAGED STATE
    let mongo_client = create_connection().await;
    let shared_state = SharedState::new(mongo_client, key);


    let actix_server = tokio::spawn(
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(shared_state.clone()))
                .service(health_check)
                .service(logout)
                .service(refresh_tokens)
                .service(login)
                .service(verify_user_link)
        })
        .bind(("0.0.0.0", 8050))
        .expect("msg")
        .run()
    );

    let grpc_task = tokio::spawn(async move {
        let mongo_client = create_connection().await;
        let address = "[::1]:8080".parse().unwrap();
        let voting_service = TokenService::new(SharedState::new(mongo_client,key_grpc));

        Server::builder()
            .add_service(VerifyTokenServer::new(voting_service))
            .serve(address)
            .await
            .expect("Failed to start gRPC server");
    });

    tokio::select! {
        _ = actix_server => {} 
        _ = grpc_task => {} 
    }




}


#[get("/")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("Ok")
}