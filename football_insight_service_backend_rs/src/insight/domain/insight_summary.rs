use crate::insight::domain::overview::InsightOverview;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsightSummary {
    pub headline: String,
    pub summary: String,
    pub bullets: Vec<String>,
    pub focus_match_id: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankingMovement {
    pub name: String,
    pub current_rank: i32,
    pub previous_rank: i32,
}

impl RankingMovement {
    pub fn rise(&self) -> i32 {
        self.previous_rank - self.current_rank
    }
}

pub fn generate_insight_summary(
    overview: &InsightOverview,
    standings_movement: Option<RankingMovement>,
    scorer_movement: Option<RankingMovement>,
) -> InsightSummary {
    let focus_match = overview.recent_matches.first();
    let top_team = overview.standings_top.first();
    let top_scorer = overview.top_scorers.first();
    let chasing_scorer = overview.top_scorers.get(1);

    let headline = if let Some(match_item) = focus_match {
        format!(
            "{} {}:{} {}，联赛格局继续收紧",
            match_item.home_team_name,
            match_item.home_score,
            match_item.away_score,
            match_item.away_team_name
        )
    } else if let Some(team) = top_team {
        format!("{} 暂时领跑积分榜", team.team_name)
    } else {
        "这一轮之后，联赛格局仍在变化".to_string()
    };

    let summary = if let (Some(team), Some(scorer_a), Some(scorer_b)) =
        (top_team, top_scorer, chasing_scorer)
    {
        format!(
            "{}目前位居积分榜首位，{}与{}占据射手榜头部，联赛头部竞争仍在持续。",
            team.team_name, scorer_a.player_name, scorer_b.player_name
        )
    } else if let Some(team) = top_team {
        format!(
            "{}目前仍在积分榜头部，最近赛果正在继续改写联赛走势。",
            team.team_name
        )
    } else {
        "最近赛果、榜单头部和轮次复盘都值得继续关注。".to_string()
    };

    let mut bullets = Vec::new();

    if let Some(movement) =
        standings_movement.filter(|item| item.rise() > 0 && item.current_rank <= 8)
    {
        bullets.push(format!(
            "{} 本期上升 {} 位，来到第 {} 位。",
            movement.name,
            movement.rise(),
            movement.current_rank
        ));
    }

    if let Some(movement) =
        scorer_movement.filter(|item| item.rise() > 0 && item.current_rank <= 10)
    {
        bullets.push(format!(
            "{} 在射手榜上升 {} 位，目前排名第 {}。",
            movement.name,
            movement.rise(),
            movement.current_rank
        ));
    }

    if let Some(match_item) = focus_match {
        bullets.push(format!(
            "{} {}:{} {} 是最新焦点赛果。",
            match_item.home_team_name,
            match_item.home_score,
            match_item.away_score,
            match_item.away_team_name
        ));
    }

    if bullets.is_empty() {
        if let Some(team) = top_team {
            bullets.push(format!(
                "{} 目前以 {} 分位居积分榜首位。",
                team.team_name, team.points
            ));
        }
        if let Some(player) = top_scorer {
            bullets.push(format!(
                "{} 目前以 {} 球领跑射手榜。",
                player.player_name, player.score_value
            ));
        }
    }

    InsightSummary {
        headline,
        summary,
        bullets,
        focus_match_id: focus_match.map(|item| item.match_id),
    }
}
