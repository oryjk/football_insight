use anyhow::Context;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL is required to run migrations")?;

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("failed to connect to postgres")?;

    if let Some(sql_file) = std::env::args().nth(1) {
        let sql = std::fs::read_to_string(&sql_file)
            .with_context(|| format!("failed to read sql file: {sql_file}"))?;

        sqlx::raw_sql(&sql)
            .execute(&pool)
            .await
            .with_context(|| format!("failed to execute sql file: {sql_file}"))?;

        println!("sql file applied successfully: {sql_file}");
        return Ok(());
    }

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("failed to apply migrations")?;

    println!("migrations applied successfully");
    Ok(())
}
