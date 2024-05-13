use crate::{
    domain::doctors::{
        models::{Doctor, NewDoctor},
        repository::{
            doctors_repository_impl::DoctorsRepository,
            doctors_repository_trait::DoctorsRepositoryTrait,
        },
    },
    Ctx,
};
use okapi::openapi3::Responses;
use rocket::{
    http::{ContentType, Status},
    post,
    response::Responder,
    serde::json::Json,
    Request, Response, Route,
};
use rocket_okapi::{
    gen::OpenApiGenerator, okapi::schemars, response::OpenApiResponderInner, OpenApiError,
};
use rocket_okapi::{openapi, openapi_get_routes, JsonSchema};
use schemars::Map;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

fn example_name() -> &'static str {
    "John Doe"
}
fn example_pesel_number() -> &'static str {
    "96021807250"
}
fn example_pwz_number() -> &'static str {
    "5425740"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDoctorDto {
    /// Doctor's name
    #[schemars(example = "example_name")]
    pub name: String,
    /// Doctor's PESEL number
    #[schemars(example = "example_pesel_number")]
    pub pesel_number: String,
    /// Doctor's PWZ number
    #[schemars(example = "example_pwz_number")]
    pub pwz_number: String,
}

pub enum CreateDoctorErrorStatus {
    InputError(String),
    ValidationError(String),
    DatabaseError(String),
}

impl<'r> Responder<'r, 'static> for CreateDoctorErrorStatus {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            CreateDoctorErrorStatus::InputError(message) => Response::build()
                .sized_body(message.len(), std::io::Cursor::new(message))
                .header(ContentType::JSON)
                .status(Status::UnprocessableEntity)
                .ok(),
            CreateDoctorErrorStatus::ValidationError(message) => Response::build()
                .sized_body(message.len(), std::io::Cursor::new(message))
                .header(ContentType::JSON)
                .status(Status::BadRequest)
                .ok(),
            CreateDoctorErrorStatus::DatabaseError(message) => Response::build()
                .sized_body(message.len(), std::io::Cursor::new(message))
                .header(ContentType::JSON)
                .status(Status::BadRequest)
                .ok(),
        }
    }
}

impl OpenApiResponderInner for CreateDoctorErrorStatus {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "400".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the pwz_number or the pesel_number are either incorrect or already exist in the database"
                    .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the request body is incorrect".to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

#[openapi(tag = "Doctors")]
#[post("/", format = "application/json", data = "<dto>")]
pub async fn create_doctor(
    ctx: &Ctx,
    dto: Json<CreateDoctorDto>,
) -> Result<Json<Doctor>, CreateDoctorErrorStatus> {
    let new_doctor = NewDoctor::new(dto.0.name, dto.0.pwz_number, dto.0.pesel_number)
        .map_err(|err| CreateDoctorErrorStatus::ValidationError(err.to_string()))?;

    let created_doctor = DoctorsRepository::new(ctx.pool.borrow())
        .create_doctor(new_doctor)
        .await
        .map_err(|err| CreateDoctorErrorStatus::DatabaseError(err.to_string()))?;

    Ok(Json(created_doctor))
}

pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![create_doctor]
}

#[cfg(test)]
mod integration_tests {
    use crate::{create_tables::create_tables, domain::doctors::models::Doctor, Context};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json,
        Build, Rocket,
    };
    use std::sync::Arc;

    async fn rocket(pool: sqlx::PgPool) -> Rocket<Build> {
        create_tables(&pool, true).await.unwrap();

        let pool = Arc::new(pool);
        rocket::build()
            .manage(Context { pool })
            .mount("/doctors", super::get_routes())
    }

    #[sqlx::test]
    async fn creates_doctor(pool: sqlx::PgPool) {
        let rocket = rocket(pool).await;
        let client = Client::tracked(rocket).await.unwrap();

        let mut request = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#);
        request.add_header(ContentType::JSON);
        let response = request.dispatch().await;

        let doctor: Doctor = json::from_str(&response.into_string().await.unwrap()).unwrap();
        assert_eq!(doctor.name, "John Doex");
        assert_eq!(doctor.pesel_number, "96021807250");
        assert_eq!(doctor.pwz_number, "5425740");
    }

    #[sqlx::test]
    async fn returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
        let rocket = rocket(pool).await;
        let client = Client::tracked(rocket).await.unwrap();

        let mut request_with_wrong_key = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250", "pwz_number":"5425740"}"#);
        request_with_wrong_key.add_header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);

        let mut request_with_wrong_value = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251", "pwz_number":"5425740"}"#);
        request_with_wrong_value.add_header(ContentType::JSON);
        let response = request_with_wrong_value.dispatch().await;

        assert_eq!(response.status(), Status::BadRequest);
    }
}
