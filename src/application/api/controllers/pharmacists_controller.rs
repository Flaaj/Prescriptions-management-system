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
    application::api::utils::{error::ApiError, openapi_responses::get_openapi_responses},
    domain::pharmacists::{
        models::Pharmacist,
        repository::{
            CreatePharmacistRepositoryError, GetPharmacistByIdRepositoryError,
            GetPharmacistsRepositoryError,
        },
        service::{
            CreatePharmacistError, GetPharmacistByIdError, GetPharmacistsWithPaginationError,
        },
    },
    Ctx,
};

fn example_name() -> &'static str {
    "John Doe"
}
fn example_pesel_number() -> &'static str {
    "96021807250"
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreatePharmacistDto {
    #[schemars(example = "example_name")]
    name: String,
    #[schemars(example = "example_pesel_number")]
    pesel_number: String,
}

impl<'r> Responder<'r, 'static> for CreatePharmacistError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::DomainError(message) => (message, Status::UnprocessableEntity),
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    CreatePharmacistRepositoryError::DuplicatedPeselNumber => Status::Conflict,
                    CreatePharmacistRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for CreatePharmacistError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "422",
                "Returned when the name or the pesel_number are incorrect",
            ),
            (
                "409",
                "Returned when pharmacist with given pesel_number exist in the database",
            ),
        ])
    }
}

#[openapi(tag = "Pharmacists")]
#[post("/pharmacists", format = "application/json", data = "<dto>")]
pub async fn create_pharmacist(
    ctx: &Ctx,
    dto: Json<CreatePharmacistDto>,
) -> Result<Created<Json<Pharmacist>>, CreatePharmacistError> {
    let created_pharmacist = ctx
        .pharmacists_service
        .create_pharmacist(dto.0.name, dto.0.pesel_number)
        .await?;

    let location = format!("/pharmacists/{}", created_pharmacist.id);
    Ok(Created::new(location).body(Json(created_pharmacist)))
}

impl<'r> Responder<'r, 'static> for GetPharmacistByIdError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPharmacistByIdRepositoryError::NotFound(_) => Status::NotFound,
                    GetPharmacistByIdRepositoryError::DatabaseError(_) => {
                        Status::InternalServerError
                    }
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPharmacistByIdError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![
            (
                "404",
                "Returned when the the pharmacist with given id doesn't exist",
            ),
            (
                "422",
                "Returned when the the pharmacist_id is not a valid UUID",
            ),
        ])
    }
}

#[openapi(tag = "Pharmacists")]
#[get("/pharmacists/<pharmacist_id>", format = "application/json")]
pub async fn get_pharmacist_by_id(
    ctx: &Ctx,
    pharmacist_id: Uuid,
) -> Result<Json<Pharmacist>, GetPharmacistByIdError> {
    let pharmacist = ctx
        .pharmacists_service
        .get_pharmacist_by_id(pharmacist_id)
        .await?;

    Ok(Json(pharmacist))
}

impl<'r> Responder<'r, 'static> for GetPharmacistsWithPaginationError {
    fn respond_to(self, req: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (message, status) = match self {
            Self::RepositoryError(err) => {
                let message = err.to_string();
                let status = match err {
                    GetPharmacistsRepositoryError::InvalidPaginationParams(_) => {
                        Status::UnprocessableEntity
                    }
                    GetPharmacistsRepositoryError::DatabaseError(_) => Status::InternalServerError,
                };
                (message, status)
            }
        };

        ApiError::build_rocket_response(req, message, status)
    }
}

impl OpenApiResponderInner for GetPharmacistsWithPaginationError {
    fn responses(_: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        get_openapi_responses(vec![(
            "422",
            "Returned when the the page < 0 or page_size < 1",
        )])
    }
}

#[openapi(tag = "Pharmacists")]
#[get("/pharmacists?<page>&<page_size>", format = "application/json")]
pub async fn get_pharmacists_with_pagination(
    ctx: &Ctx,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<Json<Vec<Pharmacist>>, GetPharmacistsWithPaginationError> {
    let pharmacists = ctx
        .pharmacists_service
        .get_pharmacists_with_pagination(page, page_size)
        .await?;

    Ok(Json(pharmacists))
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
        application::api::controllers::fake_api_context::create_api_context,
        domain::pharmacists::models::Pharmacist,
    };

    async fn create_api_client() -> Client {
        let context = create_api_context();

        let routes = routes![
            super::create_pharmacist,
            super::get_pharmacist_by_id,
            super::get_pharmacists_with_pagination
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }

    #[tokio::test]
    async fn creates_pharmacist_and_reads_by_id() {
        let client = create_api_client().await;

        let create_pharmacist_response = client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(create_pharmacist_response.status(), Status::Created);

        let created_pharmacist: Pharmacist =
            json::from_str(&create_pharmacist_response.into_string().await.unwrap()).unwrap();

        assert_eq!(created_pharmacist.name, "John Doex");
        assert_eq!(created_pharmacist.pesel_number, "96021807250");

        let get_pharmacist_by_id_response = client
            .get(format!("/pharmacists/{}", created_pharmacist.id))
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(get_pharmacist_by_id_response.status(), Status::Ok);

        let pharmacist: Pharmacist =
            json::from_str(&get_pharmacist_by_id_response.into_string().await.unwrap()).unwrap();

        assert_eq!(pharmacist.name, "John Doex");
        assert_eq!(pharmacist.pesel_number, "96021807250");
    }

    #[tokio::test]
    async fn create_pharmacist_returns_unprocessable_entity_if_body_has_incorrect_keys() {
        let client = create_api_client().await;

        let request_with_wrong_key = client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250"}"#)
            .header(ContentType::JSON);
        let response = request_with_wrong_key.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn create_pharmacist_returns_unprocessable_entity_if_body_has_incorrect_value_incorrect()
    {
        let client = create_api_client().await;

        let mut request_with_incorrect_value = client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807251"}"#);
        request_with_incorrect_value.add_header(ContentType::JSON);
        let response = request_with_incorrect_value.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn create_pharmacist_returns_conflict_if_pesel_number_is_duplicated() {
        let client = create_api_client().await;

        let request = client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON);
        request.dispatch().await;

        let request_with_duplicated_pesel = client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON);
        let response = request_with_duplicated_pesel.dispatch().await;

        assert_eq!(response.status(), Status::Conflict);
    }

    #[tokio::test]
    async fn get_pharmacist_by_id_returns_unprocessable_entity_if_id_param_is_invalid() {
        let client = create_api_client().await;

        let request = client.get("/pharmacists/10").header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::UnprocessableEntity);
    }

    #[tokio::test]
    async fn get_pharmacist_by_id_returns_not_found_if_such_pharmacist_does_not_exist() {
        let client = create_api_client().await;

        let request = client
            .get("/pharmacists/00000000-0000-0000-0000-000000000000")
            .header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::NotFound);
    }

    #[tokio::test]
    async fn gets_pharmacists_with_pagination() {
        let client = create_api_client().await;
        client
            .post("/pharmacists")
            .body(r#"{"name":"John Doex", "pesel_number":"96021817257"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/pharmacists")
            .body(r#"{"name":"John Doey", "pesel_number":"99031301347"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/pharmacists")
            .body(r#"{"name":"John Doez", "pesel_number":"92022900002"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;
        client
            .post("/pharmacists")
            .body(r#"{"name":"John Doeq", "pesel_number":"96021807250"}"#)
            .header(ContentType::JSON)
            .dispatch()
            .await;

        let response = client
            .get("/pharmacists?page=1&page_size=2")
            .header(ContentType::JSON)
            .dispatch()
            .await;

        assert_eq!(response.status(), Status::Ok);

        let pharmacists: Vec<Pharmacist> =
            json::from_str(&response.into_string().await.unwrap()).unwrap();

        assert_eq!(pharmacists.len(), 2);
    }

    #[tokio::test]
    async fn get_pharmacists_with_pagination_returns_unprocessable_entity_if_page_or_page_size_is_invalid(
    ) {
        let client = create_api_client().await;

        assert_eq!(
            client
                .get("/pharmacists?page=-1")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );

        assert_eq!(
            client
                .get("/pharmacists?page_size=0")
                .header(ContentType::JSON)
                .dispatch()
                .await
                .status(),
            Status::UnprocessableEntity
        );
    }
}
