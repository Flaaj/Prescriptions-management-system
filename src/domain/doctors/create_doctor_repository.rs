use super::create_doctor::NewDoctor;

impl NewDoctor {
    pub async fn commit_to_repository(self, pool: &sqlx::PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"INSERT INTO doctors (name, pwz_number, pesel_number) VALUES ($1, $2, $3)"#,
            self.name,
            self.pwz_number,
            self.pesel_number
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
