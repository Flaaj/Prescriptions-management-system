use std::net::Ipv4Addr;

use okapi::openapi3::Responses;
use rocket::{
    get,
    http::Status,
    post,
    request::{FromRequest, Outcome},
    response::Responder,
    serde::json::Json,
    Request,
};
use rocket_okapi::{gen::OpenApiGenerator, openapi, response::OpenApiResponderInner, OpenApiError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::{
        api::utils::{error::ApiError, openapi_responses::get_openapi_responses},
        authentication::{
            models::UserRole,
            repository::CreateUserRepositoryError,
            service::{AuthenticationWithCredentialsError, CreateUserError},
        },
        sessions::models::Session,
    },
    domain::doctors::{repository::CreateDoctorRepositoryError, service::CreateDoctorError},
    Context, Ctx,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LoginError {
    Unauthorized,
    InvalidToken,
    InternalError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = LoginError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(header) => {
                let (_, session_token) = header.split_at(7);
                let session_id = Uuid::parse_str(session_token);
                if session_id.is_err() {
                    return Outcome::Error((Status::Unauthorized, LoginError::InvalidToken));
                }
                let session_id = session_id.unwrap();

                let config = req.rocket().state::<Context>().unwrap();

                let session = config
                    .sessions_service
                    .get_session_by_id(session_id)
                    .await
                    .map_err(|_| (Status::InternalServerError, LoginError::InternalError));

                match session {
                    Ok(session) => Outcome::Success(session),
                    Err(_) => Outcome::Error((Status::Unauthorized, LoginError::Unauthorized)),
                }
            }
            None => Outcome::Error((Status::Unauthorized, LoginError::Unauthorized)),
        }
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

pub enum RegisterDoctorError {
    DoctorsError(CreateDoctorError),
    UsersError(CreateUserError),
}

impl<'r> Responder<'r, 'static> for RegisterDoctorError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DoctorsError(doctors_err) => match doctors_err {
                CreateDoctorError::DomainError(err) => (err, Status::UnprocessableEntity),
                CreateDoctorError::RepositoryError(err) => {
                    let message = err.to_string();
                    let status = match err {
                        CreateDoctorRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                        CreateDoctorRepositoryError::DuplicatedPwzNumber => Status::Conflict,
                        CreateDoctorRepositoryError::DatabaseError(_) => {
                            Status::InternalServerError
                        }
                    };
                    (message, status)
                }
            },
            Self::UsersError(users_err) => match users_err {
                CreateUserError::DomainError(err) => (err, Status::UnprocessableEntity),
                CreateUserError::RepositoryError(err) => {
                    let message = err.to_string();
                    let status = match err {
                        CreateUserRepositoryError::DatabaseError(_) => Status::InternalServerError,
                    };
                    (message, status)
                }
            },
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for RegisterDoctorError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            // TODO: Add all responses
            (
                "401",
                "Returned when the user is not authorized to access the resource.",
            ),
        ])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuccessResponse {
    success: bool,
}

#[openapi(tag = "Auth")]
#[post("/auth/register/doctor", data = "<dto>", format = "application/json")]
pub async fn register_doctor(
    ctx: &Ctx,
    dto: Json<RegisterDoctorDto>,
) -> Result<Json<SuccessResponse>, RegisterDoctorError> {
    let created_doctor = ctx
        .doctors_service
        .create_doctor(dto.0.name, dto.0.pesel_number, dto.0.pwz_number)
        .await
        .map_err(|err| RegisterDoctorError::DoctorsError(err))?;

    ctx.authentication_service
        .register_user(
            dto.0.username,
            dto.0.password,
            dto.0.email,
            dto.0.phone_number,
            UserRole::Doctor,
            Some(created_doctor.id),
            None,
        )
        .await
        .map_err(|err| RegisterDoctorError::UsersError(err))?;

    Ok(Json(SuccessResponse { success: true }))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SessionTokenResponse {
    token: String,
}

impl<'r> Responder<'r, 'static> for AuthenticationWithCredentialsError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let message = self.to_string();
        let status = Status::Unauthorized;
        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for AuthenticationWithCredentialsError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![("401", "Ivalid credentials")])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LoginDoctorDto {
    username: String,
    password: String,
}

#[openapi(tag = "Auth")]
#[post("/auth/login/doctor", data = "<dto>", format = "application/json")]
pub async fn login_doctor(
    ctx: &Ctx,
    dto: Json<LoginDoctorDto>,
) -> Result<Json<SessionTokenResponse>, AuthenticationWithCredentialsError> {
    let user = ctx
        .authentication_service
        .authenticate_with_credentials(dto.0.username, dto.0.password)
        .await
        .map_err(|_| AuthenticationWithCredentialsError::InvalidCredentials)?;

    let session = ctx
        .sessions_service
        .create_session(
            user.id,
            user.doctor.map(|d| d.id),
            user.pharmacist.map(|p| p.id),
            Ipv4Addr::new(127, 0, 0, 1).into(),
            "".to_string(),
        )
        .await
        .unwrap();

    Ok(Json(SessionTokenResponse {
        token: session.id.to_string(),
    }))
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

#[get("/test-collection/endpoint-that-requires-authorization-as-doctor")]
pub async fn endpoint_that_requires_authorization_as_doctor(
    session: Session,
) -> Result<String, AuthError> {
    Ok("You are authorized as a doctor".to_string())
}

#[cfg(test)]
mod tests {
    use rocket::{
        http::{ContentType, Header, Status},
        local::asynchronous::Client,
        routes,
    };

    use super::SessionTokenResponse;
    use crate::application::api::controllers::fake_api_context::create_api_context;

    async fn create_api_client() -> Client {
        let context = create_api_context();

        let routes = routes![
            super::endpoint_that_requires_authorization_as_doctor,
            super::register_doctor,
            super::login_doctor
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    async fn test_doctor_auth() {
        let client = create_api_client().await;

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .post("/auth/register/doctor")
            .header(ContentType::JSON)
            .body(
                r#"{
                    "username": "doctor",
                    "password": "password123",
                    "email": "doctor_john_doe@gmail.com",
                    "phone_number": "123456789",
                    "name": "John Doe",
                    "pesel_number": "99031301347",
                    "pwz_number": "3123456"
                }"#,
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .post("/auth/login/doctor")
            .header(ContentType::JSON)
            .body(
                r#"{
                    "username": "doctor",
                    "password": "password123"
                }"#,
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
        let token = response
            .into_json::<SessionTokenResponse>()
            .await
            .unwrap()
            .token;

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }
}
