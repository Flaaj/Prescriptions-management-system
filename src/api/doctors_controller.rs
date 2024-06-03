use crate::{
    domain::doctors::{
        models::Doctor,
        repository::{
            CreateDoctorRepositoryError, GetDoctorByIdRepositoryError, GetDoctorsRepositoryError,
        },
        service::{CreateDoctorError, GetDoctorByIdError, GetDoctorsWithPaginationError},
    },
    Ctx,
};
use okapi::openapi3::Responses;
use rocket::{
    get,
    http::Status,
    post,
    response::{status::Created, Responder},
    serde::json::Json,
    Request,
};
use rocket_okapi::{
    gen::OpenApiGenerator, okapi::schemars, response::OpenApiResponderInner, OpenApiError,
};
use rocket_okapi::{openapi, JsonSchema};
use schemars::Map;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::ApiError;

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
    name: String,
    #[schemars(example = "example_pesel_number")]
    pesel_number: String,
    #[schemars(example = "example_pwz_number")]
    pwz_number: String,
}

impl<'r> Responder<'r, 'static> for CreateDoctorError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    CreateDoctorRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                    CreateDoctorRepositoryError::DuplicatedPwzNumber => Status::Conflict,
                    CreateDoctorRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for CreateDoctorError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "422".to_string(),
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
                    "Returned when the doctor with given pwz_number or pesel_number exist in the database"
                        .to_string(),
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
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetDoctorByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetDoctorByIdRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetDoctorByIdError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
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
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the doctor_id is not a valid UUID".to_string(),
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

impl<'r> Responder<'r, 'static> for GetDoctorsWithPaginationError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetDoctorsRepositoryError::InvalidPaginationParams(_) => {
                        Status::UnprocessableEntity
                    }
                    GetDoctorsRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetDoctorsWithPaginationError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the page < 0 or page_size < 1".to_string(),
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
) -> Result<Json<Vec<Doctor>>, GetDoctorsWithPaginationError> {
    let doctors = ctx
        .doctors_service
        .get_doctors_with_pagination(page, page_size)
        .await?;

    Ok(Json(doctors))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        domain::{
            doctors::{models::Doctor, repository::DoctorsRepositoryFake, service::DoctorsService},
            drugs::{repository::DrugsRepositoryFake, service::DrugsService},
            patients::{repository::PatientsRepositoryFake, service::PatientsService},
            pharmacists::{repository::PharmacistsRepositoryFake, service::PharmacistsService},
            prescriptions::{
                repository::PrescriptionsRepositoryFake, service::PrescriptionsService,
            },
        },
        Context,
    };
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        routes,
        serde::json,
    };

    async fn create_api_client() -> Client {
        let doctors_repository = Box::new(DoctorsRepositoryFake::new());
        let doctors_service = Arc::new(DoctorsService::new(doctors_repository));

        let pharmacists_rerpository = Box::new(PharmacistsRepositoryFake::new());
        let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_rerpository));

        let patients_repository = Box::new(PatientsRepositoryFake::new());
        let patients_service = Arc::new(PatientsService::new(patients_repository));

        let drugs_repository = Box::new(DrugsRepositoryFake::new());
        let drugs_service = Arc::new(DrugsService::new(drugs_repository));

        let prescriptions_repository = Box::new(PrescriptionsRepositoryFake::new(
            None, None, None, None, None,
        ));
        let prescriptions_service = Arc::new(PrescriptionsService::new(prescriptions_repository));

        let context = Context {
            doctors_service,
            pharmacists_service,
            patients_service,
            drugs_service,
            prescriptions_service,
        };

        let routes = routes![
            super::create_doctor,
            super::get_doctor_by_id,
            super::get_doctors_with_pagination
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    async fn creates_doctor_and_reads_by_id() {
        let client = create_api_client().await;

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

    #[tokio::test]
    async fn create_doctor_returns_unprocessable_entity_if_body_has_incorrect_keys() {
        let client = create_api_client().await;

        let request_with_wrong_key = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250", "pwz_number":"5425740"}"#)
            .header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn create_doctor_returns_unprocessable_entity_if_body_has_incorrect_value_incorrect() {
        let client = create_api_client().await;

        let mut request_with_incorrect_value = client
            .post("/doctors")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251", "pwz_number":"5425740"}"#);
        request_with_incorrect_value.add_header(ContentType::JSON);
        let response = request_with_incorrect_value.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn create_doctor_returns_conflict_if_pwz_or_pesel_numbers_are_duplicated() {
        let client = create_api_client().await;

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

    #[tokio::test]
    async fn get_doctor_by_id_returns_unprocessable_entity_if_id_param_is_invalid() {
        let client = create_api_client().await;

        let request = client.get("/doctors/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn get_doctor_by_id_returns_not_found_if_such_doctor_does_not_exist() {
        let client = create_api_client().await;

        let request = client
            .get("/doctors/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[tokio::test]
    async fn gets_doctors_with_pagination() {
        let client = create_api_client().await;
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

    #[tokio::test]
    async fn get_doctors_with_pagination_returns_unprocessable_entity_if_page_or_page_size_is_invalid(
    ) {
        let client = create_api_client().await;

        assert_eq!(
            client
                .get("/doctors?page=-1")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );

        assert_eq!(
            client
                .get("/doctors?page_size=0")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }
}
