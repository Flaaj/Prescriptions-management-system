use okapi::openapi3::Responses;
use rocket::{get, http::Status, response::Responder, Request};
use rocket_okapi::{gen::OpenApiGenerator, openapi, response::OpenApiResponderInner, OpenApiError};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::application::api::utils::{error::ApiError, openapi_responses::get_openapi_responses};

fn example_username() -> &'static str {
    "Doctor_Doe-123"
}
fn example_password() -> &'static str {
    "eR4a3@!#g(1a"
}
fn example_name() -> &'static str {
    "John Doe"
}
fn example_email() -> &'static str {
    "john.doe@gmail.com"
}
fn example_phone_number() -> &'static str {
    "+48 123 456 789"
}
fn example_pesel_number() -> &'static str {
    "92022900002"
}
fn example_pwz_number() -> &'static str {
    "3123456"
}

#[derive(Deserialize, JsonSchema)]
pub struct RegisterDoctorDto {
    #[schemars(example = "example_username")]
    username: String,
    #[schemars(example = "example_password")]
    password: String,
    #[schemars(example = "example_email")]
    email: String,
    #[schemars(example = "example_phone_number")]
    phone_number: String,
    #[schemars(example = "example_name")]
    name: String,
    #[schemars(example = "example_pesel_number")]
    pesel_number: String,
    #[schemars(example = "example_pwz_number")]
    pwz_number: String,
}

pub struct AuthError;

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let message = "Unauthorized".into();
        let status = Status::Unauthorized;
        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for AuthError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![(
            "401",
            "Returned when the user is not authorized to access the resource.",
        )])
    }
}

#[openapi(tag = "Auth")]
#[get("/test-collection/endpoint-that-requires-authorization-as-doctor")]
pub async fn endpoint_that_requires_authorization_as_doctor() -> Result<String, AuthError> {
    Ok("You are authorized as a doctor".to_string())
}

#[cfg(test)]
mod tests {
    use rocket::{
        http::{ContentType, Header, Status},
        local::asynchronous::Client,
        routes,
    };

    async fn create_api_client() -> Client {
        let routes = routes![];

        let rocket = rocket::build().mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    // #[tokio::test]
    // async fn registers_doctor_account() {
    //     let client = create_api_client().await;

    //     let response = client
    //         .post("/auth/register/doctor")
    //         .header(ContentType::JSON)
    //         .body(
    //             r#"{
    //             "username": "doctor",
    //             "password": "password",
    //             "email": "doctor_john_doe@gmail.com",
    //             "phone_number": "123456789",
    //             "name": "John Doe",
    //             "pesel_number": "12345678901",
    //             "pwz_number": "123456789"
    //         }"#,
    //         )
    //         .dispatch()
    //         .await;

    //     assert_eq!(response.status(), Status::Ok);
    // }

    // #[tokio::test]
    // async fn test_authorization() {
    //     let client = create_api_client().await;

    //     let response = client
    //         .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
    //         .dispatch()
    //         .await;

    //     assert_eq!(response.status(), Status::Unauthorized);

    //     let login_response = client
    //         .post("/auth/login/doctor")
    //         .header(ContentType::JSON)
    //         .body(r#"{"username": "doctor", "password": "password"}"#)
    //         .dispatch()
    //         .await;

    //     assert_eq!(login_response.status(), Status::Ok);

    //     let token = login_response.into_string().await.unwrap();

    //     let response = client
    //         .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
    //         .header(Header::new("Authorization", format!("Bearer {}", token)))
    //         .dispatch()
    //         .await;

    //     assert_eq!(response.status(), Status::Ok);
    // }
}
