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
    gen::OpenApiGenerator, okapi::schemars, openapi, response::OpenApiResponderInner, OpenApiError,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::api::utils::{error::ApiError, openapi_responses::get_openapi_responses}, context::Ctx, domain::patients::{
        entities::Patient,
        repository::{
            CreatePatientRepositoryError, GetPatientByIdRepositoryError, GetPatientsRepositoryError,
        },
        service::{CreatePatientError, GetPatientByIdError, GetPatientsWithPaginationError},
    }
};

fn example_name() -> &'static str {
    "John Doe"
}
fn example_pesel_number() -> &'static str {
    "96021807250"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePatientDto {
    #[schemars(example = "example_name")]
    name: String,
    #[schemars(example = "example_pesel_number")]
    pesel_number: String,
}

impl<'r> Responder<'r, 'static> for CreatePatientError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    CreatePatientRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                    CreatePatientRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for CreatePatientError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "422",
                "Returned when the name or the pesel_number are incorrect",
            ),
            (
                "409",
                "Returned when patient with given pesel_number exist in the database",
            ),
        ])
    }
}

#[openapi(tag = "Patients")]
#[post("/patients", format = "application/json", data = "<dto>")]
pub async fn create_patient(
    ctx: &Ctx,
    dto: Json<CreatePatientDto>,
) -> Result<Created<Json<Patient>>, CreatePatientError> {
    let created_patient = ctx
        .patients_service
        .create_patient(dto.0.name, dto.0.pesel_number)
        .await?;

    let location = format!("/patients/{}", created_patient.id);
    Ok(Created::new(location).body(Json(created_patient)))
}

impl<'r> Responder<'r, 'static> for GetPatientByIdError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPatientByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetPatientByIdRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPatientByIdError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "404",
                "Returned when the the patient with given id doesn't exist",
            ),
            (
                "422",
                "Returned when the the patient_id is not a valid UUID",
            ),
        ])
    }
}

#[openapi(tag = "Patients")]
#[get("/patients/<patient_id>", format = "application/json")]
pub async fn get_patient_by_id(
    ctx: &Ctx,
    patient_id: Uuid,
) -> Result<Json<Patient>, GetPatientByIdError> {
    let patient = ctx.patients_service.get_patient_by_id(patient_id).await?;

    Ok(Json(patient))
}

impl<'r> Responder<'r, 'static> for GetPatientsWithPaginationError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPatientsRepositoryError::InvalidPaginationParams(_) => {
                        Status::UnprocessableEntity
                    }
                    GetPatientsRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPatientsWithPaginationError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            ("404", "Returned when the the page < 0 or page_size < 1"),
            ("422", "Returned when the the page < 0 or page_size < 1"),
        ])
    }
}

#[openapi(tag = "Patients")]
#[get("/patients?<page>&<page_size>", format = "application/json")]
pub async fn get_patients_with_pagination(
    ctx: &Ctx,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Json<Vec<Patient>>, GetPatientsWithPaginationError> {
    let patients = ctx
        .patients_service
        .get_patients_with_pagination(page, page_size)
        .await?;

    Ok(Json(patients))
}

#[cfg(test)]
mod tests {

    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        routes,
        serde::json,
    };

    use crate::{
        context::setup_context, domain::patients::entities::Patient,
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn create_api_client(pool: sqlx::PgPool) -> Client {
        create_tables(&pool, true).await.unwrap();
        let context = setup_context(pool);

        let routes = routes![
            super::create_patient,
            super::get_patient_by_id,
            super::get_patients_with_pagination
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[sqlx::test]
    async fn creates_patient_and_reads_by_id(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let create_patient_response = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(create_patient_response.status(), Status::Created);

        let created_patient: Patient =
            json::from_str(&create_patient_response.into_string().await.unwrap()).unwrap();

        assert_eq!(created_patient.name, "John Doex");
        assert_eq!(created_patient.pesel_number, "96021807250");

        let get_patient_by_id_response = client
            .get(format!("/patients/{}", created_patient.id))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(get_patient_by_id_response.status(), Status::Ok);

        let patient: Patient =
            json::from_str(&get_patient_by_id_response.into_string().await.unwrap()).unwrap();

        assert_eq!(patient.name, "John Doex");
        assert_eq!(patient.pesel_number, "96021807250");
    }

    #[sqlx::test]
    async fn create_patient_returns_unprocessable_entity_if_body_has_incorrect_keys(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request_with_wrong_key = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250"}"#)
            .header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn create_patient_returns_unprocessable_entity_if_body_has_incorrect_value_incorrect(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let mut request_with_incorrect_value = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251"}"#);
        request_with_incorrect_value.add_header(ContentType::JSON);
        let response = request_with_incorrect_value.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn create_patient_returns_conflict_if_pesel_number_is_duplicated(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON);
        request.dispatch().await;

        let request_with_duplicated_pesel = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pesel.dispatch().await;

        assert_eq!(response.status(), Status::Conflict);
    }

    #[sqlx::test]
    async fn get_patient_by_id_returns_unprocessable_entity_if_id_param_is_invalid(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client.get("/patients/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn get_patient_by_id_returns_not_found_if_such_patient_does_not_exist(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client
            .get("/patients/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn gets_patients_with_pagination(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;
        client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021817257"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/patients")
            .body(r#"{"name":"John Doey", "pesel_number":"99031301347"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/patients")
            .body(r#"{"name":"John Doez", "pesel_number":"92022900002"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/patients")
            .body(r#"{"name":"John Doeq", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        let response = client
            .get("/patients?page=1&page_size=2")
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let patients: Vec<Patient> =
            json::from_str(&response.into_string().await.unwrap()).unwrap();

        assert_eq!(patients.len(), 2);
    }

    #[sqlx::test]
    async fn get_patients_with_pagination_returns_unprocessable_entity_if_page_or_page_size_is_invalid(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        assert_eq!(
            client
                .get("/patients?page=-1")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );

        assert_eq!(
            client
                .get("/patients?page_size=0")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }
}
