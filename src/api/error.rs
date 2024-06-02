use chrono::Utc;
use rocket::{
    http::{self, ContentType},
    Request, Response,
};

pub struct ApiError {
    pub message: String,
    pub path: String,
    pub status: http::Status,
    pub method: http::Method,
    pub timestamp_ms: i64,
}

impl ApiError {
    pub fn new(message: String, path: String, status: http::Status, method: http::Method) -> Self {
        Self {
            message,
            path,
            status,
            method,
            timestamp_ms: Utc::now().timestamp_millis(),
        }
    }

    fn to_json_string(&self) -> String {
        format!(
            r#"{{"message":"{}","path":"{}","status":{},"method":"{}","timestamp_ms":{}}}"#,
            self.message, self.path, self.status.code, self.method, self.timestamp_ms
        )
    }

    pub fn build_rocket_response<'r>(
        req: &'r Request<'_>,
        message: String,
        status: http::Status,
    ) -> rocket::response::Result<'static> {
        let path = req.uri().path().to_string();
        let method = req.method();

        let body = ApiError::new(message, path, status, method).to_json_string();

        Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(ContentType::JSON)
            .status(status)
            .ok()
    }
}
