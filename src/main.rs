use std::sync::Arc;
use std::env;
use log::LevelFilter;
use std::net::SocketAddr;
use axum::{
    Router,
    extract::Extension,
    response::IntoResponse,
    extract::Query,
    response::Html,
    routing::get,
    http::StatusCode,
    Json};
use sqlx::postgres::{PgPoolOptions};
use tera::Tera;
use serde::Deserialize;

mod routes;






async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Html("<h1>404 Not Found</h1><p>The requested resource could not be found on this server.</p>"))
}


#[tokio::main]
async fn main() {
    // Configure logging
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::builder()
        .filter(Some("http_log"), LevelFilter::Debug)
        .init();


    // Connect to the database
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&format!("postgres://{}:{}@{}:{}/{}",
            env::var("PSQL_DB_USERNAME").unwrap(),
            env::var("PSQL_DB_PASSWORD").unwrap(),
            env::var("PSQL_DB_HOST").unwrap(),
            env::var("PSQL_DB_PORT").unwrap(),
            env::var("PSQL_DB_DATABASE").unwrap()))
        .await
        .expect("Failed to connect to database");
    let shared_pool = Arc::new(pool);


    // Setup routes
    let app = Router::new()
        .nest("/api", routes::auth::router(shared_pool.clone()))
        .fallback(not_found);

 
    // Configure tera HTML template engine
    let tera = Tera::new("/app/templates/**/*").unwrap();

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Starting the server on {addr}...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

