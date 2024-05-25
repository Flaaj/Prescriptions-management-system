use chrono::{DateTime, Utc};
use rocket::http;

// TODO: add global responder that returns this struct instead of just error messages in case of error response
#[allow(dead_code)]
pub struct ApiError {
    path: String,
    method: http::Method,
    status: http::Status,
    message: String,
    timestamp: DateTime<Utc>,
}
