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
            id UUID PRIMARY KEY,
            patient_id UUID NOT NULL,
            doctor_id UUID NOT NULL,
            prescription_type PrescriptionType NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            end_date TIMESTAMPTZ NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    sqlx::query!(
        r#"
        CREATE TABLE IF NOT EXISTS prescribed_drugs (
            id UUID PRIMARY KEY,
            prescription_id UUID NOT NULL,
            drug_id UUID NOT NULL,
            quantity INT NOT NULL
        );"#
    )
    .execute(pool)
    .await?;

    Ok(())
}