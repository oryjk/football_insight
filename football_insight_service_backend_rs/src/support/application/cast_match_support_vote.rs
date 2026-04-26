use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::support::{
    domain::support::{
        SupportMatchDetail, refresh_match_detail, validate_support_window,
        validate_supported_team_in_match, validate_vote_team,
    },
    ports::support_repository::SupportRepository,
};

pub struct CastMatchSupportVoteUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl CastMatchSupportVoteUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        user_id: Uuid,
        match_id: i64,
        supported_team_id: i64,
    ) -> anyhow::Result<SupportMatchDetail> {
        self.execute_at(user_id, match_id, supported_team_id, Utc::now())
            .await
    }

    pub async fn execute_at(
        &self,
        user_id: Uuid,
        match_id: i64,
        supported_team_id: i64,
        now: chrono::DateTime<Utc>,
    ) -> anyhow::Result<SupportMatchDetail> {
        let context = self.repository.get_user_context(user_id).await?;
        validate_vote_team(
            context.favorite_team.as_ref().map(|item| item.team_id),
            supported_team_id,
        )?;

        let mut match_detail = self
            .repository
            .find_match_detail(match_id, Some(user_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("support match not found"))?;

        refresh_match_detail(&mut match_detail, now);
        validate_supported_team_in_match(&match_detail, supported_team_id)?;
        validate_support_window(match_detail.support_window_status)?;

        if match_detail.viewer.has_supported {
            anyhow::bail!("you have already supported this match");
        }

        self.repository
            .create_vote(user_id, match_id, supported_team_id)
            .await?;

        let mut updated_detail = self
            .repository
            .find_match_detail(match_id, Some(user_id))
            .await?
            .ok_or_else(|| anyhow::anyhow!("support match not found"))?;
        refresh_match_detail(&mut updated_detail, now);

        Ok(updated_detail)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::support::{
        application::cast_match_support_vote::CastMatchSupportVoteUseCase,
        domain::support::{
            SupportMatchDetail, SupportMatchTeam, SupportTeamSummary, SupportUserContext,
            SupportViewerState, SupportWindowStatus,
        },
        ports::support_repository::SupportRepository,
    };

    struct FakeRepository {
        favorite_team: Option<SupportTeamSummary>,
        detail: Mutex<SupportMatchDetail>,
    }

    #[async_trait]
    impl SupportRepository for FakeRepository {
        async fn list_teams(&self) -> anyhow::Result<Vec<SupportTeamSummary>> {
            unreachable!()
        }

        async fn get_user_context(&self, user_id: Uuid) -> anyhow::Result<SupportUserContext> {
            Ok(SupportUserContext {
                user_id,
                favorite_team: self.favorite_team.clone(),
            })
        }

        async fn set_favorite_team(
            &self,
            _user_id: Uuid,
            _team_id: i64,
        ) -> anyhow::Result<SupportTeamSummary> {
            unreachable!()
        }

        async fn find_matches_for_team(
            &self,
            _team_id: i64,
            _viewer_user_id: Option<Uuid>,
            _now: chrono::DateTime<Utc>,
        ) -> anyhow::Result<Vec<SupportMatchDetail>> {
            unreachable!()
        }

        async fn find_match_detail(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Option<SupportMatchDetail>> {
            Ok(Some(self.detail.lock().expect("detail").clone()))
        }

        async fn create_vote(
            &self,
            user_id: Uuid,
            _match_id: i64,
            supported_team_id: i64,
        ) -> anyhow::Result<()> {
            let mut detail = self.detail.lock().expect("detail");
            detail.viewer.user_id = Some(user_id);
            detail.viewer.supported_team_id = Some(supported_team_id);
            detail.viewer.has_supported = true;
            detail.home_team.support_count = 81;
            detail.home_team.support_share_pct = 68;
            detail.total_support_count = 121;
            Ok(())
        }
    }

    #[tokio::test]
    async fn execute_casts_vote_for_favorite_team() {
        let user_id = Uuid::new_v4();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
            .single()
            .expect("valid now");
        let use_case = CastMatchSupportVoteUseCase::new(Arc::new(FakeRepository {
            favorite_team: Some(SupportTeamSummary {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                avatar_storage_url: Some("chengdu.png".to_string()),
                rank_no: Some(2),
            }),
            detail: Mutex::new(build_detail(false)),
        }));

        let result = use_case
            .execute_at(user_id, 1, 77680, now)
            .await
            .expect("vote should pass");

        assert!(result.viewer.has_supported);
        assert_eq!(result.viewer.supported_team_id, Some(77680));
        assert_eq!(result.total_support_count, 121);
    }

    #[tokio::test]
    async fn execute_rejects_vote_for_non_favorite_team() {
        let user_id = Uuid::new_v4();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
            .single()
            .expect("valid now");
        let use_case = CastMatchSupportVoteUseCase::new(Arc::new(FakeRepository {
            favorite_team: Some(SupportTeamSummary {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                avatar_storage_url: Some("chengdu.png".to_string()),
                rank_no: Some(2),
            }),
            detail: Mutex::new(build_detail(false)),
        }));

        let error = use_case
            .execute_at(user_id, 1, 22, now)
            .await
            .expect_err("vote should be rejected");

        assert!(error.to_string().contains("favorite team"));
    }

    #[tokio::test]
    async fn execute_rejects_duplicate_vote() {
        let user_id = Uuid::new_v4();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 11, 12, 0, 0)
            .single()
            .expect("valid now");
        let use_case = CastMatchSupportVoteUseCase::new(Arc::new(FakeRepository {
            favorite_team: Some(SupportTeamSummary {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                avatar_storage_url: Some("chengdu.png".to_string()),
                rank_no: Some(2),
            }),
            detail: Mutex::new(build_detail(true)),
        }));

        let error = use_case
            .execute_at(user_id, 1, 77680, now)
            .await
            .expect_err("duplicate vote should be rejected");

        assert!(error.to_string().contains("already supported"));
    }

    fn build_detail(has_supported: bool) -> SupportMatchDetail {
        SupportMatchDetail {
            match_id: 1,
            season: 2026,
            round_number: 6,
            match_date: "2026-04-12".to_string(),
            match_time: "19:35".to_string(),
            status: "upcoming".to_string(),
            kickoff_at: "2026-04-12T11:35:00+00:00".parse().expect("kickoff"),
            support_window_status: SupportWindowStatus::Open,
            countdown_seconds: 0,
            total_support_count: 120,
            home_team: SupportMatchTeam {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                avatar_storage_url: Some("chengdu.png".to_string()),
                score: "-".to_string(),
                support_count: 80,
                support_share_pct: 67,
                season_support_rank: Some(2),
            },
            away_team: SupportMatchTeam {
                team_id: 22,
                team_name: "上海申花".to_string(),
                avatar_storage_url: Some("shenhua.png".to_string()),
                score: "-".to_string(),
                support_count: 40,
                support_share_pct: 33,
                season_support_rank: Some(1),
            },
            viewer: SupportViewerState {
                user_id: None,
                favorite_team_id: Some(77680),
                supported_team_id: has_supported.then_some(77680),
                has_supported,
                can_support: !has_supported,
            },
        }
    }
}
