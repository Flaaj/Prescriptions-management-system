use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{application::sessions::models::Session, Context};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum AuthorizationError {
    Unauthorized,
}

pub struct DoctorSession(pub Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DoctorSession {
    type Error = AuthorizationError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(header) => {
                let (_, session_token) = header.split_at(7);
                let session_id = Uuid::parse_str(session_token);
                if session_id.is_err() {
                    return Outcome::Error((
                        Status::Unauthorized,
                        AuthorizationError::Unauthorized,
                    ));
                }
                let session_id = session_id.unwrap();

                let config = req.rocket().state::<Context>().unwrap();

                let session = config
                    .sessions_service
                    .get_session_by_id(session_id)
                    .await
                    .map_err(|_| (Status::Unauthorized, AuthorizationError::Unauthorized));

                match session {
                    Ok(session) => {
                        if session.validate().is_err() {
                            return Outcome::Error((
                                Status::Unauthorized,
                                AuthorizationError::Unauthorized,
                            ));
                        }
                        if session.doctor_id.is_none() {
                            return Outcome::Error((
                                Status::Unauthorized,
                                AuthorizationError::Unauthorized,
                            ));
                        }

                        Outcome::Success(Self(session))
                    }
                    Err(_) => {
                        Outcome::Error((Status::Unauthorized, AuthorizationError::Unauthorized))
                    }
                }
            }
            None => Outcome::Error((Status::Unauthorized, AuthorizationError::Unauthorized)),
        }
    }
}

pub struct PharmacistSession(pub Session);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PharmacistSession {
    type Error = AuthorizationError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.headers().get_one("Authorization") {
            Some(header) => {
                let (_, session_token) = header.split_at(7);
                let session_id = Uuid::parse_str(session_token);
                if session_id.is_err() {
                    return Outcome::Error((
                        Status::Unauthorized,
                        AuthorizationError::Unauthorized,
                    ));
                }
                let session_id = session_id.unwrap();

                let config = req.rocket().state::<Context>().unwrap();

                let session = config
                    .sessions_service
                    .get_session_by_id(session_id)
                    .await
                    .map_err(|_| (Status::Unauthorized, AuthorizationError::Unauthorized));

                match session {
                    Ok(session) => {
                        if session.validate().is_err() {
                            return Outcome::Error((
                                Status::Unauthorized,
                                AuthorizationError::Unauthorized,
                            ));
                        }
                        if session.pharmacist_id.is_none() {
                            return Outcome::Error((
                                Status::Unauthorized,
                                AuthorizationError::Unauthorized,
                            ));
                        }

                        Outcome::Success(Self(session))
                    }
                    Err(_) => {
                        Outcome::Error((Status::Unauthorized, AuthorizationError::Unauthorized))
                    }
                }
            }
            None => Outcome::Error((Status::Unauthorized, AuthorizationError::Unauthorized)),
        }
    }
}
