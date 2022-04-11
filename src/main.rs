use axum::http::{StatusCode, Uri};
use axum::response::{IntoResponse, Redirect};
use axum::{routing::post, Extension, Json, Router, Server};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime};

mod misc;
mod query;

#[tokio::main]
async fn main() {
    let pool = SqlitePoolOptions::new()
        // .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename("rusdirect.db")
                .create_if_missing(true)
                .busy_timeout(Duration::from_secs(30)),
        )
        .await
        .unwrap_or_else(|err| exit_error!("Cannot create database pool: {}", err));
    sqlx::query(include_query!("migration"))
        .execute(&pool)
        .await
        .unwrap_or_else(|err| exit_error!("Cannot run migration query: {}", err));

    let app = Router::new()
        .route("/*id", post(insert).get(redirect))
        .layer(Extension(pool));
    Server::bind(&SocketAddr::from(([0, 0, 0, 0], 8080)))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Debug)]
struct RedirectionRequest {
    id: String,
    target: String,
}

async fn insert(
    Extension(pool): Extension<SqlitePool>,
    Json(req): Json<RedirectionRequest>,
) -> impl IntoResponse {
    sqlx::query(include_query!("insert_redirection"))
        .bind(&req.id)
        .bind(&req.target)
        .bind(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        )
        .execute(&pool)
        .await
        .unwrap();
    StatusCode::OK
}

async fn redirect(Extension(pool): Extension<SqlitePool>, uri: Uri) -> impl IntoResponse {
    let target = sqlx::query_as::<_, (String,)>(include_query!("get_redirection"))
        .bind(uri.path().strip_prefix('/'))
        .fetch_optional(&pool)
        .await
        .unwrap();

    match target {
        Some(target) => Redirect::temporary(&target.0).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}
