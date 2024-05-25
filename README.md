###### Run database:
- `docker compose up` (requires having docker-desktop installed and added to PATH)

###### Run unit tests:
- `cargo watch -x test`

###### Run development:
- `cargo run`

#### TODO:
[ ] pharmacists controller
[ ] drugs controller
[ ] patients controller
[ ] prescriptions controller
[ ] explicit tests for non-existing relations when creating prescription (for instance if trying to create prescription using doctor_id that doesn't exist)


