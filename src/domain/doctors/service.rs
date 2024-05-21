use crate::domain::doctors::{
    models::{Doctor, NewDoctor},
    repository::doctors_repository_trait::DoctorsRepositoryTrait,
};
use uuid::Uuid;

pub enum CreateDoctorError {
    ValidationError(String),
    DatabaseError(String),
}

pub enum GetDoctorByIdError {
    InputError,
    DatabaseError(String),
}

pub enum GetDoctorWithPaginationError {
    InputError(String),
}

#[derive(Clone)]
pub struct DoctorsService<R: DoctorsRepositoryTrait> {
    repo: R,
}

impl<R: DoctorsRepositoryTrait> DoctorsService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_doctor(
        &self,
        name: String,
        pesel_number: String,
        pwz_number: String,
    ) -> Result<Doctor, CreateDoctorError> {
        let new_doctor = NewDoctor::new(name, pwz_number, pesel_number)
            .map_err(|err| CreateDoctorError::ValidationError(err.to_string()))?;

        let created_doctor = self
            .repo
            .create_doctor(new_doctor)
            .await
            .map_err(|err| CreateDoctorError::DatabaseError(err.to_string()))?;

        Ok(created_doctor)
    }

    pub async fn get_doctor_by_id(&self, doctor_id: Uuid) -> Result<Doctor, GetDoctorByIdError> {
        let doctor = self
            .repo
            .get_doctor_by_id(doctor_id)
            .await
            .map_err(|err| GetDoctorByIdError::DatabaseError(err.to_string()))?;

        Ok(doctor)
    }

    pub async fn get_doctors_with_pagination(
        &self,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<Doctor>, GetDoctorWithPaginationError> {
        let doctor = self
            .repo
            .get_doctors(page, page_size)
            .await
            .map_err(|err| GetDoctorWithPaginationError::InputError(err.to_string()))?;

        Ok(doctor)
    }
}

// #[cfg(test)]
// mod integration_tests {
//     use crate::{create_tables::create_tables, domain::doctors::models::Doctor, Context};
//     use rocket::{
//         http::{ContentType, Status},
//         local::asynchronous::Client,
//         serde::json,
//     };
//     use std::sync::Arc;

//     async fn create_api_client(pool: sqlx::PgPool) -> Client {
//         create_tables(&pool, true).await.unwrap();

//         let pool = Arc::new(pool);
//         let rocket = rocket::build()
//             .manage(Context { pool })
//             .mount("/", super::get_routes());

//         Client::tracked(rocket).await.unwrap()
//     }

//     #[sqlx::test]
//     async fn creates_doctor_and_reads_by_id(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;

//         let create_doctor_response = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(create_doctor_response.status(), Status::Created);

//         let created_doctor: Doctor =
//             json::from_str(&create_doctor_response.into_string().await.unwrap()).unwrap();

//         assert_eq!(created_doctor.name, "John Doex");
//         assert_eq!(created_doctor.pesel_number, "96021807250");
//         assert_eq!(created_doctor.pwz_number, "5425740");

//         let get_doctor_by_id_response = client
//             .get(format!("/doctors/{}", created_doctor.id))
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(get_doctor_by_id_response.status(), Status::Ok);

//         let doctor: Doctor =
//             json::from_str(&get_doctor_by_id_response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctor.name, "John Doex");
//         assert_eq!(doctor.pesel_number, "96021807250");
//         assert_eq!(doctor.pwz_number, "5425740");
//     }

//     #[sqlx::test]
//     async fn create_doctor_returns_error_if_body_is_incorrect(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;

//         let request_with_wrong_key = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_numberr":"96021807250", "pwz_number":"5425740"}"#)
//             .header(ContentType::JSON);
//         let response = request_with_wrong_key.dispatch().await;

//         assert_eq!(response.status(), Status::UnprocessableEntity);

//         let mut request_with_incorrect_value = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"96021807251", "pwz_number":"5425740"}"#);
//         request_with_incorrect_value.add_header(ContentType::JSON);
//         let response = request_with_incorrect_value.dispatch().await;

//         assert_eq!(response.status(), Status::BadRequest);
//     }

//     #[sqlx::test]
//     async fn create_doctor_returns_error_if_pwz_or_pesel_numbers_are_duplicated(
//         pool: sqlx::PgPool,
//     ) {
//         let client = create_api_client(pool).await;

//         let request = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"5425740"}"#)
//             .header(ContentType::JSON);
//         let response = request.dispatch().await;

//         assert_eq!(response.status(), Status::Created);

//         let request_with_duplicated_pesel = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"96021807250", "pwz_number":"8463856"}"#)
//             .header(ContentType::JSON);
//         let response = request_with_duplicated_pesel.dispatch().await;

//         assert_eq!(response.status(), Status::BadRequest);

//         let request_with_duplicated_pwz = client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"99031301347", "pwz_number":"5425740"}"#)
//             .header(ContentType::JSON);
//         let response = request_with_duplicated_pwz.dispatch().await;

//         assert_eq!(response.status(), Status::BadRequest);
//     }

//     #[sqlx::test]
//     async fn get_doctor_by_id_returns_error_if_id_param_is_invalid(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;

//         let request = client.get("/doctors/10").header(ContentType::JSON);
//         let response = request.dispatch().await;

//         assert_eq!(response.status(), Status::UnprocessableEntity);
//     }

//     #[sqlx::test]
//     async fn get_doctor_by_id_returns_error_if_such_doctor_does_not_exist(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;

//         let request = client
//             .get("/doctors/00000000-0000-0000-0000-000000000000")
//             .header(ContentType::JSON);
//         let response = request.dispatch().await;

//         assert_eq!(response.status(), Status::BadRequest);
//     }

//     #[sqlx::test]
//     async fn gets_doctors_with_pagination(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;
//         client
//             .post("/doctors")
//             .body(r#"{"name":"John Doex", "pesel_number":"96021817257", "pwz_number":"5425740"}"#)
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;
//         client
//             .post("/doctors")
//             .body(r#"{"name":"John Doey", "pesel_number":"99031301347", "pwz_number":"8463856"}"#)
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;
//         client
//             .post("/doctors")
//             .body(r#"{"name":"John Doez", "pesel_number":"92022900002", "pwz_number":"3123456"}"#)
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;
//         client
//             .post("/doctors")
//             .body(r#"{"name":"John Doeq", "pesel_number":"96021807250", "pwz_number":"5425751"}"#)
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         let response = client
//             .get("/doctors?page=1&page_size=2")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);

//         let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctors.len(), 2);

//         let response = client
//             .get("/doctors?page=1&page_size=3")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);

//         let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctors.len(), 1);

//         let response = client
//             .get("/doctors?page_size=10")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);

//         let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctors.len(), 4);

//         let response = client
//             .get("/doctors?page=1")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);

//         let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctors.len(), 0);

//         let response = client
//             .get("/doctors")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::Ok);

//         let doctors: Vec<Doctor> = json::from_str(&response.into_string().await.unwrap()).unwrap();

//         assert_eq!(doctors.len(), 4);
//     }

//     #[sqlx::test]
//     async fn get_doctors_with_pagination_returns_error_if_params_are_invalid(pool: sqlx::PgPool) {
//         let client = create_api_client(pool).await;

//         let response = client
//             .get("/doctors?page=-1")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::BadRequest);

//         let response = client
//             .get("/doctors?page_size=0")
//             .header(ContentType::JSON)
//             .dispatch()
//             .await;

//         assert_eq!(response.status(), Status::BadRequest);
//     }
// }
