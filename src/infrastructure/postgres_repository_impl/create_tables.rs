pub async fn create_tables(pool: &sqlx::PgPool, drop: bool) -> Result<(), sqlx::Error> {
    if drop {
        sqlx::query(r#"DROP TABLE IF EXISTS prescription_fills;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS prescribed_drugs;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS drugs;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS prescriptions;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS patients;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS pharmacists;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS doctors;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TABLE IF EXISTS sessions;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS prescription_type;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS drug_content_type;"#)
            .execute(pool)
            .await?;
        sqlx::query(r#"DROP TYPE IF EXISTS user_role;"#)
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'prescription_type') THEN
            CREATE TYPE prescription_type AS ENUM ('regular', 'for_antibiotics', 'for_chronic_disease_drugs', 'for_immunological_drugs');
            END IF;
        END
        $$;"#
    )
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'drug_content_type') THEN
            CREATE TYPE drug_content_type AS ENUM ('solid_pills', 'liquid_pills', 'bottle_of_liquid');
            END IF;
        END
        $$;"#
    )
        .execute(pool)
        .await?;

    sqlx::query(
            r#"
            DO $$
            BEGIN
                IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role') THEN
                CREATE TYPE user_role AS ENUM ('doctor', 'pharmacist');
                END IF;
            END
            $$;"#
        )
            .execute(pool)
            .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS doctors (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            pesel_number VARCHAR(11) UNIQUE NOT NULL,
            pwz_number VARCHAR(7) UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pharmacists (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            pesel_number VARCHAR(11) UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS patients (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            pesel_number VARCHAR(11) UNIQUE NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prescriptions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            patient_id UUID NOT NULL REFERENCES patients(id),
            doctor_id UUID NOT NULL REFERENCES doctors(id),
            prescription_type prescription_type NOT NULL,
            code VARCHAR(8) NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            end_date TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS drugs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            content_type drug_content_type NOT NULL,
            pills_count INT,
            mg_per_pill INT,
            ml_per_pill INT,
            volume_ml INT,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prescribed_drugs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prescription_id UUID NOT NULL REFERENCES prescriptions(id),
            drug_id UUID NOT NULL REFERENCES drugs(id),
            quantity INT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS prescription_fills (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prescription_id UUID UNIQUE NOT NULL REFERENCES prescriptions(id),
            pharmacist_id UUID NOT NULL REFERENCES pharmacists(id),
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            username VARCHAR(100) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            email VARCHAR(255) UNIQUE NOT NULL,
            phone_number VARCHAR(15) NOT NULL,
            role user_role NOT NULL,
            doctor_id UUID,
            pharmacist_id UUID,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            user_id UUID NOT NULL,
            doctor_id UUID,
            pharmacist_id UUID,
            ip_address VARCHAR(255) NOT NULL,
            user_agent VARCHAR(255) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            expires_at TIMESTAMPTZ NOT NULL,
            invalidated_at TIMESTAMPTZ
        );"#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
