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
use schemars::{JsonSchema, Map};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::drugs::{
        models::{Drug, DrugContentType},
        repository::{GetDrugByIdRepositoryError, GetDrugsRepositoryError},
        service::{CreateDrugError, GetDrugByIdError, GetDrugsWithPaginationError},
    },
    Ctx,
};

use super::error::ApiError;

fn example_drug_name() -> &'static str {
    "Apap"
}
fn example_drug_content_type() -> DrugContentType {
    DrugContentType::SolidPills
}
fn example_pills_count() -> Option<i32> {
    Some(30)
}
fn example_mg_per_pill() -> Option<i32> {
    Some(300)
}
fn example_ml_per_pill() -> Option<i32> {
    None
}
fn example_volume_ml() -> Option<i32> {
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateDrugDto {
    #[schemars(example = "example_drug_name")]
    name: String,
    #[schemars(example = "example_drug_content_type")]
    content_type: DrugContentType,
    #[schemars(example = "example_pills_count")]
    pills_count: Option<i32>,
    #[schemars(example = "example_mg_per_pill")]
    mg_per_pill: Option<i32>,
    #[schemars(example = "example_ml_per_pill")]
    ml_per_pill: Option<i32>,
    #[schemars(example = "example_volume_ml")]
    volume_ml: Option<i32>,
}

impl<'r> Responder<'r, 'static> for CreateDrugError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(err) => (err.to_string(), Status::InternalServerError),
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for CreateDrugError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description:
                    "Returned when the quantity parameters dont match the content type (for instance when missing volume_ml from BOTTLE_OF_LIQUID content_type)"
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

#[openapi(tag = "Drugs")]
#[post("/drugs", format = "json", data = "<dto>")]
pub async fn create_drug(
    ctx: &Ctx,
    dto: Json<CreateDrugDto>,
) -> Result<Created<Json<Drug>>, CreateDrugError> {
    let created_drug = ctx
        .drugs_service
        .create_drug(
            dto.0.name,
            dto.0.content_type,
            dto.0.pills_count,
            dto.0.mg_per_pill,
            dto.0.ml_per_pill,
            dto.0.volume_ml,
        )
        .await?;

    let location = format!("/drugs/{}", created_drug.id);
    Ok(Created::new(location).body(Json(created_drug)))
}

impl<'r> Responder<'r, 'static> for GetDrugByIdError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetDrugByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetDrugByIdRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetDrugByIdError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        use rocket_okapi::okapi::openapi3::{RefOr, Response as OpenApiReponse};

        let mut responses = Map::new();
        responses.insert(
            "404".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the drug with the given id was not found".to_string(),
                ..Default::default()
            }),
        );
        responses.insert(
            "422".to_string(),
            RefOr::Object(OpenApiReponse {
                description: "Returned when the drug_id is not a valid UUID".to_string(),
                ..Default::default()
            }),
        );
        Ok(Responses {
            responses,
            ..Default::default()
        })
    }
}

#[openapi(tag = "Drugs")]
#[get("/drugs/<drug_id>")]
pub async fn get_drug_by_id(ctx: &Ctx, drug_id: Uuid) -> Result<Json<Drug>, GetDrugByIdError> {
    let drug = ctx.drugs_service.get_drug_by_id(drug_id).await?;

    Ok(Json(drug))
}

impl<'r> Responder<'r, 'static> for GetDrugsWithPaginationError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetDrugsRepositoryError::InvalidPaginationParams(_) => {
                        Status::UnprocessableEntity
                    }
                    GetDrugsRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetDrugsWithPaginationError {
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

#[openapi(tag = "Drugs")]
#[get("/drugs?<page>&<page_size>", format = "application/json")]
pub async fn get_drugs_with_pagination(
    ctx: &Ctx,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Json<Vec<Drug>>, GetDrugsWithPaginationError> {
    let drugs = ctx
        .drugs_service
        .get_drugs_with_pagination(page, page_size)
        .await?;

    Ok(Json(drugs))
}
#[cfg(test)]
mod tests {
    use crate::{
        domain::{
            doctors::{repository::DoctorsRepositoryFake, service::DoctorsService},
            drugs::{
                models::{Drug, DrugContentType},
                repository::DrugsRepositoryFake,
                service::DrugsService,
            },
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
    use std::sync::Arc;

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
            super::create_drug,
            super::get_drug_by_id,
            super::get_drugs_with_pagination,
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    async fn creates_and_gets_drug_by_id() {
        let client = create_api_client().await;

        let created_drug_response = client
            .post("/drugs")
            .header(ContentType::JSON)
            .body(r#"{"name": "Drug 1", "pills_count": 30, "mg_per_pill": 300, "content_type": "SOLID_PILLS"}"#)
            .dispatch()
            .await;

        assert_eq!(created_drug_response.status(), Status::Created);

        let created_drug: Drug =
            json::from_str(&created_drug_response.into_string().await.unwrap()).unwrap();

        assert_eq!(created_drug.name, "Drug 1");
        assert_eq!(created_drug.pills_count, Some(30));
        assert_eq!(created_drug.mg_per_pill, Some(300));
        assert_eq!(created_drug.content_type, DrugContentType::SolidPills);

        let response = client
            .get(format!("/drugs/{}", created_drug.id))
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);
    }

    #[tokio::test]
    async fn create_drug_returns_unprocessable_entity_with_invalid_data() {
        let client = create_api_client().await;
        assert_eq!(client
            .post("/drugs")
            .header(ContentType::JSON)
            .body(r#"{"name": "Drug 1", "pills_count": "30", "mg_per_pill": 300, "content_type": "SOLID_PILLS"}"#)
            .dispatch()
            .await.status(), Status::UnprocessableEntity);

        assert_eq!(client
            .post("/drugs")
            .header(ContentType::JSON)
            .body(r#"{"name": "Drug 1", "pills_count": 30, "ml_per_pill": 300, "content_type": "SOLID_PILLS"}"#)
            .dispatch()
            .await.status(), Status::UnprocessableEntity);

        assert_eq!(client
            .post("/drugs")
            .header(ContentType::JSON)
            .body(r#"{"name": "Drug 2", "pills_count": 30, "volume_ml": 300, "content_type": "LIQUID_PILLS"}"#)
            .dispatch()
            .await.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn get_drug_by_id_returns_unprocessable_entity_if_id_param_is_invalid() {
        let client = create_api_client().await;

        let request = client.get("/drugs/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn get_drug_by_id_returns_not_found_if_such_doctor_does_not_exist() {
        let client = create_api_client().await;

        let request = client
            .get("/drugs/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[tokio::test]
    async fn gets_drugs_with_pagination() {
        let client = create_api_client().await;
        client
            .post("/drugs")
            .body(r#"{"name":"Drug 1", "pills_count":30, "mg_per_pill":300, "content_type":"SOLID_PILLS"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/drugs")
            .body(r#"{"name":"Drug 2", "pills_count":20, "ml_per_pill":200, "content_type":"LIQUID_PILLS"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/drugs")
            .body(r#"{"name":"Drug 3", "volume_ml":1000, "content_type":"BOTTLE_OF_LIQUID"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/drugs")
            .body(r#"{"name":"Drug 4", "pills_count":10, "mg_per_pill":400, "content_type":"SOLID_PILLS"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        let response = client
            .get("/drugs?page=1&page_size=2")
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let doctors: Vec<Drug> = json::from_str(&response.into_string().await.unwrap()).unwrap();

        assert_eq!(doctors.len(), 2);
    }

    #[tokio::test]
    async fn get_drugs_with_pagination_returns_unprocessable_entity_if_page_or_page_size_is_invalid(
    ) {
        let client = create_api_client().await;

        assert_eq!(
            client
                .get("/drugs?page=-1")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );

        assert_eq!(
            client
                .get("/drugs?page_size=0")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }
}
