use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response}, body,
};
use tracing::warn;

pub type Result<T> = core::result::Result<T, CustError>;

#[derive(Debug, serde::Serialize, Clone)]
pub struct CustError {
    message: String,
    #[serde(skip)]
    status: StatusCode,
}

impl CustError {
    pub fn new(message: String, status: StatusCode) -> Self {
        Self { message, status }
    }
}

impl IntoResponse for CustError {
    fn into_response(self) -> axum::response::Response {
        warn!("Generating error: {}", self.message);
        let msg = serde_json::to_string(&self).unwrap();

        Response::builder()
            .status(self.status)
            .header("Content-Type", "application/json")
            .body(body::boxed(msg))
            .unwrap()
    }
}

impl From<sqlx::Error> for CustError {
    fn from(e: sqlx::Error) -> Self {
        Self {
            message: format!("Database error: {}", e),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<base64::DecodeError> for CustError {
    fn from(e: base64::DecodeError) -> Self {
        Self {
            message: format!("Parsing error: {}", e),
            status: StatusCode::BAD_REQUEST,
        }
    }
}

impl From<io::Error> for CustError {
    fn from(e: io::Error) -> Self {
        Self {
            message: format!("IO error: {}", e),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
