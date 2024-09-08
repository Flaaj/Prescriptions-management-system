use okapi::openapi3::Responses;
use rocket::{get, http::Status, post, response::Responder, serde::json::Json, Request};
use rocket_okapi::{gen::OpenApiGenerator, openapi, response::OpenApiResponderInner, OpenApiError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    application::{
        api::{
            guards::{
                authorization::{DoctorSession, PharmacistSession},
                client_request_info::ClientRequestInfo,
            },
            utils::{error::ApiError, openapi_responses::get_openapi_responses},
        },
        authentication::{
            entities::UserRole,
            repository::CreateUserRepositoryError,
            service::{AuthenticationWithCredentialsError, CreateUserError},
        },
        sessions::{
            entities::Session, repository::UpdateSessionRepositoryError,
            service::InvalidateSessionError,
        },
    },
    context::Ctx,
    domain::{
        doctors::{repository::CreateDoctorRepositoryError, service::CreateDoctorError},
        pharmacists::{
            repository::CreatePharmacistRepositoryError, service::CreatePharmacistError,
        },
    },
};

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
        // TODO: Add all responses
        get_openapi_responses(vec![])
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
pub struct RegisterPharmacistDto {
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
}

pub enum RegisterPharmacistError {
    PharmacistsError(CreatePharmacistError),
    UsersError(CreateUserError),
}

impl<'r> Responder<'r, 'static> for RegisterPharmacistError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::PharmacistsError(pharmacists_err) => match pharmacists_err {
                CreatePharmacistError::DomainError(err) => (err, Status::UnprocessableEntity),
                CreatePharmacistError::RepositoryError(err) => {
                    let message = err.to_string();
                    let status = match err {
                        CreatePharmacistRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                        CreatePharmacistRepositoryError::DatabaseError(_) => {
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

impl OpenApiResponderInner for RegisterPharmacistError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        // TODO: Add all responses
        get_openapi_responses(vec![])
    }
}

#[openapi(tag = "Auth")]
#[post(
    "/auth/register/pharmacist",
    data = "<dto>",
    format = "application/json"
)]
pub async fn register_pharmacist(
    ctx: &Ctx,
    dto: Json<RegisterPharmacistDto>,
) -> Result<Json<SuccessResponse>, RegisterPharmacistError> {
    let created_pharmacist = ctx
        .pharmacists_service
        .create_pharmacist(dto.0.name, dto.0.pesel_number)
        .await
        .map_err(|err| RegisterPharmacistError::PharmacistsError(err))?;

    ctx.authentication_service
        .register_user(
            dto.0.username,
            dto.0.password,
            dto.0.email,
            dto.0.phone_number,
            UserRole::Pharmacist,
            None,
            Some(created_pharmacist.id),
        )
        .await
        .map_err(|err| RegisterPharmacistError::UsersError(err))?;

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
pub struct LoginWithCredentialsDto {
    username: String,
    password: String,
}

#[openapi(tag = "Auth")]
#[post("/auth/login/doctor", data = "<dto>", format = "application/json")]
pub async fn login_doctor(
    ctx: &Ctx,
    dto: Json<LoginWithCredentialsDto>,
    client: ClientRequestInfo,
) -> Result<Json<SessionTokenResponse>, AuthenticationWithCredentialsError> {
    let user = ctx
        .authentication_service
        .authenticate_with_credentials(dto.0.username, dto.0.password, UserRole::Doctor)
        .await
        .map_err(|_| AuthenticationWithCredentialsError::InvalidCredentials)?;

    let session = ctx
        .sessions_service
        .create_session(
            user.id,
            user.doctor.map(|d| d.id),
            None,
            client.ip_address,
            client.user_agent,
        )
        .await
        .unwrap();

    Ok(Json(SessionTokenResponse {
        token: session.id.to_string(),
    }))
}

#[openapi(tag = "Auth")]
#[post("/auth/login/pharmacist", data = "<dto>", format = "application/json")]
pub async fn login_pharmacist(
    ctx: &Ctx,
    dto: Json<LoginWithCredentialsDto>,
    client: ClientRequestInfo,
) -> Result<Json<SessionTokenResponse>, AuthenticationWithCredentialsError> {
    let user = ctx
        .authentication_service
        .authenticate_with_credentials(dto.0.username, dto.0.password, UserRole::Pharmacist)
        .await
        .map_err(|_| AuthenticationWithCredentialsError::InvalidCredentials)?;

    let session = ctx
        .sessions_service
        .create_session(
            user.id,
            None,
            user.pharmacist.map(|p| p.id),
            client.ip_address,
            client.user_agent,
        )
        .await
        .unwrap();

    Ok(Json(SessionTokenResponse {
        token: session.id.to_string(),
    }))
}

impl<'r> Responder<'r, 'static> for InvalidateSessionError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(err) => (err.to_string(), Status::UnprocessableEntity),
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    UpdateSessionRepositoryError::DatabaseError(_) => Status::InternalServerError,
                    UpdateSessionRepositoryError::NotFound(_) => Status::NotFound,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for InvalidateSessionError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![("404", "Session not found")])
    }
}

#[openapi(tag = "Auth")]
#[post("/auth/logout", format = "application/json")]
pub async fn logout(
    ctx: &Ctx,
    session: Session,
) -> Result<Json<SuccessResponse>, InvalidateSessionError> {
    ctx.sessions_service
        .invalidate_session(session)
        .await
        .map(|_| Json(SuccessResponse { success: true }))
}

pub struct AuthError;

impl<'r> Responder<'r, 'static> for AuthError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let message = "Unauthorized".into();
        let status = Status::Unauthorized;
        ApiError::build_rocket_response(req, message, status)
    }
}

#[get("/test-collection/endpoint-that-requires-authorization-as-doctor")]
pub async fn endpoint_that_requires_authorization_as_doctor(
    session: DoctorSession,
) -> Result<String, AuthError> {
    Ok(format!(
        "You are authorized as a doctor {}",
        session.0.doctor_id.unwrap()
    ))
}

#[get("/test-collection/endpoint-that-requires-authorization-as-pharmacist")]
pub async fn endpoint_that_requires_authorization_as_pharmacist(
    session: PharmacistSession,
) -> Result<String, AuthError> {
    Ok(format!(
        "You are authorized as a pharmacist {}",
        session.0.pharmacist_id.unwrap()
    ))
}

#[cfg(test)]
mod tests {
    use rocket::{
        http::{ContentType, Header, Status},
        local::asynchronous::Client,
        routes,
    };

    use super::SessionTokenResponse;
    use crate::{
        context::setup_context,
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn create_api_client(pool: sqlx::PgPool) -> Client {
        create_tables(&pool, true).await.unwrap();
        let context = setup_context(pool);

        let routes = routes![
            super::register_doctor,
            super::register_pharmacist,
            super::login_doctor,
            super::login_pharmacist,
            super::endpoint_that_requires_authorization_as_doctor,
            super::endpoint_that_requires_authorization_as_pharmacist,
            super::logout
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[sqlx::test]
    async fn test_doctor_auth(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);

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

        let response = client
            .post("/auth/logout")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-doctor")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);
    }

    #[sqlx::test]
    async fn test_pharmacist_auth(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-pharmacist")
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);

        let response = client
            .post("/auth/register/pharmacist")
            .header(ContentType::JSON)
            .body(
                r#"{
                    "username": "pharmacist",
                    "password": "password123",
                    "email": "pharmacist_john_doe@gmail.com",
                    "phone_number": "123456789",
                    "name": "John Doe",
                    "pesel_number": "99031301347"
                }"#,
            )
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .post("/auth/login/pharmacist")
            .header(ContentType::JSON)
            .body(
                r#"{
                    "username": "pharmacist",
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
            .get("/test-collection/endpoint-that-requires-authorization-as-pharmacist")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .post("/auth/logout")
            .header(ContentType::JSON)
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .get("/test-collection/endpoint-that-requires-authorization-as-pharmacist")
            .header(Header::new("Authorization", format!("Bearer {}", token)))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Forbidden);
    }
}
