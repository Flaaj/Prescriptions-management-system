use std::borrow::Borrow;

use rocket::{http::Status, post, response::status, routes, serde::json::Json, Route};

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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDoctorDto {
    pub name: String,
    pub pesel_number: String,
    pub pwz_number: String,
}

#[post("/", format = "application/json", data = "<dto>")]
async fn create_doctor(
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
