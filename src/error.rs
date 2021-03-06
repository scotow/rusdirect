use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("database connection failure: {0}")]
    DatabaseConnectionFailure(sqlx::Error),
    #[error("expiration date calculation failure: {0}")]
    ExpirationCalculationFailure(std::time::SystemTimeError),
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::DatabaseConnectionFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::ExpirationCalculationFailure(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (self.status_code(), self.to_string()).into_response()
    }
}
