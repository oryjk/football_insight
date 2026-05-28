use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::seat_swap::{
    domain::{
        SeatSwapContact, SeatSwapDesiredSeat, SeatSwapRequest, SeatSwapRequestStatus, SeatSwapUser,
    },
    ports::seat_swap_repository::{
        MatchedCancellationInput, SeatSwapConfirmation, SeatSwapRepository,
        UpsertSeatSwapRequestInput,
    },
};

#[derive(Clone)]
pub struct PostgresSeatSwapRepository {
    pool: PgPool,
}

impl PostgresSeatSwapRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SeatSwapRepository for PostgresSeatSwapRepository {
    async fn list_active_requests(&self, match_id: i64) -> anyhow::Result<Vec<SeatSwapRequest>> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.id,
                r.match_id,
                r.user_id,
                COALESCE(u.display_name, u.account_identifier, '球迷') AS display_name,
                u.avatar_url,
                r.current_region_key,
                r.current_region_name,
                r.current_row,
                r.current_seat_no,
                r.wechat_id,
                r.phone_number,
                r.seat_swap_notice_enabled,
                r.status,
                r.matched_request_id,
                r.created_at,
                r.updated_at
            FROM f_i_seat_swap_requests r
            JOIN f_i_users u ON u.id = r.user_id
            WHERE r.match_id = $1
              AND r.status IN ('active', 'matched')
            ORDER BY r.updated_at DESC
            "#,
        )
        .bind(match_id)
        .fetch_all(&self.pool)
        .await?;

        let mut requests = Vec::with_capacity(rows.len());
        for row in rows {
            requests.push(self.map_request_row(row).await?);
        }
        Ok(requests)
    }

    async fn find_request_by_user(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>> {
        let Some(row) = sqlx::query(
            r#"
            SELECT
                r.id,
                r.match_id,
                r.user_id,
                COALESCE(u.display_name, u.account_identifier, '球迷') AS display_name,
                u.avatar_url,
                r.current_region_key,
                r.current_region_name,
                r.current_row,
                r.current_seat_no,
                r.wechat_id,
                r.phone_number,
                r.seat_swap_notice_enabled,
                r.status,
                r.matched_request_id,
                r.created_at,
                r.updated_at
            FROM f_i_seat_swap_requests r
            JOIN f_i_users u ON u.id = r.user_id
            WHERE r.match_id = $1
              AND r.user_id = $2
              AND r.status IN ('active', 'matched')
            ORDER BY r.updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(self.map_request_row(row).await?))
    }

    async fn find_request_by_id(
        &self,
        request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>> {
        let Some(row) = sqlx::query(
            r#"
            SELECT
                r.id,
                r.match_id,
                r.user_id,
                COALESCE(u.display_name, u.account_identifier, '球迷') AS display_name,
                u.avatar_url,
                r.current_region_key,
                r.current_region_name,
                r.current_row,
                r.current_seat_no,
                r.wechat_id,
                r.phone_number,
                r.seat_swap_notice_enabled,
                r.status,
                r.matched_request_id,
                r.created_at,
                r.updated_at
            FROM f_i_seat_swap_requests r
            JOIN f_i_users u ON u.id = r.user_id
            WHERE r.id = $1
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(None);
        };

        Ok(Some(self.map_request_row(row).await?))
    }

    async fn find_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
        let row = sqlx::query(
            r#"
            SELECT request_id, target_request_id, confirmed_by_user_id
            FROM f_i_seat_swap_confirmations
            WHERE match_id = $1
              AND request_id = $2
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| SeatSwapConfirmation {
            request_id: row.get("request_id"),
            target_request_id: row.get("target_request_id"),
            confirmed_by_user_id: row.get("confirmed_by_user_id"),
        }))
    }

    async fn list_confirmations_by_request(
        &self,
        match_id: i64,
        request_id: Uuid,
    ) -> anyhow::Result<Vec<SeatSwapConfirmation>> {
        let rows = sqlx::query(
            r#"
            SELECT request_id, target_request_id, confirmed_by_user_id
            FROM f_i_seat_swap_confirmations
            WHERE match_id = $1
              AND request_id = $2
            ORDER BY created_at ASC
            "#,
        )
        .bind(match_id)
        .bind(request_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| SeatSwapConfirmation {
                request_id: row.get("request_id"),
                target_request_id: row.get("target_request_id"),
                confirmed_by_user_id: row.get("confirmed_by_user_id"),
            })
            .collect())
    }

    async fn find_confirmation_between(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapConfirmation>> {
        let row = sqlx::query(
            r#"
            SELECT request_id, target_request_id, confirmed_by_user_id
            FROM f_i_seat_swap_confirmations
            WHERE match_id = $1
              AND request_id = $2
              AND target_request_id = $3
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .bind(request_id)
        .bind(target_request_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| SeatSwapConfirmation {
            request_id: row.get("request_id"),
            target_request_id: row.get("target_request_id"),
            confirmed_by_user_id: row.get("confirmed_by_user_id"),
        }))
    }

    async fn upsert_request(
        &self,
        input: UpsertSeatSwapRequestInput,
    ) -> anyhow::Result<SeatSwapRequest> {
        let request_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;

        let existing_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM f_i_seat_swap_requests
            WHERE match_id = $1
              AND user_id = $2
              AND status = 'active'
            LIMIT 1
            "#,
        )
        .bind(input.match_id)
        .bind(input.user_id)
        .fetch_optional(&mut *tx)
        .await?;

        let final_request_id = existing_id.unwrap_or(request_id);
        if existing_id.is_some() {
            sqlx::query(
                r#"
                UPDATE f_i_seat_swap_requests
                SET current_region_key = $3,
                    current_region_name = $4,
                    current_row = $5,
                    current_seat_no = $6,
                    wechat_id = $7,
                    phone_number = $8,
                    seat_swap_notice_enabled = $9,
                    status = 'active',
                    matched_request_id = NULL,
                    updated_at = NOW()
                WHERE match_id = $1
                  AND user_id = $2
                  AND status = 'active'
                "#,
            )
            .bind(input.match_id)
            .bind(input.user_id)
            .bind(&input.current_region_key)
            .bind(&input.current_region_name)
            .bind(&input.current_row)
            .bind(&input.current_seat_no)
            .bind(&input.wechat_id)
            .bind(&input.phone_number)
            .bind(input.mini_program_notice_enabled)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                r#"
                INSERT INTO f_i_seat_swap_requests (
                    id,
                    match_id,
                    user_id,
                    current_region_key,
                    current_region_name,
                    current_row,
                    current_seat_no,
                    wechat_id,
                    phone_number,
                    seat_swap_notice_enabled,
                    status,
                    created_at,
                    updated_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'active', NOW(), NOW())
                "#,
            )
            .bind(final_request_id)
            .bind(input.match_id)
            .bind(input.user_id)
            .bind(&input.current_region_key)
            .bind(&input.current_region_name)
            .bind(&input.current_row)
            .bind(&input.current_seat_no)
            .bind(&input.wechat_id)
            .bind(&input.phone_number)
            .bind(input.mini_program_notice_enabled)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query("DELETE FROM f_i_seat_swap_desired_seats WHERE request_id = $1")
            .bind(final_request_id)
            .execute(&mut *tx)
            .await?;

        for (index, seat) in input.desired_seats.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO f_i_seat_swap_desired_seats (
                    id,
                    request_id,
                    region_key,
                    region_name,
                    desired_row,
                    desired_seat_no,
                    sort_order
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(final_request_id)
            .bind(&seat.region_key)
            .bind(&seat.region_name)
            .bind(&seat.desired_row)
            .bind(&seat.desired_seat_no)
            .bind(index as i32)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            "DELETE FROM f_i_seat_swap_confirmations WHERE match_id = $1 AND confirmed_by_user_id = $2",
        )
        .bind(input.match_id)
        .bind(input.user_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        self.find_request_by_user(input.match_id, input.user_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("seat swap request not found after upsert"))
    }

    async fn cancel_request(
        &self,
        match_id: i64,
        user_id: Uuid,
    ) -> anyhow::Result<Option<SeatSwapRequest>> {
        let request = self.find_request_by_user(match_id, user_id).await?;
        let Some(request) = request else {
            return Ok(None);
        };

        sqlx::query(
            r#"
            UPDATE f_i_seat_swap_requests
            SET status = 'cancelled',
                updated_at = NOW()
            WHERE id = $1
              AND status = 'active'
            "#,
        )
        .bind(request.id)
        .execute(&self.pool)
        .await?;

        Ok(Some(request))
    }

    async fn set_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<SeatSwapConfirmation> {
        let confirmation_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO f_i_seat_swap_confirmations (
                id,
                match_id,
                request_id,
                target_request_id,
                confirmed_by_user_id,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (match_id, request_id, target_request_id)
            DO UPDATE SET
                confirmed_by_user_id = EXCLUDED.confirmed_by_user_id,
                created_at = NOW()
            "#,
        )
        .bind(confirmation_id)
        .bind(match_id)
        .bind(request_id)
        .bind(target_request_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(SeatSwapConfirmation {
            request_id,
            target_request_id,
            confirmed_by_user_id: user_id,
        })
    }

    async fn delete_confirmation(
        &self,
        match_id: i64,
        request_id: Uuid,
        target_request_id: Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM f_i_seat_swap_confirmations
            WHERE match_id = $1
              AND request_id = $2
              AND target_request_id = $3
              AND confirmed_by_user_id = $4
            "#,
        )
        .bind(match_id)
        .bind(request_id)
        .bind(target_request_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    async fn mark_matched(
        &self,
        request_id: Uuid,
        target_request_id: Uuid,
    ) -> anyhow::Result<bool> {
        let mut tx = self.pool.begin().await?;
        let locked_statuses = sqlx::query(
            r#"
            SELECT id, status
            FROM f_i_seat_swap_requests
            WHERE id IN ($1, $2)
            FOR UPDATE
            "#,
        )
        .bind(request_id)
        .bind(target_request_id)
        .fetch_all(&mut *tx)
        .await?;

        if locked_statuses.len() != 2
            || locked_statuses
                .iter()
                .any(|row| row.get::<String, _>("status") != "active")
        {
            tx.rollback().await?;
            return Ok(false);
        }

        sqlx::query(
            r#"
            UPDATE f_i_seat_swap_requests
            SET status = 'matched',
                matched_request_id = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(request_id)
        .bind(target_request_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            UPDATE f_i_seat_swap_requests
            SET status = 'matched',
                matched_request_id = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(target_request_id)
        .bind(request_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            DELETE FROM f_i_seat_swap_confirmations
            WHERE match_id = (
                SELECT match_id
                FROM f_i_seat_swap_requests
                WHERE id = $1
                LIMIT 1
            )
              AND (
                  request_id IN ($1, $2)
                  OR target_request_id IN ($1, $2)
              )
            "#,
        )
        .bind(request_id)
        .bind(target_request_id)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(true)
    }

    async fn update_status(
        &self,
        request_id: Uuid,
        status: SeatSwapRequestStatus,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE f_i_seat_swap_requests
            SET status = $2,
                updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(request_id)
        .bind(status.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn cancel_matched_pair(
        &self,
        request_id: Uuid,
        target_request_id: Uuid,
    ) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            r#"
            UPDATE f_i_seat_swap_requests
            SET status = 'active',
                matched_request_id = NULL,
                updated_at = NOW()
            WHERE id IN ($1, $2)
            "#,
        )
        .bind(request_id)
        .bind(target_request_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            DELETE FROM f_i_seat_swap_confirmations
            WHERE (
                request_id = $1
                AND target_request_id = $2
            )
               OR (
                request_id = $2
                AND target_request_id = $1
            )
            "#,
        )
        .bind(request_id)
        .bind(target_request_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn insert_matched_cancellation(
        &self,
        input: MatchedCancellationInput,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_seat_swap_cancellations (
                id,
                match_id,
                request_id,
                target_request_id,
                cancelled_by_user_id,
                reason,
                evidence_object_key,
                evidence_url,
                created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(input.match_id)
        .bind(input.request_id)
        .bind(input.target_request_id)
        .bind(input.cancelled_by_user_id)
        .bind(input.reason)
        .bind(input.evidence_object_key)
        .bind(input.evidence_url)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

impl PostgresSeatSwapRepository {
    async fn map_request_row(&self, row: sqlx::postgres::PgRow) -> anyhow::Result<SeatSwapRequest> {
        let request_id: Uuid = row.get("id");
        let desired_seats = sqlx::query(
            r#"
            SELECT region_key, region_name, desired_row, desired_seat_no
            FROM f_i_seat_swap_desired_seats
            WHERE request_id = $1
            ORDER BY sort_order ASC, region_name ASC
            "#,
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|row| SeatSwapDesiredSeat {
            region_key: row.get("region_key"),
            region_name: row.get("region_name"),
            desired_row: row.get("desired_row"),
            desired_seat_no: row.get("desired_seat_no"),
        })
        .collect();

        let wechat_id: Option<String> = row.get("wechat_id");
        let phone_number: Option<String> = row.get("phone_number");
        let status: String = row.get("status");

        Ok(SeatSwapRequest {
            id: request_id,
            match_id: row.get("match_id"),
            user: SeatSwapUser {
                user_id: row.get("user_id"),
                display_name: row.get("display_name"),
                avatar_url: row.get("avatar_url"),
            },
            current_region_key: row.get("current_region_key"),
            current_region_name: row.get("current_region_name"),
            current_row: row.get("current_row"),
            current_seat_no: row.get("current_seat_no"),
            desired_seats,
            contact: SeatSwapContact::new(wechat_id, phone_number).map_err(|error| {
                anyhow::anyhow!("invalid seat swap contact in database: {error:?}")
            })?,
            seat_swap_notice_enabled: row.get("seat_swap_notice_enabled"),
            status: SeatSwapRequestStatus::try_from(status.as_str()).map_err(|error| {
                anyhow::anyhow!("invalid seat swap status in database: {error:?}")
            })?,
            matched_request_id: row.get("matched_request_id"),
            created_at: row.get::<DateTime<Utc>, _>("created_at"),
            updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seat_swap::domain::SeatSwapValidationError;

    #[test]
    fn request_status_round_trips_database_values() {
        assert_eq!(
            SeatSwapRequestStatus::try_from("active").expect("active"),
            SeatSwapRequestStatus::Active
        );
        assert_eq!(SeatSwapRequestStatus::Matched.as_str(), "matched");
        assert_eq!(
            SeatSwapRequestStatus::try_from("bad").expect_err("invalid"),
            SeatSwapValidationError::InvalidStatus
        );
    }
}
