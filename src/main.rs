use uuid::Uuid;

use crate::domain::prescriptions::component::models::prescription_model::Prescription;

pub mod domain;

fn main() {
    println!("Hello, world!");
    let _prescription = Prescription::new(
        Uuid::new_v4(), //
        Uuid::new_v4(),
        None,
        None,
    );
}
