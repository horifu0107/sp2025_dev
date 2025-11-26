use shared::{
    config::DatabaseConfig,
    error::{AppError, AppResult}
};
use sqlx::{postgres::PgConnectOptions, PgPool};
use kernel::model::id::{ReservationId};
use uuid::Uuid;
pub mod model;
use sqlx::postgres::PgQueryResult;


fn make_pg_connect_options(cfg: &DatabaseConfig) -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .username(&cfg.username)
        .password(&cfg.password)
        .database(&cfg.database)
}

#[derive(Clone)]
pub struct ConnectionPool(PgPool);

impl ConnectionPool {
    pub fn new(pool: PgPool) -> Self {
        Self(pool)
    }

    pub fn inner_ref(&self) -> &PgPool {
        &self.0
    }
    pub async fn begin(&self) -> AppResult<sqlx::Transaction<'_, sqlx::Postgres>> {
        self.0.begin().await.map_err(AppError::TransactionError)
    }

    pub async fn mark_reminder_as_done(&self, reservation_id: Uuid) -> AppResult<PgQueryResult> {
        let result = sqlx::query(
            r#"
            UPDATE reservations
            SET reminder_is_already = TRUE
            WHERE reservation_id = $1
            "#,
        )
        .bind(reservation_id)
        .execute(self.inner_ref())
        .await
        .map_err(AppError::DbQueryError)?;

        Ok(result)
    }

}

pub fn connect_database_with(cfg: &DatabaseConfig) -> ConnectionPool {
    ConnectionPool(PgPool::connect_lazy_with(make_pg_connect_options(cfg)))
}
