use std::ops::Add;
use std::time::{Duration, SystemTime};

use axum::extract::ContentLengthLimit;
use axum::http::{StatusCode, Uri};
use axum::response::Redirect;
use axum::Extension;
use log::{info, trace, warn};
use sqlx::SqlitePool;

use crate::cleaner::ExcessCleaner;
use crate::error::Error;
use crate::include_query;
use crate::redirection::RedirectionRequest;

pub async fn insert(
    Extension(pool): Extension<SqlitePool>,
    Extension(duration): Extension<Duration>,
    Extension(cleaner): Extension<ExcessCleaner>,
    req: ContentLengthLimit<RedirectionRequest, 2048>,
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

pub async fn redirect(
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
