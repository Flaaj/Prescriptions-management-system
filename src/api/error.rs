use rocket::http;

pub struct ApiError {
    path: String,
    method: http::Method,
    status: u16,
    message: String,
    timestamp: String,
}
