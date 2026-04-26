use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::support::{
    domain::support::{SupportProfileView, refresh_match_detail},
    ports::support_repository::SupportRepository,
};

pub struct GetSupportProfileUseCase {
    repository: Arc<dyn SupportRepository>,
}

impl GetSupportProfileUseCase {
    pub fn new(repository: Arc<dyn SupportRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, user_id: Uuid) -> anyhow::Result<SupportProfileView> {
        self.execute_at(user_id, Utc::now()).await
    }

    pub async fn execute_at(
        &self,
        user_id: Uuid,
        now: chrono::DateTime<Utc>,
    ) -> anyhow::Result<SupportProfileView> {
        let context = self.repository.get_user_context(user_id).await?;
        let favorite_team = context.favorite_team.clone();

        let Some(favorite_team) = favorite_team.clone() else {
            return Ok(SupportProfileView {
                favorite_team: None,
                next_match: None,
            });
        };

        let mut next_match = self
            .repository
            .find_matches_for_team(favorite_team.team_id, Some(user_id), now)
            .await?
            .into_iter()
            .find(|item| item.kickoff_at > now);

        if let Some(match_detail) = next_match.as_mut() {
            refresh_match_detail(match_detail, now);
        }

        Ok(SupportProfileView {
            favorite_team: Some(favorite_team),
            next_match,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::support::{
        application::get_support_profile::GetSupportProfileUseCase,
        domain::support::{
            SupportMatchDetail, SupportMatchTeam, SupportTeamSummary, SupportUserContext,
            SupportViewerState, SupportWindowStatus,
        },
        ports::support_repository::SupportRepository,
    };

    struct FakeRepository {
        favorite_team: Option<SupportTeamSummary>,
        matches: Vec<SupportMatchDetail>,
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
            Ok(self.matches.clone())
        }

        async fn find_match_detail(
            &self,
            _match_id: i64,
            _viewer_user_id: Option<Uuid>,
        ) -> anyhow::Result<Option<SupportMatchDetail>> {
            unreachable!()
        }

        async fn create_vote(
            &self,
            _user_id: Uuid,
            _match_id: i64,
            _supported_team_id: i64,
        ) -> anyhow::Result<()> {
            unreachable!()
        }
    }

    #[tokio::test]
    async fn execute_returns_next_match_for_favorite_team() {
        let user_id = Uuid::new_v4();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 10, 10, 0, 0)
            .single()
            .expect("valid now");
        let use_case = GetSupportProfileUseCase::new(Arc::new(FakeRepository {
            favorite_team: Some(SupportTeamSummary {
                team_id: 77680,
                team_name: "成都蓉城".to_string(),
                avatar_storage_url: Some("chengdu.png".to_string()),
                rank_no: Some(2),
            }),
            matches: vec![build_match("2026-04-12T11:35:00+00:00")],
        }));

        let result = use_case.execute_at(user_id, now).await.expect("profile");

        assert_eq!(
            result.favorite_team.as_ref().map(|item| item.team_id),
            Some(77680)
        );
        assert_eq!(
            result.next_match.as_ref().map(|item| item.match_id),
            Some(1)
        );
        assert_eq!(
            result
                .next_match
                .as_ref()
                .map(|item| item.support_window_status.as_str()),
            Some("open")
        );
    }

    #[tokio::test]
    async fn execute_returns_empty_state_without_favorite_team() {
        let user_id = Uuid::new_v4();
        let now = Utc
            .with_ymd_and_hms(2026, 4, 10, 10, 0, 0)
            .single()
            .expect("valid now");
        let use_case = GetSupportProfileUseCase::new(Arc::new(FakeRepository {
            favorite_team: None,
            matches: vec![build_match("2026-04-12T11:35:00+00:00")],
        }));

        let result = use_case.execute_at(user_id, now).await.expect("profile");

        assert!(result.favorite_team.is_none());
        assert!(result.next_match.is_none());
    }

    fn build_match(kickoff: &str) -> SupportMatchDetail {
        SupportMatchDetail {
            match_id: 1,
            season: 2026,
            round_number: 6,
            match_date: "2026-04-12".to_string(),
            match_time: "19:35".to_string(),
            status: "upcoming".to_string(),
            kickoff_at: kickoff.parse().expect("kickoff"),
            support_window_status: SupportWindowStatus::Locked,
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
                supported_team_id: None,
                has_supported: false,
                can_support: false,
            },
        }
    }
}
