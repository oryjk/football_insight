use std::sync::Arc;

use uuid::Uuid;

use crate::{
    auth::{
        domain::membership::membership_tier_rank, ports::user_membership_port::UserMembershipPort,
    },
    match_id_unlock::{
        domain::match_id_unlock::{
            MATCH_ID_UNLOCK_MINIMUM_TIER, MatchIdUnlockError, MatchIdUnlockSource,
        },
        ports::match_id_unlock_repository::MatchIdUnlockRepository,
    },
};

pub struct GetMatchIdEntitlementUseCase {
    repository: Arc<dyn MatchIdUnlockRepository>,
    user_membership_port: Arc<dyn UserMembershipPort>,
}

impl GetMatchIdEntitlementUseCase {
    pub fn new(
        repository: Arc<dyn MatchIdUnlockRepository>,
        user_membership_port: Arc<dyn UserMembershipPort>,
    ) -> Self {
        Self {
            repository,
            user_membership_port,
        }
    }

    pub async fn execute(
        &self,
        input: MatchIdEntitlementInput,
    ) -> anyhow::Result<MatchIdEntitlementView> {
        if !self.repository.match_exists(input.match_id).await? {
            return Err(MatchIdUnlockError::MatchNotFound.into());
        }

        // UserMembershipPort 的实现已应用 resolve_effective_membership_tier，
        // 会员过期会回退到 V3，因此这里拿到的即生效等级。
        let effective_tier = self
            .user_membership_port
            .get_user_membership_tier(input.user_id)
            .await?
            .unwrap_or_else(|| "V1".to_string());

        if membership_tier_rank(&effective_tier)
            >= membership_tier_rank(MATCH_ID_UNLOCK_MINIMUM_TIER)
        {
            return Ok(MatchIdEntitlementView {
                unlocked: true,
                via: Some(MatchIdUnlockSource::Membership),
                effective_tier,
            });
        }

        let unlocked = self
            .repository
            .find_unlock(input.user_id, input.match_id)
            .await?;

        Ok(MatchIdEntitlementView {
            unlocked,
            via: unlocked.then_some(MatchIdUnlockSource::Purchase),
            effective_tier,
        })
    }
}

pub struct MatchIdEntitlementInput {
    pub user_id: Uuid,
    pub match_id: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MatchIdEntitlementView {
    pub unlocked: bool,
    pub via: Option<MatchIdUnlockSource>,
    pub effective_tier: String,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::{GetMatchIdEntitlementUseCase, MatchIdEntitlementInput, MatchIdEntitlementView};
    use crate::{
        auth::ports::user_membership_port::UserMembershipPort,
        match_id_unlock::{
            domain::match_id_unlock::{MatchIdUnlockError, MatchIdUnlockSource},
            ports::match_id_unlock_repository::MatchIdUnlockRepository,
        },
    };

    struct FakeRepository {
        matches_exist: bool,
        unlocked_matches: Vec<i64>,
    }

    #[async_trait]
    impl MatchIdUnlockRepository for FakeRepository {
        async fn match_exists(&self, _match_id: i64) -> anyhow::Result<bool> {
            Ok(self.matches_exist)
        }

        async fn find_unlock(&self, _user_id: Uuid, match_id: i64) -> anyhow::Result<bool> {
            Ok(self.unlocked_matches.contains(&match_id))
        }
    }

    struct FakeUserMembershipPort {
        tier: Option<String>,
    }

    #[async_trait]
    impl UserMembershipPort for FakeUserMembershipPort {
        async fn get_user_open_id(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(Some("openid".to_string()))
        }

        async fn get_user_membership_tier(&self, _user_id: Uuid) -> anyhow::Result<Option<String>> {
            Ok(self.tier.clone())
        }

        async fn update_user_membership_tier(
            &self,
            _user_id: Uuid,
            _tier: &str,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn is_seat_swap_notice_enabled(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<bool>> {
            Ok(Some(false))
        }
    }

    fn use_case(tier: Option<&str>, unlocked_matches: Vec<i64>) -> GetMatchIdEntitlementUseCase {
        GetMatchIdEntitlementUseCase::new(
            Arc::new(FakeRepository {
                matches_exist: true,
                unlocked_matches,
            }),
            Arc::new(FakeUserMembershipPort {
                tier: tier.map(str::to_string),
            }),
        )
    }

    fn input() -> MatchIdEntitlementInput {
        MatchIdEntitlementInput {
            user_id: Uuid::parse_str("aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa").unwrap(),
            match_id: 571,
        }
    }

    #[tokio::test]
    async fn membership_tier_at_v6_unlocks_via_membership() {
        let view = use_case(Some("V6"), vec![])
            .execute(input())
            .await
            .expect("entitlement");

        assert_eq!(
            view,
            MatchIdEntitlementView {
                unlocked: true,
                via: Some(MatchIdUnlockSource::Membership),
                effective_tier: "V6".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn expired_membership_falls_back_below_v6_and_stays_locked() {
        // 过期回退为 V3 由 UserMembershipPort 实现负责，这里模拟其返回值。
        let view = use_case(Some("V3"), vec![])
            .execute(input())
            .await
            .expect("entitlement");

        assert_eq!(
            view,
            MatchIdEntitlementView {
                unlocked: false,
                via: None,
                effective_tier: "V3".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn purchased_match_unlocks_via_purchase_below_v6() {
        let view = use_case(Some("V5"), vec![571])
            .execute(input())
            .await
            .expect("entitlement");

        assert_eq!(
            view,
            MatchIdEntitlementView {
                unlocked: true,
                via: Some(MatchIdUnlockSource::Purchase),
                effective_tier: "V5".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn missing_tier_defaults_to_v1_and_stays_locked() {
        let view = use_case(None, vec![])
            .execute(input())
            .await
            .expect("entitlement");

        assert_eq!(
            view,
            MatchIdEntitlementView {
                unlocked: false,
                via: None,
                effective_tier: "V1".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn missing_match_returns_match_not_found() {
        let use_case = GetMatchIdEntitlementUseCase::new(
            Arc::new(FakeRepository {
                matches_exist: false,
                unlocked_matches: vec![],
            }),
            Arc::new(FakeUserMembershipPort {
                tier: Some("V9".to_string()),
            }),
        );

        let error = use_case
            .execute(input())
            .await
            .expect_err("match should be missing");

        assert!(
            error
                .downcast_ref::<MatchIdUnlockError>()
                .is_some_and(|error| matches!(error, MatchIdUnlockError::MatchNotFound))
        );
    }
}
