use crate::{
    domain::doctors::{
        models::Doctor,
        repository::{
            CreateDoctorRepositoryError, GetDoctorByIdRepositoryError, GetDoctorsRepositoryError,
        },
        service::{CreateDoctorError, GetDoctorByIdError, GetDoctorWithPaginationError},
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
use uuid::Uuid;

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
    #[schemars(example = "example_name")]
    pub name: String,
    #[schemars(example = "example_pesel_number")]
    pub pesel_number: String,
    #[schemars(example = "example_pwz_number")]
    pub pwz_number: String,
}

impl<'r> Responder<'r, 'static> for CreateDoctorError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            CreateDoctorError::DomainError(message) => Response::build()
                .sized_body(message.len(), std::io::Cursor::new(message))
                .header(ContentType::JSON)
                .status(Status::UnprocessableEntity)
                .ok(),
            CreateDoctorError::RepositoryError(repository_err) => {
                let message = repository_err.to_string();
                Response::build()
                    .sized_body(message.len(), std::io::Cursor::new(message))
                    .header(ContentType::JSON)
                    .status(match repository_err {
                        CreateDoctorRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                        CreateDoctorRepositoryError::DuplicatedPwzNumber => Status::Conflict,
                        CreateDoctorRepositoryError::DatabaseError(_) => {
                            Status::InternalServerError
                        }
                    })
                    .ok()
            }
        }
    }
}

impl OpenApiResponderInner for CreateDoctorError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "421".to_string(),
            RefOr::Object(OpenApiReponse {
                description:
                    "Returned when the name, the pesel_number or the pwz_number are incorrect"
                        .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiReponse {
                description:
                    "Returned when the pwz_number or the pesel_number exist in the database"
                        .to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Unexpected server error - please contact developer".to_string(),
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
) -> Result<Created<Json<Doctor>>, CreateDoctorError> {
    let created_doctor = ctx
        .doctors_service
        .create_doctor(dto.0.name, dto.0.pesel_number, dto.0.pwz_number)
        .await?;

    let location = format!("/doctors/{}", created_doctor.id);
    Ok(Created::new(location).body(Json(created_doctor)))
}

impl<'r> Responder<'r, 'static> for GetDoctorByIdError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            GetDoctorByIdError::RepositoryError(repository_err) => {
                let message = repository_err.to_string();
                Response::build()
                    .sized_body(message.len(), std::io::Cursor::new(message))
                    .header(ContentType::JSON)
                    .status(match repository_err {
                        GetDoctorByIdRepositoryError::NotFound(_) => Status::NotFound,
                        GetDoctorByIdRepositoryError::DatabaseError(_) => {
                            Status::InternalServerError
                        }
                    })
                    .ok()
            }
        }
    }
}

impl OpenApiResponderInner for GetDoctorByIdError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the doctor with given id doesn't exist".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "421".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the id is not UUID".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Unexpected server error - please contact developer".to_string(),
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
    doctor_id: Uuid,
) -> Result<Json<Doctor>, GetDoctorByIdError> {
    let doctor = ctx.doctors_service.get_doctor_by_id(doctor_id).await?;

    Ok(Json(doctor))
}

impl<'r> Responder<'r, 'static> for GetDoctorWithPaginationError {
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            Self::RepositoryError(repository_err) => {
                let message = repository_err.to_string();
                Response::build()
                    .sized_body(message.len(), std::io::Cursor::new(message))
                    .header(ContentType::JSON)
                    .status(match repository_err {
                        GetDoctorsRepositoryError::InvalidPaginationParams(_) => Status::BadRequest,
                        GetDoctorsRepositoryError::DatabaseError(_) => Status::InternalServerError,
                    })
                    .ok()
            }
        }
    }
}

impl OpenApiResponderInner for GetDoctorWithPaginationError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "400".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the page < 0 or page_size < 1".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "500".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Unexpected server error - please contact developer".to_string(),
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
#[get("/doctors?<page>&<page_size>", format = "application/json")]
pub async fn get_doctors_with_pagination(
    ctx: &Ctx,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Json<Vec<Doctor>>, GetDoctorWithPaginationError> {
    let doctors = ctx
        .doctors_service
        .get_doctors_with_pagination(page, page_size)
        .await?;

    Ok(Json(doctors))
}

pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![create_doctor, get_doctor_by_id, get_doctors_with_pagination]
}

#[cfg(test)]
mod tests {
    use crate::{create_tables::create_tables, domain::doctors::models::Doctor, setup_context};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json,
    };

    async fn create_api_client(pool: sqlx::PgPool) -> Client {
        create_tables(&pool, true).await.unwrap();

        let context = setup_context(pool);
        let rocket = rocket::build()
            .manage(context)
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

        assert_eq!(get_doctor_by_id_response.status(), Status::Ok);

        let doctor: Doctor =
            json::from_str(&get_doctor_by_id_response.into_string().await.unwrap()).unwrap();

        assert_eq!(doctor.name, "John Doex");
        assert_eq!(doctor.pesel_number, "96021807250");
        assert_eq!(doctor.pwz_number, "5425740");
    }

    #[sqlx::test]
    async fn create_doctor_returns_unprocessable_entity_if_body_has_incorrect_keys(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request_with_wrong_key = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn create_doctor_returns_unprocessable_entity_if_body_has_incorrect_value_incorrect(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let mut request_with_incorrect_value = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251", "pwz_number":"5425740"}"#);
        request_with_incorrect_value.add_header(ContentType::JSON);
        let response = request_with_incorrect_value.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn create_doctor_returns_conflict_if_pwz_or_pesel_numbers_are_duplicated(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        request.dispatch().await;

        let request_with_duplicated_pesel = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"8463856"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pesel.dispatch().await;

        assert_eq!(response.status(), Status::Conflict);

        let request_with_duplicated_pwz = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"99031301347", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pwz.dispatch().await;

        assert_eq!(response.status(), Status::Conflict);
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_unprocessable_entity_if_id_param_is_invalid(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client.get("/doctors/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn get_doctor_by_id_returns_not_found_if_such_doctor_does_not_exist(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request = client
            .get("/doctors/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn gets_doctors_with_pagination(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;
        client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021817257", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/doctors")
            .body(r#"{"name":"John Doey", "pesel_number":"99031301347", "pwz_number":"8463856"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/doctors")
            .body(r#"{"name":"John Doez", "pesel_number":"92022900002", "pwz_number":"3123456"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/doctors")
            .body(r#"{"name":"John Doeq", "pesel_number":"96021807250", "pwz_number":"5425751"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        let response = client
            .get("/doctors?page=1&page_size=2")
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

        assert_eq!(doctors.len(), 2);
    }
}
