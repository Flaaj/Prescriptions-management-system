pub async fn create_tables(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(r#"DROP TABLE IF EXISTS prescribed_drugs;"#)
        .execute(pool)
        .await?;
    sqlx::query!(r#"DROP TABLE IF EXISTS prescriptions;"#)
        .execute(pool)
        .await?;
    sqlx::query!(r#"DROP TYPE IF EXISTS prescriptiontype;"#)
        .execute(pool)
        .await?;

    sqlx::query!(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'prescriptiontype') THEN
            CREATE TYPE prescriptiontype AS ENUM ('regular', 'forantibiotics', 'forchronicdiseasedrugs', 'forimmunologicaldrugs');
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
            prescription_type PrescriptionType NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            end_date TIMESTAMPTZ NOT NULL,
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
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
            created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
        );"#
    )
    .execute(pool)
    .await?;

    Ok(())
}
