use std::net::SocketAddr;
use std::ops::Add;
use std::time::{Duration, SystemTime};

use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::{routing::post, Extension, Json, Router, Server};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::cleaner::{ExcessCleaner, ExpirationCleaner};
use crate::error::Error;

mod cleaner;
mod error;
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

    let cleaner_pool = pool.clone();
    tokio::task::spawn(async move {
        let cleaner = ExpirationCleaner {
            interval: Duration::from_secs(5 * 60),
        };
        cleaner.run(cleaner_pool).await;
    });

    let app = Router::new()
        .route("/*id", post(insert).get(redirect))
        .layer(Extension(pool))
        .layer(Extension(Duration::from_secs(30)))
        .layer(Extension(ExcessCleaner { limit: 16384 }));
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
    Extension(duration): Extension<Duration>,
    Extension(cleaner): Extension<ExcessCleaner>,
    Json(req): Json<RedirectionRequest>,
) -> Result<StatusCode, Error> {
    sqlx::query(include_query!("insert_redirection"))
        .bind(&req.id)
        .bind(&req.target)
        .bind(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .add(duration)
                .as_secs() as i64,
        )
        .execute(&pool)
        .await
        .map_err(|err| Error::DatabaseConnectionFailure(err))?;
    cleaner.run(&pool).await;

    Ok(StatusCode::CREATED)
}

async fn redirect(
    Extension(pool): Extension<SqlitePool>,
    uri: Uri,
) -> Result<Result<Redirect, StatusCode>, Error> {
    let target = sqlx::query_as::<_, (String,)>(include_query!("get_redirection"))
        .bind(uri.path().strip_prefix('/'))
        .fetch_optional(&pool)
        .await
        .map_err(|err| Error::DatabaseConnectionFailure(err))?;

    match target {
        Some(target) => Ok(Ok(Redirect::temporary(&target.0))),
        None => Ok(Err(StatusCode::NOT_FOUND)),
    }
}
