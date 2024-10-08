use chrono::{DateTime, Utc};
use okapi::openapi3::Responses;
use rocket::{
    get,
    http::Status,
    post,
    response::{status::Created, Responder},
    serde::json::Json,
    Request,
};
use rocket_okapi::{gen::OpenApiGenerator, openapi, response::OpenApiResponderInner, OpenApiError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    application::api::utils::{error::ApiError, openapi_responses::get_openapi_responses},
    context::Ctx,
    domain::prescriptions::{
        entities::{Prescription, PrescriptionType},
        repository::{
            CreatePrescriptionRepositoryError, FillPrescriptionRepositoryError,
            GetPrescriptionByIdRepositoryError, GetPrescriptionsRepositoryError,
        },
        service::{
            CreatePrescriptionError, FillPrescriptionError, GetPrescriptionByIdError,
            GetPrescriptionsWithPaginationError,
        },
    },
};

fn example_prescribed_drug() -> Vec<(Uuid, u32)> {
    vec![(Uuid::new_v4(), 2)]
}

type PrescribedDrugDto = (Uuid, u32);
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePrescriptionDto {
    doctor_id: Uuid,
    patient_id: Uuid,
    prescription_type: Option<PrescriptionType>,
    start_date: Option<DateTime<Utc>>,
    #[schemars(
        example = "example_prescribed_drug",
        description = "List of tuples with drug_id and quantity"
    )]
    prescribed_drugs: Vec<PrescribedDrugDto>,
}

impl<'r> Responder<'r, 'static> for CreatePrescriptionError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    CreatePrescriptionRepositoryError::DoctorNotFound(_) => Status::NotFound,
                    CreatePrescriptionRepositoryError::PatientNotFound(_) => Status::NotFound,
                    CreatePrescriptionRepositoryError::DrugNotFound(_) => Status::NotFound,
                    CreatePrescriptionRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for CreatePrescriptionError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(
            vec![
                (
                    "422",
                    "Returned when the body parameters are invalid, or the doctor_id, patient_id or drug_id is not a valid UUID",
                ),
                (
                    "404",
                    "Returned when doctor, patient or drug with given id doesn't exist",
                ),
            ]
        )
    }
}

#[openapi(tag = "Prescriptions")]
#[post("/prescriptions", format = "application/json", data = "<dto>")]
pub async fn create_prescription(
    ctx: &Ctx,
    dto: Json<CreatePrescriptionDto>,
) -> Result<Created<Json<Prescription>>, CreatePrescriptionError> {
    let created_prescription = ctx
        .prescriptions_service
        .create_prescription(
            dto.0.doctor_id,
            dto.0.patient_id,
            dto.0.start_date,
            dto.0.prescription_type,
            dto.0.prescribed_drugs,
        )
        .await?;

    let location = format!("/prescriptions/{}", created_prescription.id);
    Ok(Created::new(location).body(Json(created_prescription)))
}

impl<'r> Responder<'r, 'static> for GetPrescriptionByIdError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPrescriptionByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetPrescriptionByIdRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPrescriptionByIdError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "404",
                "Returned when the the prescription with given id doesn't exist",
            ),
            (
                "422",
                "Returned when the the prescription_id is not a valid UUID",
            ),
        ])
    }
}

#[openapi(tag = "Prescriptions")]
#[get("/prescriptions/<prescription_id>", format = "application/json")]
pub async fn get_prescription_by_id(
    ctx: &Ctx,
    prescription_id: Uuid,
) -> Result<Json<Prescription>, GetPrescriptionByIdError> {
    let prescription = ctx
        .prescriptions_service
        .get_prescription_by_id(prescription_id)
        .await?;

    Ok(Json(prescription))
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FillPrescriptionDto {
    pharmacist_id: Uuid,
    prescription_code: String,
}

impl<'r> Responder<'r, 'static> for FillPrescriptionError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    FillPrescriptionRepositoryError::PharmacistNotFound(_) => Status::NotFound,
                    FillPrescriptionRepositoryError::PrescriptionNotFound(_) => Status::NotFound,
                    FillPrescriptionRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for FillPrescriptionError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "404",
                "Returned when the the prescription or pharmacist with given id doesn't exist",
            ),
            (
                "422",
                "Returned when the the prescription_id or pharmacist_id is not a valid UUID, prescriptions is already filled, or the prescription cant be filled today (e.g. today is before start_date or after end_date)",
            ),
        ])
    }
}

#[openapi(tag = "Prescriptions")]
#[post(
    "/prescriptions/<prescription_id>/fill",
    format = "application/json",
    data = "<dto>"
)]
pub async fn fill_prescription(
    ctx: &Ctx,
    prescription_id: Uuid,
    dto: Json<FillPrescriptionDto>,
) -> Result<Created<Json<Prescription>>, FillPrescriptionError> {
    let prescription = ctx
        .prescriptions_service
        .fill_prescription(
            prescription_id,
            dto.0.pharmacist_id,
            dto.0.prescription_code,
        )
        .await?;

    let location = format!("/prescriptions/{}", prescription.id);
    Ok(Created::new(location).body(Json(prescription)))
}

impl<'r> Responder<'r, 'static> for GetPrescriptionsWithPaginationError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPrescriptionsRepositoryError::InvalidPaginationParams(_) => {
                        Status::UnprocessableEntity
                    }
                    GetPrescriptionsRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPrescriptionsWithPaginationError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![(
            "422",
            "Returned when the the page < 0 or page_size < 1",
        )])
    }
}

#[openapi(tag = "Prescriptions")]
#[get("/prescriptions?<page>&<page_size>", format = "application/json")]
pub async fn get_prescriptions_with_pagination(
    ctx: &Ctx,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Json<Vec<Prescription>>, GetPrescriptionsWithPaginationError> {
    let prescriptions = ctx
        .prescriptions_service
        .get_prescriptions_with_pagination(page, page_size)
        .await?;

    Ok(Json(prescriptions))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        routes,
        serde::json,
    };

    use crate::{
        application::{
            authentication::service::AuthenticationService, sessions::service::SessionsService,
        },
        domain::{
            doctors::{entities::Doctor, service::DoctorsService},
            drugs::{
                entities::{Drug, DrugContentType},
                service::DrugsService,
            },
            patients::{entities::Patient, service::PatientsService},
            pharmacists::{entities::Pharmacist, service::PharmacistsService},
            prescriptions::{entities::Prescription, service::PrescriptionsService},
        },
        infrastructure::postgres_repository_impl::{
            authentication::PostgresAuthenticationRepository, create_tables::create_tables,
            doctors::PostgresDoctorsRepository, drugs::PostgresDrugsRepository,
            patients::PostgresPatientsRepository, pharmacists::PostgresPharmacistsRepository,
            prescriptions::PostgresPrescriptionsRepository, sessions::PostgresSessionsRepository,
        },
        Context,
    };
    struct DatabaseSeeds {
        doctor: Doctor,
        pharmacist: Pharmacist,
        patient: Patient,
        drugs: Vec<Drug>,
    }

    async fn setup_services_and_seed_database(pool: sqlx::PgPool) -> (Context, DatabaseSeeds) {
        create_tables(&pool, true).await.unwrap();

        let doctors_service =
            DoctorsService::new(Box::new(PostgresDoctorsRepository::new(pool.clone())));
        let created_doctor = doctors_service
            .create_doctor("John Doctor".into(), "92022900002".into(), "3123456".into())
            .await
            .unwrap();

        let pharmacist_service =
            PharmacistsService::new(Box::new(PostgresPharmacistsRepository::new(pool.clone())));
        let created_pharmacist = pharmacist_service
            .create_pharmacist("John Pharmacist".into(), "92022900002".into())
            .await
            .unwrap();

        let patients_service =
            PatientsService::new(Box::new(PostgresPatientsRepository::new(pool.clone())));
        let created_patient = patients_service
            .create_patient("John Patient".into(), "92022900002".into())
            .await
            .unwrap();

        let drugs_service = DrugsService::new(Box::new(PostgresDrugsRepository::new(pool.clone())));
        let created_drug_0 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_1 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_2 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();
        let created_drug_3 = drugs_service
            .create_drug(
                "Gripex".into(),
                DrugContentType::SolidPills,
                Some(20),
                Some(300),
                None,
                None,
            )
            .await
            .unwrap();

        let prescriptions_service =
            PrescriptionsService::new(Box::new(PostgresPrescriptionsRepository::new(pool.clone())));

        let authentication_repository =
            Box::new(PostgresAuthenticationRepository::new(pool.clone()));
        let authentication_service =
            Arc::new(AuthenticationService::new(authentication_repository));

        let sessions_repository = Box::new(PostgresSessionsRepository::new(pool));
        let sessions_service = Arc::new(SessionsService::new(sessions_repository));

        (
            Context {
                doctors_service: Arc::new(doctors_service),
                pharmacists_service: Arc::new(pharmacist_service),
                patients_service: Arc::new(patients_service),
                drugs_service: Arc::new(drugs_service),
                prescriptions_service: Arc::new(prescriptions_service),
                authentication_service,
                sessions_service,
            },
            DatabaseSeeds {
                doctor: created_doctor,
                pharmacist: created_pharmacist,
                patient: created_patient,
                drugs: vec![
                    created_drug_0,
                    created_drug_1,
                    created_drug_2,
                    created_drug_3,
                ],
            },
        )
    }

    async fn create_api_client(pool: sqlx::PgPool) -> (Client, DatabaseSeeds) {
        let (context, seeds) = setup_services_and_seed_database(pool).await;

        let routes = routes![
            super::create_prescription,
            super::get_prescription_by_id,
            super::get_prescriptions_with_pagination,
            super::fill_prescription
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        let client = Client::tracked(rocket).await.unwrap();

        (client, seeds)
    }

    #[sqlx::test]
    async fn creates_and_fills_prescription(pool: sqlx::PgPool) {
        let (client, seeds) = create_api_client(pool).await;

        let create_prescription_response = client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                    "doctor_id": "{}",
                    "patient_id": "{}",
                    "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                    "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
                }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;

        assert_eq!(create_prescription_response.status(), Status::Created);

        let created_prescription = json::from_str::<Prescription>(
            &create_prescription_response.into_string().await.unwrap(),
        )
        .unwrap();

        assert!(created_prescription.fill.is_none());

        let fill_prescription_response = client
            .post(format!("/prescriptions/{}/fill", created_prescription.id))
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                    "pharmacist_id": "{}",
                    "prescription_code": "{}"
                }}"#,
                seeds.pharmacist.id, created_prescription.code
            ))
            .dispatch()
            .await;

        assert_eq!(fill_prescription_response.status(), Status::Created);

        json::from_str::<Prescription>(&fill_prescription_response.into_string().await.unwrap())
            .unwrap();

        let get_prescription_by_id_response = client
            .get(format!("/prescriptions/{}", created_prescription.id))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(get_prescription_by_id_response.status(), Status::Ok);

        let prescription_by_id: Prescription =
            json::from_str(&get_prescription_by_id_response.into_string().await.unwrap()).unwrap();

        assert!(prescription_by_id.fill.is_some());
    }

    #[sqlx::test]
    async fn doesnt_fill_if_already_filled(pool: sqlx::PgPool) {
        let (client, seeds) = create_api_client(pool).await;
        let create_seed_prescription_response = client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                    "doctor_id": "{}",
                    "patient_id": "{}",
                    "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                    "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
                }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;
        let seed_prescription: Prescription = json::from_str(
            &create_seed_prescription_response
                .into_string()
                .await
                .unwrap(),
        )
        .unwrap();

        assert_eq!(
            client
                .post(format!("/prescriptions/{}/fill", seed_prescription.id))
                .header(ContentType::JSON)
                .body(format!(
                    r#"{{
                        "pharmacist_id": "{}",
                        "prescription_code": "{}"
                    }}"#,
                    seeds.pharmacist.id, seed_prescription.code,
                ))
                .dispatch()
                .await
                .status(),
            Status::Created
        );

        assert_eq!(
            client
                .post(format!("/prescriptions/{}/fill", seed_prescription.id))
                .header(ContentType::JSON)
                .body(format!(
                    r#"{{
                        "pharmacist_id": "{}",
                        "prescription_code": "{}"
                    }}"#,
                    seeds.pharmacist.id, seed_prescription.code,
                ))
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }

    #[sqlx::test]
    async fn returns_error_if_prescription_does_not_exist(pool: sqlx::PgPool) {
        let (client, _) = create_api_client(pool).await;

        let get_prescription_by_id_response = client
            .get("/prescriptions/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(get_prescription_by_id_response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn gets_pharmacists_with_pagination(pool: sqlx::PgPool) {
        let (client, seeds) = create_api_client(pool).await;

        client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                "doctor_id": "{}",
                "patient_id": "{}",
                "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
            }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;
        client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                "doctor_id": "{}",
                "patient_id": "{}",
                "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
            }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;
        client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                "doctor_id": "{}",
                "patient_id": "{}",
                "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
            }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;
        client
            .post("/prescriptions")
            .header(ContentType::JSON)
            .body(format!(
                r#"{{
                "doctor_id": "{}",
                "patient_id": "{}",
                "prescription_type": "FOR_CHRONIC_DISEASE_DRUGS",
                "prescribed_drugs": [ ["{}",  1], ["{}",  2] ]
            }}"#,
                seeds.doctor.id, seeds.patient.id, seeds.drugs[0].id, seeds.drugs[1].id
            ))
            .dispatch()
            .await;

        let prescriptions_response = client
            .get("/prescriptions?page_size=2&page=1")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 2);

        let prescriptions_response = client
            .get("/prescriptions?page_size=3&page=1")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 1);

        let prescriptions_response = client
            .get("/prescriptions?page_size=10")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 4);

        let prescriptions_response = client
            .get("/prescriptions?page=1")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 0);

        let prescriptions_response = client
            .get("/prescriptions")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 4);

        let prescriptions_response = client
            .get("/prescriptions?page_size=3&page=2")
            .header(ContentType::JSON)
            .dispatch()
            .await;
        let prescriptions: Vec<Prescription> =
            json::from_str(&prescriptions_response.into_string().await.unwrap()).unwrap();

        assert_eq!(prescriptions.len(), 0);
    }

    #[sqlx::test]
    async fn get_pharmacists_with_pagination_returns_error_if_params_are_invalid(
        pool: sqlx::PgPool,
    ) {
        let (client, _) = create_api_client(pool).await;

        assert_eq!(
            client
                .get("/prescriptions?page=-1")
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );

        assert_eq!(
            client
                .get("/prescriptions?page_size=0")
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }
}
