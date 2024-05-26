# Prescriptions Management System

A system for managing prescriptions.

#### Supported use cases:
- Adding a new doctor to database
- Adding a new patient to database
- Adding a new pharmacist to database
- Adding a new drug to database
- prescribing drugs for patients by doctors
- filling a prescription by pharmacists

###### Run database:
- `docker compose up` (requires having docker-desktop installed and added to PATH)

###### Run unit tests:
- `cargo watch -x test`

###### Run development:
- `cargo run`

#### TODO:
- [ ] pharmacists controller
- [ ] drugs controller
- [ ] patients controller
- [ ] prescriptions controller
- [ ] authorization
- [ ] images service for storing drug images


