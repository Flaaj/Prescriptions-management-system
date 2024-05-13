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
    get,
    http::{ContentType, Status},
    post,
    response::{status::Created, Responder},
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
    ValidationError(String),
    DatabaseError(String),
}

impl<'r> Responder<'r, 'static> for CreateDoctorErrorStatus {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
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
                description: "Returned when the pwz_number or the pesel_number are either incorrect or already exist in the database, or the name has incorrect format"
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
#[post("/doctors", format = "application/json", data = "<dto>")]
pub async fn create_doctor(
    ctx: &Ctx,
    dto: Json<CreateDoctorDto>,
) -> Result<Created<Json<Doctor>>, CreateDoctorErrorStatus> {
    let new_doctor = NewDoctor::new(dto.0.name, dto.0.pwz_number, dto.0.pesel_number)
        .map_err(|err| CreateDoctorErrorStatus::ValidationError(err.to_string()))?;

    let created_doctor = DoctorsRepository::new(ctx.pool.borrow())
        .create_doctor(new_doctor)
        .await
        .map_err(|err| CreateDoctorErrorStatus::DatabaseError(err.to_string()))?;

    let location = format!("/doctors/{}", created_doctor.id); // assuming you have a route like this
    Ok(Created::new(location).body(Json(created_doctor)))
}

pub enum GetDoctorByIdErrorStatus {
    InputError,
    DatabaseError(String),
}

impl<'r> Responder<'r, 'static> for GetDoctorByIdErrorStatus {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            GetDoctorByIdErrorStatus::InputError => {
                let message = "Doctor id is incorrect - it must be provided in UUID format";
                Response::build()
                    .sized_body(message.len(), std::io::Cursor::new(message))
                    .header(ContentType::JSON)
                    .status(Status::UnprocessableEntity)
                    .ok()
            }
            GetDoctorByIdErrorStatus::DatabaseError(message) => Response::build()
                .sized_body(message.len(), std::io::Cursor::new(message))
                .header(ContentType::JSON)
                .status(Status::BadRequest)
                .ok(),
        }
    }
}

impl OpenApiResponderInner for GetDoctorByIdErrorStatus {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "400".to_string(),
            RefOr::Object(OpenApiReponse {
                description:
                    "Returned when the the doctor with given id doesn't exist in the database"
                        .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the id is invalid".to_string(),
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
#[get("/doctors/<doctor_id>", format = "application/json")]
pub async fn get_doctor_by_id(
    ctx: &Ctx,
    doctor_id: String,
) -> Result<Json<Doctor>, GetDoctorByIdErrorStatus> {
    let doctor_id = doctor_id
        .parse()
        .map_err(|_| GetDoctorByIdErrorStatus::InputError)?;
    let doctor = DoctorsRepository::new(ctx.pool.borrow())
        .get_doctor_by_id(doctor_id)
        .await
        .map_err(|err| GetDoctorByIdErrorStatus::DatabaseError(err.to_string()))?;

    Ok(Json(doctor))
}

pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![create_doctor, get_doctor_by_id]
}

#[cfg(test)]
mod integration_tests {
    use crate::{create_tables::create_tables, domain::doctors::models::Doctor, Context};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json,
    };
    use std::sync::Arc;

    async fn create_api_client(pool: sqlx::PgPool) -> Client {
        create_tables(&pool, true).await.unwrap();

        let pool = Arc::new(pool);
        let rocket = rocket::build()
            .manage(Context { pool })
            .mount("/", super::get_routes());

        Client::tracked(rocket).await.unwrap()
    }

    #[sqlx::test]
    async fn creates_doctor_and_reads_by_id(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let create_doctor_response = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        assert_eq!(create_doctor_response.status(), Status::Created);

        let created_doctor: Doctor =
            json::from_str(&create_doctor_response.into_string().await.unwrap()).unwrap();
        assert_eq!(created_doctor.name, "John Doex");
        assert_eq!(created_doctor.pesel_number, "96021807250");
        assert_eq!(created_doctor.pwz_number, "5425740");

        let get_doctor_by_id_response = client
            .get(format!("/doctors/{}", created_doctor.id))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        let doctor: Doctor =
            json::from_str(&get_doctor_by_id_response.into_string().await.unwrap()).unwrap();
        assert_eq!(doctor.name, "John Doex");
        assert_eq!(doctor.pesel_number, "96021807250");
        assert_eq!(doctor.pwz_number, "5425740");
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request_with_wrong_key = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);

        let mut request_with_incorrect_value = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251", "pwz_number":"5425740"}"#);
        request_with_incorrect_value.add_header(ContentType::JSON);
        let response = request_with_incorrect_value.dispatch().await;

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[sqlx::test]
    async fn create_doctor_returns_error_if_pwz_or_pesel_numbers_are_duplicated(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::Created);

        let request_with_duplicated_pesel = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"8463856"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pesel.dispatch().await;

        assert_eq!(response.status(), Status::BadRequest);

        let request_with_duplicated_pwz = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"99031301347", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pwz.dispatch().await;

        assert_eq!(response.status(), Status::BadRequest);
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_error_if_id_param_is_invalid(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request = client.get("/doctors/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_error_if_such_doctor_does_not_exist(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request = client
            .get("/doctors/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::BadRequest);
    }
}
