use axum::body::{Bytes, HttpBody};
use axum::extract::{FromRequest, RequestParts};
use axum::http::{header, StatusCode};
use axum::{async_trait, BoxError};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RedirectionRequest {
    pub path: Option<String>,
    pub target: String,
}

#[async_trait]
impl<B> FromRequest<B> for RedirectionRequest
where
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(req)
            .await
            .map_err(|_| StatusCode::BAD_REQUEST)?;

        Ok(
            match req
                .headers()
                .get(header::CONTENT_TYPE)
                .map(|h| h.to_str().ok())
                .flatten()
                .ok_or_else(|| StatusCode::BAD_REQUEST)?
            {
                "application/json" => {
                    serde_json::from_slice(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?
                }
                "application/x-www-form-urlencoded" => {
                    serde_urlencoded::from_bytes(&bytes).map_err(|_| StatusCode::BAD_REQUEST)?
                }
                _ => return Err(StatusCode::BAD_REQUEST),
            },
        )
    }
}
