use std::net::SocketAddr;
use std::time::Duration;

use axum::{routing::post, Extension, Router, Server};
use clap::Parser;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use rusdirect::cleaner::{ExcessCleaner, ExpirationCleaner};
use rusdirect::include_query;
use rusdirect::options::Options;
use rusdirect::{exit_error, handler};

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
                .busy_timeout(Duration::from_secs(15)),
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
            interval: options.clean_interval,
        };
        cleaner.run(cleaner_pool).await;
    });

    let app = Router::new()
        .route("/*id", post(handler::insert).get(handler::redirect))
        .layer(Extension(pool))
        .layer(Extension(options.expiration))
        .layer(Extension(ExcessCleaner {
            limit: options.list_size,
        }));
    Server::bind(&SocketAddr::from((options.address, options.port)))
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|err| exit_error!("http server start failure: {}", err));
}
