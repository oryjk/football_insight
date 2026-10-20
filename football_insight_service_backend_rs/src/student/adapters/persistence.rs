use crate::student::{
    domain::{ServiceCode, StudentConfig, StudentError, StudentOrder, StudentOrderPage},
    ports::StudentRepository,
};
use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

const CONFIG_KEY: &str = "student_features";
pub struct PostgresStudentRepository {
    pool: PgPool,
}
impl PostgresStudentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
fn db_error(error: impl std::fmt::Display) -> StudentError {
    tracing::warn!(%error, "student repository failed");
    StudentError::Internal
}

#[async_trait]
impl StudentRepository for PostgresStudentRepository {
    async fn fail_unissued_order(&self, user_id: Uuid, order_no: &str) -> Result<(), StudentError> {
        sqlx::query("UPDATE f_i_payment_orders p SET status='failed',updated_at=NOW() WHERE p.order_no=$1 AND p.user_id=$2 AND p.product_type=ANY($3) AND p.status='pending' AND NOT EXISTS(SELECT 1 FROM f_i_virtual_payment_orders v WHERE v.order_no=p.order_no)")
            .bind(order_no).bind(user_id).bind(ServiceCode::ALL.into_iter().map(ServiceCode::product_type).collect::<Vec<_>>())
            .execute(&self.pool).await.map_err(db_error)?;
        Ok(())
    }
    async fn config(&self) -> Result<StudentConfig, StudentError> {
        let raw: Option<String> =
            sqlx::query_scalar("SELECT config_value FROM f_i_system_configs WHERE config_key=$1")
                .bind(CONFIG_KEY)
                .fetch_optional(&self.pool)
                .await
                .map_err(db_error)?;
        match raw {
            Some(raw) => serde_json::from_str::<StudentConfig>(&raw)
                .map_err(db_error)?
                .validate()
                .map_err(db_error),
            None => Ok(StudentConfig::default()),
        }
    }
    async fn save_config(
        &self,
        config: StudentConfig,
        admin_id: Uuid,
    ) -> Result<StudentConfig, StudentError> {
        let config = config.validate()?;
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        // Serialize writes even when no config row exists yet.
        sqlx::query("SELECT pg_advisory_xact_lock(hashtext('football.student_features'))")
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let before: Option<String> = sqlx::query_scalar(
            "SELECT config_value FROM f_i_system_configs WHERE config_key=$1 FOR UPDATE",
        )
        .bind(CONFIG_KEY)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        for item in &config.services {
            if item.product_id.is_empty() {
                sqlx::query(
                    "UPDATE f_i_virtual_payment_products SET enabled=FALSE WHERE product_key=$1",
                )
                .bind(item.code.product_key())
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
            } else {
                sqlx::query("INSERT INTO f_i_virtual_payment_products(product_key,name,product_id,price_cents,enabled,kind,description) VALUES($1,$2,$3,$4,$5,'student_service',$6) ON CONFLICT(product_key) DO UPDATE SET name=EXCLUDED.name,product_id=EXCLUDED.product_id,enabled=EXCLUDED.enabled,description=EXCLUDED.description")
                    .bind(item.code.product_key()).bind(&item.title).bind(&item.product_id).bind(item.price_cents).bind(item.enabled).bind(&item.description).execute(&mut *tx).await.map_err(db_error)?;
            }
        }
        let after = serde_json::to_value(&config).map_err(db_error)?;
        sqlx::query("INSERT INTO f_i_system_configs(config_key,config_value,description) VALUES($1,$2,'学员功能说明、联系微信与购买配置') ON CONFLICT(config_key) DO UPDATE SET config_value=EXCLUDED.config_value,updated_at=NOW()")
            .bind(CONFIG_KEY).bind(after.to_string()).execute(&mut *tx).await.map_err(db_error)?;
        sqlx::query("INSERT INTO f_i_admin_audit_logs(id,admin_user_id,action,target_type,target_id,before_json,after_json) VALUES($1,$2,'student.config.set','system_config',$3,$4,$5)")
            .bind(Uuid::new_v4()).bind(admin_id).bind(CONFIG_KEY).bind(before.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())).bind(after).execute(&mut *tx).await.map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(config)
    }
    async fn orders(
        &self,
        user_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<StudentOrderPage, StudentError> {
        let types: Vec<String> = ServiceCode::ALL
            .into_iter()
            .map(ServiceCode::product_type)
            .collect();
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM f_i_payment_orders WHERE user_id=$1 AND product_type=ANY($2)",
        )
        .bind(user_id)
        .bind(&types)
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let rows = sqlx::query("SELECT order_no,product_type,amount_cents,status,paid_at,created_at FROM f_i_payment_orders WHERE user_id=$1 AND product_type=ANY($2) ORDER BY created_at DESC,order_no DESC LIMIT $3 OFFSET $4")
            .bind(user_id).bind(&types).bind(page_size).bind((page-1)*page_size).fetch_all(&mut *tx).await.map_err(db_error)?;
        let items = rows
            .into_iter()
            .map(|row| {
                Ok(StudentOrder {
                    order_no: row.try_get("order_no").map_err(db_error)?,
                    code: ServiceCode::from_product_type(
                        row.try_get::<&str, _>("product_type").map_err(db_error)?,
                    )
                    .ok_or(StudentError::Internal)?,
                    amount_cents: row.try_get("amount_cents").map_err(db_error)?,
                    status: row.try_get("status").map_err(db_error)?,
                    paid_at: row.try_get("paid_at").map_err(db_error)?,
                    created_at: row.try_get("created_at").map_err(db_error)?,
                })
            })
            .collect::<Result<Vec<_>, StudentError>>()?;
        tx.commit().await.map_err(db_error)?;
        Ok(StudentOrderPage {
            items,
            total,
            page,
            page_size,
        })
    }
}
