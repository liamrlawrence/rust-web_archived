use std::sync::Arc;
use sqlx::postgres::PgPool;
use axum::{routing::get, response::Json, Router, extract::Query, response::IntoResponse, response::Html};
use serde::{Serialize, Deserialize};



#[derive(Debug, Deserialize)]
struct HelloParams {
	name: Option<String>,
}

async fn page_login(Query(params): Query<HelloParams>) -> impl IntoResponse {
    let name = params.name.as_deref().unwrap_or("World!");

    Html(format!("Hello, <b>{name}!</b>"))
}


pub fn router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/heartbeat", get(move || handler(pool)))
        .route("/login", get(page_login))
}


#[derive(Serialize)]
struct HandlerResponse {
    message: i32
}

async fn handler(pool: Arc<PgPool>) -> Json<HandlerResponse> {
    let row: (i32,) = sqlx::query_as("SELECT prompt_tokens FROM ai_bills;")
        .fetch_one(&*pool)
        .await
        .unwrap();

    let data = HandlerResponse {
        message: row.0
    };

    Json(data)
}

