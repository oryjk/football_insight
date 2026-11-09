use super::{
    domain::{
        LineupReadError, PLAYER_LOOKUP_LIMIT, PLAYER_SEARCH_LIMIT, Position, SquadPlayer,
        SquadTeam, normalize_player_query,
    },
    ports::SquadRepository,
};
pub mod my_lineup;
pub mod public_lineup;

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub struct Squad {
    pub team: SquadTeam,
    pub active: Vec<SquadPlayer>,
    pub former: Vec<SquadPlayer>,
}

pub struct LineupService {
    squads: Arc<dyn SquadRepository>,
}

impl LineupService {
    pub fn new(squads: Arc<dyn SquadRepository>) -> Self {
        Self { squads }
    }

    /// 球队名单：现役按位置（门将、后卫、中场、前锋、未知）再按号码（空号排最后）排序；
    /// 前队员先按最后在队赛季倒序，再按同样的位置与号码规则排序。
    pub async fn squad(&self, team_id: i64) -> Result<Squad, LineupReadError> {
        if team_id <= 0 {
            return Err(LineupReadError::Invalid);
        }
        let team = self
            .squads
            .find_team(team_id)
            .await?
            .ok_or(LineupReadError::NotFound)?;
        let (mut active, mut former): (Vec<SquadPlayer>, Vec<SquadPlayer>) = self
            .squads
            .list_squad(team_id)
            .await?
            .into_iter()
            .partition(|player| player.active);
        active.sort_by(by_role);
        former.sort_by(|a, b| b.last_season.cmp(&a.last_season).then(by_role(a, b)));
        Ok(Squad {
            team,
            active,
            former,
        })
    }
}

impl LineupService {
    /// 搜全部中超球员（选人时可以选其他球队的球员）。
    pub async fn search_players(&self, query: &str) -> Result<Vec<SquadPlayer>, LineupReadError> {
        let query = normalize_player_query(query).ok_or(LineupReadError::Invalid)?;
        self.squads
            .search_players(&query, PLAYER_SEARCH_LIMIT)
            .await
    }

    /// 按 ID 补查球员资料：阵容里的外队球员不在本队名单，编辑页靠它显示名字和头像。
    pub async fn find_players(&self, ids: &[i64]) -> Result<Vec<SquadPlayer>, LineupReadError> {
        if ids.is_empty() || ids.len() > PLAYER_LOOKUP_LIMIT || ids.iter().any(|id| *id <= 0) {
            return Err(LineupReadError::Invalid);
        }
        self.squads.find_players(ids).await
    }
}

fn by_role(a: &SquadPlayer, b: &SquadPlayer) -> std::cmp::Ordering {
    Position::rank(a.position)
        .cmp(&Position::rank(b.position))
        .then(
            a.shirt_number
                .unwrap_or(i32::MAX)
                .cmp(&b.shirt_number.unwrap_or(i32::MAX)),
        )
        .then(a.name.cmp(&b.name))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::lineup::domain::SquadTeam;
    use async_trait::async_trait;

    pub(crate) struct FakeSquads {
        pub players: Vec<SquadPlayer>,
        pub missing_table: bool,
    }

    #[async_trait]
    impl SquadRepository for FakeSquads {
        async fn find_team(&self, team_id: i64) -> Result<Option<SquadTeam>, LineupReadError> {
            if self.missing_table {
                return Err(LineupReadError::Unavailable);
            }
            Ok((team_id == 77680).then(|| SquadTeam {
                team_id,
                team_name: "成都蓉城".into(),
                avatar_url: None,
            }))
        }
        async fn list_squad(&self, _team_id: i64) -> Result<Vec<SquadPlayer>, LineupReadError> {
            Ok(self
                .players
                .iter()
                .filter(|p| p.club.is_none())
                .cloned()
                .collect())
        }
        async fn search_players(
            &self,
            query: &str,
            limit: i64,
        ) -> Result<Vec<SquadPlayer>, LineupReadError> {
            Ok(self
                .players
                .iter()
                .filter(|p| p.name.contains(query))
                .take(limit as usize)
                .cloned()
                .map(with_club)
                .collect())
        }
        async fn find_players(&self, ids: &[i64]) -> Result<Vec<SquadPlayer>, LineupReadError> {
            Ok(self
                .players
                .iter()
                .filter(|p| ids.contains(&p.player_id))
                .cloned()
                .map(with_club)
                .collect())
        }
    }

    /// 跨队查询总带所在球队；测试里本队球员算成都蓉城。
    fn with_club(mut player: SquadPlayer) -> SquadPlayer {
        player
            .club
            .get_or_insert(crate::lineup::domain::PlayerClub {
                team_id: 77680,
                team_name: "成都蓉城".into(),
            });
        player
    }

    pub(crate) fn player(
        id: i64,
        number: Option<i32>,
        position: Option<Position>,
        season: i32,
        active: bool,
    ) -> SquadPlayer {
        SquadPlayer {
            player_id: id,
            name: format!("P{id}"),
            shirt_number: number,
            position,
            avatar_url: None,
            last_season: season,
            active,
            club: None,
        }
    }

    fn service(players: Vec<SquadPlayer>) -> LineupService {
        LineupService::new(Arc::new(FakeSquads {
            players,
            missing_table: false,
        }))
    }

    #[tokio::test]
    async fn rejects_non_positive_team_ids() {
        assert_eq!(
            service(vec![]).squad(0).await.unwrap_err(),
            LineupReadError::Invalid
        );
    }

    #[tokio::test]
    async fn unknown_team_is_not_found() {
        assert_eq!(
            service(vec![]).squad(1).await.unwrap_err(),
            LineupReadError::NotFound
        );
    }

    #[tokio::test]
    async fn missing_table_is_unavailable() {
        let s = LineupService::new(Arc::new(FakeSquads {
            players: vec![],
            missing_table: true,
        }));
        assert_eq!(
            s.squad(77680).await.unwrap_err(),
            LineupReadError::Unavailable
        );
    }

    #[tokio::test]
    async fn sorts_active_by_position_then_number_with_blanks_last() {
        let res = service(vec![
            player(1, Some(9), Some(Position::FW), 2026, true),
            player(2, None, Some(Position::MF), 2026, true),
            player(3, Some(6), Some(Position::MF), 2026, true),
            player(4, Some(1), Some(Position::GK), 2026, true),
            player(5, Some(3), None, 2026, true),
        ])
        .squad(77680)
        .await
        .unwrap();
        let ids: Vec<i64> = res.active.iter().map(|p| p.player_id).collect();
        assert_eq!(ids, vec![4, 3, 2, 1, 5]);
        assert!(res.former.is_empty());
    }

    #[tokio::test]
    async fn former_players_are_grouped_by_latest_season_first() {
        let res = service(vec![
            player(10, Some(7), Some(Position::FW), 2024, false),
            player(11, Some(2), Some(Position::DF), 2025, false),
            player(12, Some(8), Some(Position::MF), 2025, false),
            player(13, Some(1), Some(Position::GK), 2026, true),
        ])
        .squad(77680)
        .await
        .unwrap();
        assert_eq!(
            res.active.iter().map(|p| p.player_id).collect::<Vec<_>>(),
            vec![13]
        );
        assert_eq!(
            res.former.iter().map(|p| p.player_id).collect::<Vec<_>>(),
            vec![11, 12, 10]
        );
        assert_eq!(res.team.team_name, "成都蓉城");
    }
}
