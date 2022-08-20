use std::io;
use std::env;
use actix_web::*;
use sqlx::mysql::MySqlPoolOptions;
use dotenv::dotenv;
use crate::error::MyError::InvalidInput;
use crate::routers::{disguise_routes, vault_routes};
use crate::state::AppState;


#[path = "../routers.rs"]
mod routers;
#[path = "../handlers/mod.rs"]
mod handlers;
#[path = "../state.rs"]
mod state;
#[path = "../models/mod.rs"]
mod models;
#[path = "../dbaccess/mod.rs"]
mod dbaccess;
#[path = "../errors.rs"]
mod error;

#[actix_rt::main]
async fn main() -> io::Result<()> {
    //load the env variables
    dotenv().ok();
    //build the target database using the url in the .env
    //the target database is the database to disguise
    let target_database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set yet.");
    let target_db = MySqlPoolOptions::new().connect(&target_database_url).await.unwrap();
    //then build the vault database
    let vault_database_url = env::var("VAULT_DATABASE_URL").expect("VAULT_DATABASE_URL is not set yet.");
    let vault_db = MySqlPoolOptions::new().connect(&vault_database_url).await.unwrap();
    //put the target and vault database pool in the state
    let shared_data = web::Data::new(AppState {
        vault_db,
        target_db,
    });
    //set the app state and the invalid input
    let app = move || {
        App::new()

            .app_data(shared_data.clone())
            .app_data(web::JsonConfig::default().error_handler(|_err, _req| {
                InvalidInput("Please provide valid Json input".to_string()).into()
            }))
            .configure(disguise_routes)
            .configure(vault_routes)
    };
    println!("The server has been started.");
    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}