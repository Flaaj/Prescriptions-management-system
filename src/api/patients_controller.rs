use crate::{
    domain::patients::{
        models::Patient,
        repository::{CreatePatientRepositoryError, GetPatientByIdRepositoryError},
        service::{CreatePatientError, GetPatientByIdError},
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
    gen::OpenApiGenerator, okapi::schemars, openapi, openapi_get_routes,
    response::OpenApiResponderInner, OpenApiError,
};
use schemars::{JsonSchema, Map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(repository_err) => {
                let message = repository_err.to_string();
                let status = match repository_err {
                    CreatePatientRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                    CreatePatientRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        Response::build()
            .sized_body(message.len(), std::io::Cursor::new(message))
            .header(ContentType::JSON)
            .status(status)
            .ok()
    }
}

impl OpenApiResponderInner for CreatePatientError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "421".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the name or the pesel_number are incorrect".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "409".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when patient with given pesel_number exist in the database"
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
    fn respond_to(self, _: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(repository_err) => {
                let message = repository_err.to_string();
                let status = match repository_err {
                    GetPatientByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetPatientByIdRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        Response::build()
            .sized_body(message.len(), std::io::Cursor::new(message))
            .header(ContentType::JSON)
            .status(status)
            .ok()
    }
}

impl OpenApiResponderInner for GetPatientByIdError {
    fn responses(_generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the the patient with given id doesn't exist"
                    .to_string(),
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

#[openapi(tag = "Patients")]
#[get("/patients/<patient_id>", format = "application/json")]
pub async fn get_patient_by_id(
    ctx: &Ctx,
    patient_id: Uuid,
) -> Result<Json<Patient>, GetPatientByIdError> {
    let patient = ctx.patients_service.get_patient_by_id(patient_id).await?;

    Ok(Json(patient))
}

pub fn get_routes() -> Vec<Route> {
    openapi_get_routes![
        create_patient,
        get_patient_by_id,
        // get_patients_with_pagination,
    ]
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        serde::json,
    };

    use crate::{
        domain::{
            doctors::{repository::DoctorsRepositoryFake, service::DoctorsService},
            drugs::{repository::DrugsRepositoryFake, service::DrugsService},
            patients::{
                models::Patient, repository::PatientsRepositoryFake, service::PatientsService,
            },
            pharmacists::{repository::PharmacistsRepositoryFake, service::PharmacistsService},
            prescriptions::{
                repository::PrescriptionsRepositoryFake, service::PrescriptionsService,
            },
        },
        Context,
    };

    async fn create_api_client() -> Client {
        let patients_repository = Box::new(PatientsRepositoryFake::new());
        let patients_service = Arc::new(PatientsService::new(patients_repository));
        let pharmacists_rerpository = Box::new(PharmacistsRepositoryFake::new());
        let pharmacists_service = Arc::new(PharmacistsService::new(pharmacists_rerpository));
        let doctors_repository = Box::new(DoctorsRepositoryFake::new());
        let doctors_service = Arc::new(DoctorsService::new(doctors_repository));
        let drugs_repository = Box::new(DrugsRepositoryFake::new());
        let drugs_service = Arc::new(DrugsService::new(drugs_repository));
        let prescriptions_repository = Box::new(PrescriptionsRepositoryFake::new(
            None, None, None, None, None,
        ));
        let prescriptions_service = Arc::new(PrescriptionsService::new(prescriptions_repository));

        let context = Context {
            patients_service,
            pharmacists_service,
            doctors_service,
            drugs_service,
            prescriptions_service,
        };

        let rocket = rocket::build()
            .manage(context)
            .mount("/", super::get_routes());

        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    async fn creates_patient_and_reads_by_id() {
        let client = create_api_client().await;

        let create_patient_response = client
            .post("/patients")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
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
}
