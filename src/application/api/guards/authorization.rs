use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use rocket_okapi::request::OpenApiFromRequest;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{application::sessions::models::Session, Context};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuthorizationError {
    Unauthorized,
}

async fn get_session<'r>(req: &'r Request<'_>) -> Option<Session> {
    let ctx = req.rocket().state::<Context>().unwrap();

    let header = req.headers().get_one("Authorization")?;
    let (_, session_token) = header.split_at(7);
    let session_id = Uuid::parse_str(session_token).ok()?;

    let session = ctx
        .sessions_service
        .get_session_by_id(session_id)
        .await
        .ok()?;

    session.validate().ok()?;

    Some(session)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Session {
    type Error = AuthorizationError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_session(req).await {
            Some(session) => Outcome::Success(session),
            None => Outcome::Error((Status::Forbidden, AuthorizationError::Unauthorized)),
        }
    }
}

#[derive(OpenApiFromRequest)]
pub struct DoctorSession(pub Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DoctorSession {
    type Error = AuthorizationError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_session(req).await {
            Some(session) if session.doctor_id.is_some() => Outcome::Success(Self(session)),
            _ => Outcome::Error((Status::Forbidden, AuthorizationError::Unauthorized)),
        }
    }
}

#[derive(OpenApiFromRequest)]
pub struct PharmacistSession(pub Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PharmacistSession {
    type Error = AuthorizationError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_session(req).await {
            Some(session) if session.pharmacist_id.is_some() => Outcome::Success(Self(session)),
            _ => Outcome::Error((Status::Forbidden, AuthorizationError::Unauthorized)),
        }
    }
}
