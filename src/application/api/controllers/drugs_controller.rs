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
    domain::drugs::{
        entities::{Drug, DrugContentType},
        repository::{GetDrugByIdRepositoryError, GetDrugsRepositoryError},
        service::{CreateDrugError, GetDrugByIdError, GetDrugsWithPaginationError},
    },
};

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
        get_openapi_responses(vec![
            (
                "422",
                "Returned when the quantity parameters dont match the content type (for instance when missing volume_ml from BOTTLE_OF_LIQUID content_type)",
            ),
        ])
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
        get_openapi_responses(vec![
            (
                "404",
                "Returned when the drug with the given id was not found",
            ),
            ("422", "Returned when the drug_id is not a valid UUID"),
        ])
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
        get_openapi_responses(vec![(
            "422",
            "Returned when the the page < 0 or page_size < 1",
        )])
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
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
        routes,
        serde::json,
    };

    use crate::{
        context::setup_context,
        domain::drugs::entities::{Drug, DrugContentType},
        infrastructure::postgres_repository_impl::create_tables::create_tables,
    };

    async fn create_api_client(pool: sqlx::PgPool) -> Client {
        create_tables(&pool, true).await.unwrap();
        let context = setup_context(pool);

        let routes = routes![
            super::create_drug,
            super::get_drug_by_id,
            super::get_drugs_with_pagination,
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[sqlx::test]
    async fn creates_and_gets_drug_by_id(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

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

    #[sqlx::test]
    async fn create_drug_returns_unprocessable_entity_with_invalid_data(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;
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

    #[sqlx::test]
    async fn get_drug_by_id_returns_unprocessable_entity_if_id_param_is_invalid(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

        let request = client.get("/drugs/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[sqlx::test]
    async fn get_drug_by_id_returns_not_found_if_such_drug_does_not_exist(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;

        let request = client
            .get("/drugs/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[sqlx::test]
    async fn gets_drugs_with_pagination(pool: sqlx::PgPool) {
        let client = create_api_client(pool).await;
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

    #[sqlx::test]
    async fn get_drugs_with_pagination_returns_unprocessable_entity_if_page_or_page_size_is_invalid(
        pool: sqlx::PgPool,
    ) {
        let client = create_api_client(pool).await;

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
