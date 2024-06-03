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
            // super::create_prescription,
            // super::get_prescription_by_id,
            // super::get_prescriptions_with_pagination,
            // super::fill_prescription
        ];

        let rocket = rocket::build().manage(context).mount("/", routes);

        Client::tracked(rocket).await.unwrap()
    }
}
