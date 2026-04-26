use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use crate::auth::{
    domain::{
        membership::{
            MembershipTierRule, membership_tier_rank, resolve_referrer_membership_tier_from_rules,
            resolve_ticket_watch_poll_interval_seconds,
        },
        user::{AuthUser, StoredAuthUser, resolve_effective_membership_tier},
        wechat::{IssuedInviteCode, WechatOauthProfile},
    },
    ports::auth_repository::AuthRepository,
};
use crate::system_config::domain::public_system_config::parse_membership_tier_rules;

#[derive(Clone)]
pub struct PostgresAuthRepository {
    pool: PgPool,
}

impl PostgresAuthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn generate_invite_code() -> String {
        let raw = Uuid::new_v4().simple().to_string().to_uppercase();
        format!("FI-{}-{}", &raw[..6], &raw[6..12])
    }

    fn generate_wechat_account_identifier(open_id: &str) -> String {
        format!("wx_{}", open_id)
    }

    async fn load_membership_tier_rules(&self) -> anyhow::Result<Vec<MembershipTierRule>> {
        let value = sqlx::query_scalar::<_, String>(
            r#"
            SELECT config_value
              FROM f_i_system_configs
             WHERE config_key = 'membership_tier_rules'
             LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(parse_membership_tier_rules(value.as_deref()))
    }

    async fn load_membership_tier_rules_in_tx(
        tx: &mut Transaction<'_, Postgres>,
    ) -> anyhow::Result<Vec<MembershipTierRule>> {
        let value = sqlx::query_scalar::<_, String>(
            r#"
            SELECT config_value
              FROM f_i_system_configs
             WHERE config_key = 'membership_tier_rules'
             LIMIT 1
            "#,
        )
        .fetch_optional(&mut **tx)
        .await?;

        Ok(parse_membership_tier_rules(value.as_deref()))
    }

    async fn validate_unused_invite_code(
        tx: &mut Transaction<'_, Postgres>,
        invite_code: &str,
    ) -> anyhow::Result<()> {
        let invite = sqlx::query_scalar::<_, String>(
            r#"
            SELECT code
              FROM f_i_invite_codes
             WHERE code = $1
               AND is_active = TRUE
               AND used_by_user_id IS NULL
             FOR UPDATE
            "#,
        )
        .bind(invite_code)
        .fetch_optional(&mut **tx)
        .await?;

        if invite.is_none() {
            anyhow::bail!("invite code is invalid or already used");
        }

        Ok(())
    }

    async fn resolve_invite_code_official_wechat_open_id(
        tx: &mut Transaction<'_, Postgres>,
        invite_code: &str,
    ) -> anyhow::Result<Option<String>> {
        sqlx::query_scalar::<_, String>(
            r#"
            SELECT issued_for_wechat_open_id
              FROM f_i_invite_codes
             WHERE code = $1
             LIMIT 1
            "#,
        )
        .bind(invite_code)
        .fetch_optional(&mut **tx)
        .await
        .map_err(Into::into)
    }

    async fn resolve_referrer_user_id(
        tx: &mut Transaction<'_, Postgres>,
        referral_code: Option<&str>,
    ) -> anyhow::Result<Option<Uuid>> {
        let Some(referral_code) = referral_code
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        let referrer_user_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT used_by_user_id
              FROM f_i_invite_codes
             WHERE code = $1
               AND used_by_user_id IS NOT NULL
             LIMIT 1
            "#,
        )
        .bind(referral_code)
        .fetch_optional(&mut **tx)
        .await?;

        referrer_user_id
            .ok_or_else(|| anyhow::anyhow!("referral code is invalid"))
            .map(Some)
    }

    async fn record_referral_and_upgrade_referrer(
        tx: &mut Transaction<'_, Postgres>,
        referrer_user_id: Uuid,
        referred_user_id: Uuid,
        referral_code: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_user_referrals (
                id,
                referrer_user_id,
                referred_user_id,
                referral_invite_code
            ) VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(referrer_user_id)
        .bind(referred_user_id)
        .bind(referral_code)
        .execute(&mut **tx)
        .await?;

        let referral_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
              FROM f_i_user_referrals
             WHERE referrer_user_id = $1
            "#,
        )
        .bind(referrer_user_id)
        .fetch_one(&mut **tx)
        .await?;

        let current_tier = sqlx::query_scalar::<_, String>(
            "SELECT membership_tier FROM f_i_users WHERE id = $1 LIMIT 1",
        )
        .bind(referrer_user_id)
        .fetch_one(&mut **tx)
        .await?;

        let membership_tier_rules = Self::load_membership_tier_rules_in_tx(tx).await?;
        let next_tier =
            resolve_referrer_membership_tier_from_rules(referral_count, &membership_tier_rules);
        if membership_tier_rank(next_tier) > membership_tier_rank(&current_tier) {
            sqlx::query(
                r#"
                UPDATE f_i_users
                   SET membership_tier = $2,
                       updated_at = NOW()
                 WHERE id = $1
                "#,
            )
            .bind(referrer_user_id)
            .bind(next_tier)
            .execute(&mut **tx)
            .await?;
        }

        Ok(())
    }

    async fn create_invited_user(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        let mut tx = self.pool.begin().await?;
        Self::validate_unused_invite_code(&mut tx, invite_code).await?;
        let referrer_user_id = Self::resolve_referrer_user_id(&mut tx, referral_code).await?;
        let official_wechat_open_id =
            Self::resolve_invite_code_official_wechat_open_id(&mut tx, invite_code).await?;
        let membership_tier_rules = Self::load_membership_tier_rules_in_tx(&mut tx).await?;

        let account_identifier_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_users WHERE account_identifier = $1",
        )
        .bind(account_identifier)
        .fetch_one(&mut *tx)
        .await?;

        if account_identifier_exists > 0 {
            anyhow::bail!("phone number is already registered");
        }

        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            INSERT INTO f_i_users (
                id,
                account_identifier,
                display_name,
                password_hash,
                membership_tier,
                official_wechat_open_id
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id,
                      account_identifier,
                      display_name,
                      (
                          SELECT code
                            FROM f_i_invite_codes AS invite_codes
                           WHERE invite_codes.used_by_user_id = f_i_users.id
                           LIMIT 1
                      ) AS invite_code,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      membership_tier,
                      membership_expires_at,
                      CASE
                          WHEN official_wechat_open_id IS NULL THEN TRUE
                          ELSE EXISTS (
                              SELECT 1
                                FROM f_i_wechat_followers AS followers
                               WHERE followers.open_id = f_i_users.official_wechat_open_id
                                 AND followers.unsubscribed_at IS NULL
                          )
                      END AS membership_benefits_enabled,
                      created_at,
                      updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(account_identifier)
        .bind(account_identifier)
        .bind(password_hash)
        .bind("V3")
        .bind(&official_wechat_open_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE f_i_invite_codes
               SET used_by_user_id = $2,
                   used_at = NOW()
             WHERE code = $1
            "#,
        )
        .bind(invite_code)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        if let (Some(referrer_user_id), Some(referral_code)) = (referrer_user_id, referral_code) {
            Self::record_referral_and_upgrade_referrer(
                &mut tx,
                referrer_user_id,
                user.id,
                referral_code.trim(),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(user.into_auth_user(&membership_tier_rules))
    }

    async fn create_invited_wechat_user(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let mut tx = self.pool.begin().await?;
        Self::validate_unused_invite_code(&mut tx, invite_code).await?;
        let referrer_user_id = Self::resolve_referrer_user_id(&mut tx, referral_code).await?;
        let official_wechat_open_id =
            Self::resolve_invite_code_official_wechat_open_id(&mut tx, invite_code).await?;
        let membership_tier_rules = Self::load_membership_tier_rules_in_tx(&mut tx).await?;

        let phone_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_users WHERE account_identifier = $1",
        )
        .bind(phone_number)
        .fetch_one(&mut *tx)
        .await?;

        if phone_exists > 0 {
            anyhow::bail!("phone number is already registered");
        }

        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            INSERT INTO f_i_users (
                id,
                wx_open_id,
                union_id,
                account_identifier,
                display_name,
                avatar_url,
                password_hash,
                membership_tier,
                official_wechat_open_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id,
                      account_identifier,
                      display_name,
                      (
                          SELECT code
                            FROM f_i_invite_codes AS invite_codes
                           WHERE invite_codes.used_by_user_id = f_i_users.id
                           LIMIT 1
                      ) AS invite_code,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      membership_tier,
                      membership_expires_at,
                      CASE
                          WHEN official_wechat_open_id IS NULL THEN TRUE
                          ELSE EXISTS (
                              SELECT 1
                                FROM f_i_wechat_followers AS followers
                               WHERE followers.open_id = f_i_users.official_wechat_open_id
                                 AND followers.unsubscribed_at IS NULL
                          )
                      END AS membership_benefits_enabled,
                      created_at,
                      updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&profile.open_id)
        .bind(&profile.union_id)
        .bind(phone_number)
        .bind(
            profile
                .display_name
                .clone()
                .unwrap_or_else(|| phone_number.to_string()),
        )
        .bind(&profile.avatar_url)
        .bind(password_hash)
        .bind("V3")
        .bind(&official_wechat_open_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE f_i_invite_codes
               SET used_by_user_id = $2,
                   used_at = NOW()
             WHERE code = $1
            "#,
        )
        .bind(invite_code)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        if let (Some(referrer_user_id), Some(referral_code)) = (referrer_user_id, referral_code) {
            Self::record_referral_and_upgrade_referrer(
                &mut tx,
                referrer_user_id,
                user.id,
                referral_code.trim(),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(user.into_auth_user(&membership_tier_rules))
    }

    async fn create_invited_mini_wechat_user(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let mut tx = self.pool.begin().await?;
        Self::validate_unused_invite_code(&mut tx, invite_code).await?;
        let referrer_user_id = Self::resolve_referrer_user_id(&mut tx, referral_code).await?;
        let official_wechat_open_id =
            Self::resolve_invite_code_official_wechat_open_id(&mut tx, invite_code).await?;
        let membership_tier_rules = Self::load_membership_tier_rules_in_tx(&mut tx).await?;

        let account_identifier = Self::generate_wechat_account_identifier(&profile.open_id);

        let account_identifier_exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM f_i_users WHERE account_identifier = $1",
        )
        .bind(&account_identifier)
        .fetch_one(&mut *tx)
        .await?;

        if account_identifier_exists > 0 {
            anyhow::bail!("wechat account is already registered");
        }

        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            INSERT INTO f_i_users (
                id,
                wx_open_id,
                union_id,
                account_identifier,
                display_name,
                avatar_url,
                membership_tier,
                official_wechat_open_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id,
                      account_identifier,
                      display_name,
                      (
                          SELECT code
                            FROM f_i_invite_codes AS invite_codes
                           WHERE invite_codes.used_by_user_id = f_i_users.id
                           LIMIT 1
                      ) AS invite_code,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      membership_tier,
                      membership_expires_at,
                      CASE
                          WHEN official_wechat_open_id IS NULL THEN TRUE
                          ELSE EXISTS (
                              SELECT 1
                                FROM f_i_wechat_followers AS followers
                               WHERE followers.open_id = f_i_users.official_wechat_open_id
                                 AND followers.unsubscribed_at IS NULL
                          )
                      END AS membership_benefits_enabled,
                      created_at,
                      updated_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(&profile.open_id)
        .bind(&profile.union_id)
        .bind(&account_identifier)
        .bind(
            profile
                .display_name
                .clone()
                .unwrap_or_else(|| "微信用户".to_string()),
        )
        .bind(&profile.avatar_url)
        .bind("V3")
        .bind(&official_wechat_open_id)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE f_i_invite_codes
               SET used_by_user_id = $2,
                   used_at = NOW()
             WHERE code = $1
            "#,
        )
        .bind(invite_code)
        .bind(user.id)
        .execute(&mut *tx)
        .await?;

        if let (Some(referrer_user_id), Some(referral_code)) = (referrer_user_id, referral_code) {
            Self::record_referral_and_upgrade_referrer(
                &mut tx,
                referrer_user_id,
                user.id,
                referral_code.trim(),
            )
            .await?;
        }

        tx.commit().await?;
        Ok(user.into_auth_user(&membership_tier_rules))
    }
}

#[derive(Debug, FromRow)]
struct AuthUserRow {
    id: Uuid,
    account_identifier: String,
    display_name: Option<String>,
    invite_code: Option<String>,
    avatar_url: Option<String>,
    has_wechat_binding: bool,
    membership_tier: String,
    membership_expires_at: Option<DateTime<Utc>>,
    membership_benefits_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<AuthUserRow> for AuthUser {
    fn from(value: AuthUserRow) -> Self {
        Self {
            id: value.id,
            account_identifier: value.account_identifier,
            display_name: value.display_name,
            invite_code: value.invite_code,
            avatar_url: value.avatar_url,
            has_wechat_binding: value.has_wechat_binding,
            membership_tier: value.membership_tier,
            membership_expires_at: value.membership_expires_at,
            membership_benefits_enabled: value.membership_benefits_enabled,
            ticket_watch_poll_interval_seconds: 600,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl AuthUserRow {
    fn into_auth_user(self, membership_tier_rules: &[MembershipTierRule]) -> AuthUser {
        let membership_tier = resolve_effective_membership_tier(
            &self.membership_tier,
            self.membership_expires_at,
            Utc::now(),
        );
        let ticket_watch_poll_interval_seconds =
            resolve_ticket_watch_poll_interval_seconds(&membership_tier, membership_tier_rules);

        AuthUser {
            id: self.id,
            account_identifier: self.account_identifier,
            display_name: self.display_name,
            invite_code: self.invite_code,
            avatar_url: self.avatar_url,
            has_wechat_binding: self.has_wechat_binding,
            membership_tier,
            membership_expires_at: self.membership_expires_at,
            membership_benefits_enabled: self.membership_benefits_enabled,
            ticket_watch_poll_interval_seconds,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, FromRow)]
struct StoredAuthUserRow {
    id: Uuid,
    account_identifier: String,
    display_name: Option<String>,
    invite_code: Option<String>,
    avatar_url: Option<String>,
    has_wechat_binding: bool,
    membership_tier: String,
    membership_expires_at: Option<DateTime<Utc>>,
    membership_benefits_enabled: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    password_hash: String,
}

impl From<StoredAuthUserRow> for StoredAuthUser {
    fn from(value: StoredAuthUserRow) -> Self {
        Self {
            user: AuthUser {
                id: value.id,
                account_identifier: value.account_identifier,
                display_name: value.display_name,
                invite_code: value.invite_code,
                avatar_url: value.avatar_url,
                has_wechat_binding: value.has_wechat_binding,
                membership_tier: value.membership_tier,
                membership_expires_at: value.membership_expires_at,
                membership_benefits_enabled: value.membership_benefits_enabled,
                ticket_watch_poll_interval_seconds: 600,
                created_at: value.created_at,
                updated_at: value.updated_at,
            },
            password_hash: value.password_hash,
        }
    }
}

impl StoredAuthUserRow {
    fn into_stored_auth_user(self, membership_tier_rules: &[MembershipTierRule]) -> StoredAuthUser {
        let membership_tier = resolve_effective_membership_tier(
            &self.membership_tier,
            self.membership_expires_at,
            Utc::now(),
        );
        let ticket_watch_poll_interval_seconds =
            resolve_ticket_watch_poll_interval_seconds(&membership_tier, membership_tier_rules);

        StoredAuthUser {
            user: AuthUser {
                id: self.id,
                account_identifier: self.account_identifier,
                display_name: self.display_name,
                invite_code: self.invite_code,
                avatar_url: self.avatar_url,
                has_wechat_binding: self.has_wechat_binding,
                membership_tier,
                membership_expires_at: self.membership_expires_at,
                membership_benefits_enabled: self.membership_benefits_enabled,
                ticket_watch_poll_interval_seconds,
                created_at: self.created_at,
                updated_at: self.updated_at,
            },
            password_hash: self.password_hash,
        }
    }
}

#[async_trait]
impl AuthRepository for PostgresAuthRepository {
    async fn create_user_with_invite(
        &self,
        invite_code: &str,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_user(invite_code, None, account_identifier, password_hash)
            .await
    }

    async fn create_user_with_invite_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_user(
            invite_code,
            referral_code,
            account_identifier,
            password_hash,
        )
        .await
    }

    async fn find_user_by_account_identifier(
        &self,
        account_identifier: &str,
    ) -> anyhow::Result<Option<StoredAuthUser>> {
        let membership_tier_rules = self.load_membership_tier_rules().await?;
        let user = sqlx::query_as::<_, StoredAuthUserRow>(
            r#"
            SELECT id,
                   account_identifier,
                   display_name,
                   (
                       SELECT code
                         FROM f_i_invite_codes AS invite_codes
                        WHERE invite_codes.used_by_user_id = f_i_users.id
                        LIMIT 1
                   ) AS invite_code,
                   avatar_url,
                   (wx_open_id IS NOT NULL) AS has_wechat_binding,
                   membership_tier,
                   membership_expires_at,
                   CASE
                       WHEN official_wechat_open_id IS NULL THEN TRUE
                       ELSE EXISTS (
                           SELECT 1
                             FROM f_i_wechat_followers AS followers
                            WHERE followers.open_id = f_i_users.official_wechat_open_id
                              AND followers.unsubscribed_at IS NULL
                       )
                   END AS membership_benefits_enabled,
                   created_at,
                   updated_at,
                   password_hash
              FROM f_i_users
             WHERE account_identifier = $1
             LIMIT 1
            "#,
        )
        .bind(account_identifier)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|item| item.into_stored_auth_user(&membership_tier_rules)))
    }

    async fn find_user_by_wechat_open_id(&self, open_id: &str) -> anyhow::Result<Option<AuthUser>> {
        let membership_tier_rules = self.load_membership_tier_rules().await?;
        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT id,
                   account_identifier,
                   display_name,
                   (
                       SELECT code
                         FROM f_i_invite_codes AS invite_codes
                        WHERE invite_codes.used_by_user_id = f_i_users.id
                        LIMIT 1
                   ) AS invite_code,
                   avatar_url,
                   (wx_open_id IS NOT NULL) AS has_wechat_binding,
                   membership_tier,
                   membership_expires_at,
                   CASE
                       WHEN official_wechat_open_id IS NULL THEN TRUE
                       ELSE EXISTS (
                           SELECT 1
                             FROM f_i_wechat_followers AS followers
                            WHERE followers.open_id = f_i_users.official_wechat_open_id
                              AND followers.unsubscribed_at IS NULL
                       )
                   END AS membership_benefits_enabled,
                   created_at,
                   updated_at
              FROM f_i_users
             WHERE wx_open_id = $1
             LIMIT 1
            "#,
        )
        .bind(open_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|item| item.into_auth_user(&membership_tier_rules)))
    }

    async fn bind_wechat_to_user(
        &self,
        user_id: Uuid,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        let membership_tier_rules = self.load_membership_tier_rules().await?;
        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            UPDATE f_i_users
               SET wx_open_id = $2,
                   union_id = $3,
                   display_name = COALESCE(display_name, $4),
                   avatar_url = COALESCE(avatar_url, $5),
                   updated_at = NOW()
             WHERE id = $1
            RETURNING id,
                      account_identifier,
                      display_name,
                      (
                          SELECT code
                            FROM f_i_invite_codes AS invite_codes
                           WHERE invite_codes.used_by_user_id = f_i_users.id
                           LIMIT 1
                      ) AS invite_code,
                      avatar_url,
                      (wx_open_id IS NOT NULL) AS has_wechat_binding,
                      membership_tier,
                      membership_expires_at,
                      CASE
                          WHEN official_wechat_open_id IS NULL THEN TRUE
                          ELSE EXISTS (
                              SELECT 1
                                FROM f_i_wechat_followers AS followers
                               WHERE followers.open_id = f_i_users.official_wechat_open_id
                                 AND followers.unsubscribed_at IS NULL
                          )
                      END AS membership_benefits_enabled,
                      created_at,
                      updated_at
            "#,
        )
        .bind(user_id)
        .bind(&profile.open_id)
        .bind(&profile.union_id)
        .bind(&profile.display_name)
        .bind(&profile.avatar_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(user.into_auth_user(&membership_tier_rules))
    }

    async fn create_user_with_invite_and_wechat(
        &self,
        invite_code: &str,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_wechat_user(invite_code, None, phone_number, password_hash, profile)
            .await
    }

    async fn create_user_with_invite_and_wechat_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        phone_number: &str,
        password_hash: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_wechat_user(
            invite_code,
            referral_code,
            phone_number,
            password_hash,
            profile,
        )
        .await
    }

    async fn create_user_with_invite_and_mini_program_wechat(
        &self,
        invite_code: &str,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_mini_wechat_user(invite_code, None, profile)
            .await
    }

    async fn create_user_with_invite_and_mini_program_wechat_with_referral(
        &self,
        invite_code: &str,
        referral_code: Option<&str>,
        profile: &WechatOauthProfile,
    ) -> anyhow::Result<AuthUser> {
        self.create_invited_mini_wechat_user(invite_code, referral_code, profile)
            .await
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
        let membership_tier_rules = self.load_membership_tier_rules().await?;
        let user = sqlx::query_as::<_, AuthUserRow>(
            r#"
            SELECT id,
                   account_identifier,
                   display_name,
                   (
                       SELECT code
                         FROM f_i_invite_codes AS invite_codes
                        WHERE invite_codes.used_by_user_id = f_i_users.id
                        LIMIT 1
                   ) AS invite_code,
                   avatar_url,
                   (wx_open_id IS NOT NULL) AS has_wechat_binding,
                   membership_tier,
                   membership_expires_at,
                   CASE
                       WHEN official_wechat_open_id IS NULL THEN TRUE
                       ELSE EXISTS (
                           SELECT 1
                             FROM f_i_wechat_followers AS followers
                            WHERE followers.open_id = f_i_users.official_wechat_open_id
                              AND followers.unsubscribed_at IS NULL
                       )
                   END AS membership_benefits_enabled,
                   created_at,
                   updated_at
              FROM f_i_users
             WHERE id = $1
             LIMIT 1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user.map(|item| item.into_auth_user(&membership_tier_rules)))
    }

    async fn record_user_login(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO f_i_user_activity_snapshots (
                user_id,
                last_login_at
            ) VALUES ($1, NOW())
            ON CONFLICT (user_id) DO UPDATE
               SET last_login_at = NOW(),
                   updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn issue_invite_code_for_wechat_follower(
        &self,
        open_id: &str,
    ) -> anyhow::Result<IssuedInviteCode> {
        let mut tx = self.pool.begin().await?;

        let follower = sqlx::query_as::<_, (Option<Uuid>,)>(
            r#"
            SELECT latest_invite_code_id
              FROM f_i_wechat_followers
             WHERE open_id = $1
             FOR UPDATE
            "#,
        )
        .bind(open_id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some((Some(invite_code_id),)) = follower {
            let existing = sqlx::query_scalar::<_, String>(
                r#"
                SELECT code
                  FROM f_i_invite_codes
                 WHERE id = $1
                   AND is_active = TRUE
                   AND used_by_user_id IS NULL
                 LIMIT 1
                "#,
            )
            .bind(invite_code_id)
            .fetch_optional(&mut *tx)
            .await?;

            if let Some(code) = existing {
                sqlx::query(
                    r#"
                    UPDATE f_i_invite_codes
                       SET issued_for_wechat_open_id = COALESCE(issued_for_wechat_open_id, $2)
                     WHERE id = $1
                    "#,
                )
                .bind(invite_code_id)
                .bind(open_id)
                .execute(&mut *tx)
                .await?;

                sqlx::query(
                    r#"
                    UPDATE f_i_wechat_followers
                       SET subscribed_at = NOW(),
                           unsubscribed_at = NULL,
                           subscribe_count = subscribe_count + 1,
                           updated_at = NOW()
                     WHERE open_id = $1
                    "#,
                )
                .bind(open_id)
                .execute(&mut *tx)
                .await?;

                tx.commit().await?;
                return Ok(IssuedInviteCode { code });
            }
        }

        let invite_id = Uuid::new_v4();
        let invite_code = Self::generate_invite_code();

        sqlx::query(
            r#"
            INSERT INTO f_i_invite_codes (id, code, issued_for_wechat_open_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(invite_id)
        .bind(&invite_code)
        .bind(open_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO f_i_wechat_followers (
                id,
                open_id,
                latest_invite_code_id,
                subscribe_count,
                subscribed_at,
                unsubscribed_at
            ) VALUES ($1, $2, $3, 1, NOW(), NULL)
            ON CONFLICT (open_id) DO UPDATE
               SET latest_invite_code_id = EXCLUDED.latest_invite_code_id,
                   subscribe_count = f_i_wechat_followers.subscribe_count + 1,
                   subscribed_at = NOW(),
                   unsubscribed_at = NULL,
                   updated_at = NOW()
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(open_id)
        .bind(invite_id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(IssuedInviteCode { code: invite_code })
    }

    async fn mark_wechat_follower_unsubscribed(&self, open_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE f_i_wechat_followers
               SET unsubscribed_at = NOW(),
                   updated_at = NOW()
             WHERE open_id = $1
            "#,
        )
        .bind(open_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn reset_password_with_invite(
        &self,
        invite_code: &str,
        account_identifier: &str,
        password_hash: &str,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE f_i_users AS users
               SET password_hash = $3,
                   updated_at = NOW()
              FROM f_i_invite_codes AS invite_codes
             WHERE users.account_identifier = $2
               AND invite_codes.code = $1
               AND invite_codes.used_by_user_id = users.id
            "#,
        )
        .bind(invite_code)
        .bind(account_identifier)
        .bind(password_hash)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            anyhow::bail!("invalid account identifier or invite code");
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::auth::ports::user_membership_port::UserMembershipPort for PostgresAuthRepository {
    async fn get_user_open_id(&self, user_id: Uuid) -> anyhow::Result<Option<String>> {
        let open_id: Option<String> =
            sqlx::query_scalar("SELECT wx_open_id FROM f_i_users WHERE id = $1 LIMIT 1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await?;

        Ok(open_id)
    }

    async fn get_user_membership_tier(&self, user_id: Uuid) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            "SELECT membership_tier, membership_expires_at FROM f_i_users WHERE id = $1 LIMIT 1",
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| {
            let membership_tier: String = row.get("membership_tier");
            let membership_expires_at: Option<DateTime<Utc>> = row.get("membership_expires_at");

            resolve_effective_membership_tier(&membership_tier, membership_expires_at, Utc::now())
        }))
    }

    async fn update_user_membership_tier(&self, user_id: Uuid, tier: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE f_i_users
            SET membership_tier = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(tier)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
