use std::net::SocketAddr;
use std::ops::Add;
use std::time::{Duration, SystemTime};

use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::{routing::post, Extension, Json, Router, Server};
use clap::Parser;
use log::{info, trace, warn};
use serde::Deserialize;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

use rusdirect::cleaner::{ExcessCleaner, ExpirationCleaner};
use rusdirect::error::Error;
use rusdirect::exit_error;
use rusdirect::include_query;
use rusdirect::options::Options;

#[tokio::main]
async fn main() {
    let options = Options::parse();
    env_logger::Builder::new()
        .filter_level(options.log_level)
        .init();

    let pool = SqlitePoolOptions::new()
        .connect_with(
            SqliteConnectOptions::new()
                .filename("rusdirect.db")
                .create_if_missing(true)
                .busy_timeout(Duration::from_secs(30)),
        )
        .await
        .unwrap_or_else(|err| exit_error!("database pool creation failure: {}", err));
    sqlx::query(include_query!("migration"))
        .execute(&pool)
        .await
        .unwrap_or_else(|err| exit_error!("database migration failure: {}", err));

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
        .layer(Extension(ExcessCleaner { limit: 5 }));
    Server::bind(&SocketAddr::from(([0, 0, 0, 0], 8080)))
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|err| exit_error!("http server start failure: {}", err));
}

#[derive(Deserialize, Debug)]
struct RedirectionRequest {
    path: String,
    target: String,
}

async fn insert(
    Extension(pool): Extension<SqlitePool>,
    Extension(duration): Extension<Duration>,
    Extension(cleaner): Extension<ExcessCleaner>,
    Json(req): Json<RedirectionRequest>,
) -> Result<StatusCode, Error> {
    sqlx::query(include_query!("insert_redirection"))
        .bind(&req.path)
        .bind(&req.target)
        .bind(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_err(|err| Error::ExpirationCalculationFailure(err))?
                .add(duration)
                .as_secs() as i64,
        )
        .execute(&pool)
        .await
        .map_err(|err| {
            warn!("database access failure: {}", err);
            Error::DatabaseConnectionFailure(err)
        })?;
    info!(
        "redirection from \"{}\" to \"{}\" added",
        &req.path, &req.target
    );

    cleaner.run(&pool).await;

    Ok(StatusCode::CREATED)
}

async fn redirect(
    Extension(pool): Extension<SqlitePool>,
    uri: Uri,
) -> Result<Result<Redirect, StatusCode>, Error> {
    let path = uri.path().trim_start_matches('/');
    let target = sqlx::query_as::<_, (String,)>(include_query!("get_redirection"))
        .bind(path)
        .fetch_optional(&pool)
        .await
        .map_err(|err| {
            warn!("database access failure: {}", err);
            Error::DatabaseConnectionFailure(err)
        })?;

    match target {
        Some(target) => {
            info!("redirection from \"{}\" to \"{}\" proceed", path, target.0);
            Ok(Ok(Redirect::temporary(&target.0)))
        }
        None => {
            trace!("redirection request from \"{}\" not found", path);
            Ok(Err(StatusCode::NOT_FOUND))
        }
    }
}
