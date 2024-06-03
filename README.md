# Prescriptions Management System

A system for managing prescriptions.

#### Supported use cases:
- Adding a new doctor to database
- Adding a new patient to database
- Adding a new pharmacist to database
- Adding a new drug to database
- prescribing drugs for patients by doctors
- filling a prescription by pharmacists

###### Run application:
- `docker compose up -d` (requires having docker-desktop installed and added to PATH)

###### Run unit tests:
- `cargo watch -x test`

###### Run development:
- `cargo run`

###### Hosted preview:
- base url: https://prescriptions-management-system.onrender.com/
- swagger: https://prescriptions-management-system.onrender.com/swagger-ui

#### TODO:
- [ ] authorization for doctors and pharmacists
- [ ] images service for storing drug images
- [ ] require prescription code to fill
- [ ] add deleting to each collection
- [ ] add updating to each collection
- [ ] add phone number to patient collection and send sms with prescription code
- [ ] drug variants (e.g. 10 pills | 20 pills | 50 pills)


