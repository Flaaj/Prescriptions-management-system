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
use rocket::{http::Status, post, response::status, routes, serde::json::Json, Route};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDoctorDto {
    pub name: String,
    pub pesel_number: String,
    pub pwz_number: String,
}

#[post("/", format = "application/json", data = "<dto>")]
pub async fn create_doctor(
    ctx: &Ctx,
    dto: Json<CreateDoctorDto>,
) -> Result<Json<Doctor>, status::Custom<String>> {
    let new_doctor = NewDoctor::new(dto.0.name, dto.0.pwz_number, dto.0.pesel_number)
        .map_err(|err| status::Custom(Status::BadRequest, err.to_string()))?;

    let created_doctor = DoctorsRepository::new(ctx.pool.borrow())
        .create_doctor(new_doctor)
        .await
        .map_err(|err| status::Custom(Status::BadRequest, err.to_string()))?;

    Ok(Json(created_doctor))
}

pub fn get_routes() -> Vec<Route> {
    routes![create_doctor]
}

#[cfg(test)]
mod integration_tests {
    use crate::{create_tables::create_tables, Context};
    use rocket::{
        http::{ContentType, Status},
        local::asynchronous::Client,
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
            .body(r#"{"name":"John Doe", "pesel_number":"96021807250", "pwz_number":"5425740"}"#);
        request.add_header(ContentType::JSON);
        let response = request.dispatch().await;

        assert_eq!(response.status(), Status::Ok);
    }
}
