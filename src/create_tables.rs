pub async fn create_tables(pool: &sqlx::PgPool, drop: bool) -> Result<(), sqlx::Error> {
    if drop {
        sqlx::query!(r#"DROP TABLE IF EXISTS prescribed_drugs;"#)
            .execute(pool)
            .await?;
        sqlx::query!(r#"DROP TABLE IF EXISTS prescriptions;"#)
            .execute(pool)
            .await?;
        sqlx::query!(r#"DROP TYPE IF EXISTS prescription_type;"#)
            .execute(pool)
            .await?;
        sqlx::query!(r#"DROP TABLE IF EXISTS doctors;"#)
            .execute(pool)
            .await?;
    }

    sqlx::query!(
    r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'prescription_type') THEN
            CREATE TYPE prescription_type AS ENUM ('regular', 'for_antibiotics', 'for_chronic_disease_drugs', 'for_immunological_drugs');
            END IF;
        END
        $$;
        "#
    )//
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS prescriptions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            patient_id UUID NOT NULL,
            doctor_id UUID NOT NULL,
            prescription_type prescription_type NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            end_date TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS prescribed_drugs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prescription_id UUID NOT NULL,
            drug_id UUID NOT NULL,
            quantity INT NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS doctors (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR(100) NOT NULL,
            pesel_number VARCHAR(11) NOT NULL,
            pwz_number VARCHAR(7) NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    Ok(())
}
