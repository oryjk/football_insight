#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    GK,
    DF,
    MF,
    FW,
}

impl Position {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "GK" => Some(Self::GK),
            "DF" => Some(Self::DF),
            "MF" => Some(Self::MF),
            "FW" => Some(Self::FW),
            _ => None,
        }
    }

    /// 排序权重：门将、后卫、中场、前锋，未知排最后。
    pub fn rank(position: Option<Self>) -> u8 {
        match position {
            Some(Self::GK) => 0,
            Some(Self::DF) => 1,
            Some(Self::MF) => 2,
            Some(Self::FW) => 3,
            None => 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadTeam {
    pub team_id: i64,
    pub team_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SquadPlayer {
    pub player_id: i64,
    pub name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<Position>,
    pub avatar_url: Option<String>,
    pub last_season: i32,
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupReadError {
    #[error("球队参数无效")]
    Invalid,
    #[error("球队不存在")]
    NotFound,
    #[error("名单暂不可用")]
    Unavailable,
}

use std::collections::{HashMap, HashSet};

pub const FORMATIONS: [&str; 7] = [
    "4-3-3", "4-4-2", "4-2-3-1", "3-5-2", "3-4-3", "5-3-2", "4-1-4-1",
];

/// 共识阵容的最少公开阵容数；不足时只返回进度。
pub const CONSENSUS_MIN_LINEUPS: usize = 10;

/// 阵型的位置需求 [门将, 后卫, 中场, 前锋]：第一个数字为后卫，最后一个为前锋，中间之和为中场。
pub fn formation_needs(formation: &str) -> Option<[usize; 4]> {
    if !FORMATIONS.contains(&formation) {
        return None;
    }
    let parts: Vec<usize> = formation
        .split('-')
        .map(|part| part.parse().ok())
        .collect::<Option<Vec<usize>>>()?;
    if parts.len() < 3 {
        return None;
    }
    let mf = parts[1..parts.len() - 1].iter().sum();
    Some([1, parts[0], mf, *parts.last()?])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusPick {
    pub player_id: i64,
    /// 在共识中占用的位置类；补齐的球员可能与名单位置不同。
    pub group: Position,
    pub picks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Consensus {
    pub total: usize,
    pub formation: Option<String>,
    pub formation_picks: usize,
    pub players: Vec<ConsensusPick>,
}

pub fn percent(part: usize, total: usize) -> u8 {
    if total == 0 {
        return 0;
    }
    ((part as f64) * 100.0 / (total as f64)).round() as u8
}

/// 共识阵容：取最常用阵型，按名单位置在每类位置里取被选次数最多的球员（平票按 player_id），
/// 某类不够时用剩余最热门的球员补齐，结果恰为 11 人。不足门槛时 formation 为 None、players 为空。
pub fn compute_consensus(
    lineups: &[(String, Vec<i64>)],
    positions: &HashMap<i64, Option<Position>>,
) -> Consensus {
    let total = lineups.len();
    let locked = Consensus {
        total,
        formation: None,
        formation_picks: 0,
        players: Vec::new(),
    };
    if total < CONSENSUS_MIN_LINEUPS {
        return locked;
    }
    let mut best: Option<(&str, usize)> = None;
    for formation in FORMATIONS {
        let count = lineups
            .iter()
            .filter(|(f, _)| f.as_str() == formation)
            .count();
        if count > 0 && best.is_none_or(|(_, top)| count > top) {
            best = Some((formation, count));
        }
    }
    let Some((formation, formation_picks)) = best else {
        return locked;
    };
    let Some(needs) = formation_needs(formation) else {
        return locked;
    };
    let mut counts: HashMap<i64, usize> = HashMap::new();
    for (_, players) in lineups {
        for id in players {
            *counts.entry(*id).or_default() += 1;
        }
    }
    let mut ranked: Vec<(i64, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    const GROUPS: [Position; 4] = [Position::GK, Position::DF, Position::MF, Position::FW];
    let mut chosen: Vec<ConsensusPick> = Vec::with_capacity(11);
    let mut used: HashSet<i64> = HashSet::new();
    let mut shortage = [0usize; 4];
    for (index, group) in GROUPS.iter().enumerate() {
        let mut taken = 0;
        for (id, picks) in &ranked {
            if taken == needs[index] {
                break;
            }
            if positions.get(id).copied().flatten() == Some(*group) && used.insert(*id) {
                chosen.push(ConsensusPick {
                    player_id: *id,
                    group: *group,
                    picks: *picks,
                });
                taken += 1;
            }
        }
        shortage[index] = needs[index] - taken;
    }
    for (index, group) in GROUPS.iter().enumerate() {
        for _ in 0..shortage[index] {
            if let Some((id, picks)) = ranked.iter().find(|(id, _)| !used.contains(id)) {
                used.insert(*id);
                chosen.push(ConsensusPick {
                    player_id: *id,
                    group: *group,
                    picks: *picks,
                });
            }
        }
    }
    chosen.sort_by(|a, b| {
        Position::rank(Some(a.group))
            .cmp(&Position::rank(Some(b.group)))
            .then(b.picks.cmp(&a.picks))
            .then(a.player_id.cmp(&b.player_id))
    });
    Consensus {
        total,
        formation: Some(formation.to_string()),
        formation_picks,
        players: chosen,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupSlot {
    pub slot: u8,
    pub player_id: Option<i64>,
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lineup {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub share_code: String,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupSaveError {
    #[error("账号不可用，请重新登录")]
    Unauthorized,
    #[error("阵型无效")]
    InvalidFormation,
    #[error("站位数据无效")]
    InvalidSlots,
    #[error("发布阵容需要选满11名球员")]
    IncompleteLineup,
    #[error("球员不属于这支球队")]
    PlayerNotInSquad,
    #[error("同一球员不能重复上场")]
    DuplicatePlayer,
    #[error("球队不存在")]
    TeamNotFound,
    #[error("阵容暂不可用")]
    Unavailable,
}

/// 与球队名单无关的纯校验：阵型、11 个不重复的位置序号、坐标范围、球员不重复。
pub fn validate_lineup_input(formation: &str, slots: &[LineupSlot]) -> Result<(), LineupSaveError> {
    if !FORMATIONS.contains(&formation) {
        return Err(LineupSaveError::InvalidFormation);
    }
    if slots.len() != 11 {
        return Err(LineupSaveError::InvalidSlots);
    }
    let in_unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
    let mut seen_slots = HashSet::new();
    for slot in slots {
        if slot.slot > 10 || !seen_slots.insert(slot.slot) || !in_unit(slot.x) || !in_unit(slot.y) {
            return Err(LineupSaveError::InvalidSlots);
        }
    }
    let mut seen_players = HashSet::new();
    for id in slots.iter().filter_map(|s| s.player_id) {
        if !seen_players.insert(id) {
            return Err(LineupSaveError::DuplicatePlayer);
        }
    }
    Ok(())
}

#[cfg(test)]
mod lineup_validation_tests {
    use super::*;

    fn slots(n: u8) -> Vec<LineupSlot> {
        (0..n)
            .map(|slot| LineupSlot {
                slot,
                player_id: None,
                x: 0.5,
                y: 0.5,
            })
            .collect()
    }

    #[test]
    fn accepts_eleven_unique_slots_for_every_formation() {
        for f in FORMATIONS {
            assert_eq!(validate_lineup_input(f, &slots(11)), Ok(()), "{f}");
        }
    }

    #[test]
    fn rejects_unknown_formation() {
        assert_eq!(
            validate_lineup_input("9-9-9", &slots(11)),
            Err(LineupSaveError::InvalidFormation)
        );
    }

    #[test]
    fn rejects_wrong_slot_count_duplicate_or_out_of_range_slot_index() {
        assert_eq!(
            validate_lineup_input("4-3-3", &slots(10)),
            Err(LineupSaveError::InvalidSlots)
        );
        assert_eq!(
            validate_lineup_input("4-3-3", &slots(12)),
            Err(LineupSaveError::InvalidSlots)
        );
        let mut dup = slots(11);
        dup[3].slot = 2;
        assert_eq!(
            validate_lineup_input("4-3-3", &dup),
            Err(LineupSaveError::InvalidSlots)
        );
        let mut high = slots(11);
        high[10].slot = 11;
        assert_eq!(
            validate_lineup_input("4-3-3", &high),
            Err(LineupSaveError::InvalidSlots)
        );
    }

    #[test]
    fn rejects_coordinates_outside_unit_square_or_not_finite() {
        for (x, y) in [
            (-0.01, 0.5),
            (1.01, 0.5),
            (0.5, -0.01),
            (0.5, 1.01),
            (f64::NAN, 0.5),
            (0.5, f64::INFINITY),
        ] {
            let mut s = slots(11);
            s[0].x = x;
            s[0].y = y;
            assert_eq!(
                validate_lineup_input("4-3-3", &s),
                Err(LineupSaveError::InvalidSlots),
                "{x},{y}"
            );
        }
    }

    #[test]
    fn rejects_the_same_player_twice() {
        let mut s = slots(11);
        s[0].player_id = Some(7);
        s[5].player_id = Some(7);
        assert_eq!(
            validate_lineup_input("4-3-3", &s),
            Err(LineupSaveError::DuplicatePlayer)
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LineupPublicError {
    #[error("参数无效")]
    Invalid,
    #[error("账号不可用，请重新登录")]
    Unauthorized,
    #[error("阵容或球队不存在")]
    NotFound,
    #[error("阵容暂不可用")]
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineupCodeImage {
    pub image_base64: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineupSort {
    Latest,
    Likes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicLineupQuery {
    pub sort: LineupSort,
    pub page: u32,
    pub page_size: u32,
}

impl PublicLineupQuery {
    pub fn parse(sort: &str, page: u32, page_size: u32) -> Result<Self, LineupPublicError> {
        let sort = match sort {
            "latest" => LineupSort::Latest,
            "likes" => LineupSort::Likes,
            _ => return Err(LineupPublicError::Invalid),
        };
        if page == 0 || !(1..=50).contains(&page_size) {
            return Err(LineupPublicError::Invalid);
        }
        Ok(Self {
            sort,
            page,
            page_size,
        })
    }

    pub fn offset(self) -> i64 {
        i64::from(self.page - 1) * i64::from(self.page_size)
    }
}

pub fn valid_share_code(code: &str) -> bool {
    code.len() == 8 && code.bytes().all(|b| b.is_ascii_alphanumeric())
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupAuthor {
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineupLikeState {
    pub likes_count: i64,
    pub liked_by_me: bool,
    pub disliked_by_me: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineupReaction {
    Like,
    Dislike,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicLineupItem {
    pub share_code: String,
    pub formation: String,
    pub author: LineupAuthor,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub likes: LineupLikeState,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConsensusPlayer {
    pub player: SquadPlayer,
    pub group: Position,
    pub pick_rate: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupConsensus {
    pub team: SquadTeam,
    pub total: usize,
    pub threshold: usize,
    pub formation: Option<String>,
    pub formation_share: u8,
    pub players: Vec<ConsensusPlayer>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublicLineupPage {
    pub team: SquadTeam,
    pub items: Vec<PublicLineupItem>,
    pub total: i64,
    pub query: PublicLineupQuery,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SharedLineup {
    pub item: PublicLineupItem,
    pub team: SquadTeam,
    pub slots: Vec<LineupSlot>,
    pub players: Vec<SquadPlayer>,
}

#[cfg(test)]
mod public_validation_tests {
    use super::*;
    #[test]
    fn public_query_rejects_unknown_sort_and_invalid_paging() {
        for (sort, page, size) in [
            ("bad", 1, 20),
            ("latest", 0, 20),
            ("likes", 1, 0),
            ("likes", 1, 51),
        ] {
            assert_eq!(
                PublicLineupQuery::parse(sort, page, size),
                Err(LineupPublicError::Invalid)
            );
        }
        assert_eq!(
            PublicLineupQuery::parse("likes", 2, 50).unwrap().offset(),
            50
        );
    }
    #[test]
    fn shared_codes_are_exactly_eight_ascii_base62_characters() {
        assert!(valid_share_code("aAzZ0199"));
        for code in ["", "short", "123456789", "1234/678", "1234é78"] {
            assert!(!valid_share_code(code));
        }
    }
}

/// 私密编辑快照；没有分享码，也不参与任何公开查询。
#[derive(Debug, Clone, PartialEq)]
pub struct LineupDraft {
    pub formation: String,
    pub slots: Vec<LineupSlot>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LineupEditor {
    pub lineup: Option<Lineup>,
    pub draft: Option<LineupDraft>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishedLineup {
    pub lineup: Lineup,
    pub draft: LineupDraft,
}

pub fn validate_publish_input(
    formation: &str,
    slots: &[LineupSlot],
) -> Result<(), LineupSaveError> {
    validate_lineup_input(formation, slots)?;
    if slots.iter().any(|slot| slot.player_id.is_none()) {
        return Err(LineupSaveError::IncompleteLineup);
    }
    Ok(())
}

#[cfg(test)]
mod consensus_tests {
    use super::*;

    fn consensus_positions() -> std::collections::HashMap<i64, Option<Position>> {
        use Position::*;
        let mut map = std::collections::HashMap::new();
        for id in [1, 2] {
            map.insert(id, Some(GK));
        }
        for id in 10..=15 {
            map.insert(id, Some(DF));
        }
        for id in 20..=25 {
            map.insert(id, Some(MF));
        }
        for id in 30..=33 {
            map.insert(id, Some(FW));
        }
        map.insert(40, None);
        map
    }
    const BASE: [i64; 11] = [1, 10, 11, 12, 13, 20, 21, 22, 30, 31, 32];
    fn lineups(count: usize, formation: &str) -> Vec<(String, Vec<i64>)> {
        (0..count)
            .map(|_| (formation.to_string(), BASE.to_vec()))
            .collect()
    }

    #[test]
    fn formation_needs_split_every_formation_into_ten_outfield_players() {
        for formation in FORMATIONS {
            let needs = formation_needs(formation).unwrap();
            assert_eq!(needs[0], 1, "{formation}");
            assert_eq!(needs[1] + needs[2] + needs[3], 10, "{formation}");
        }
        assert_eq!(formation_needs("4-2-3-1"), Some([1, 4, 5, 1]));
        assert_eq!(formation_needs("3-5-2"), Some([1, 3, 5, 2]));
        assert_eq!(formation_needs("4-1-4-1"), Some([1, 4, 5, 1]));
        assert_eq!(formation_needs("9-9"), None);
    }

    #[test]
    fn consensus_unlocks_at_the_threshold() {
        let positions = consensus_positions();
        let locked = compute_consensus(&lineups(9, "4-3-3"), &positions);
        assert_eq!(
            (locked.total, locked.formation.clone(), locked.players.len()),
            (9, None, 0)
        );
        let unlocked = compute_consensus(&lineups(10, "4-3-3"), &positions);
        assert_eq!(unlocked.formation.as_deref(), Some("4-3-3"));
        assert_eq!(unlocked.formation_picks, 10);
        assert_eq!(
            unlocked
                .players
                .iter()
                .map(|p| p.player_id)
                .collect::<Vec<_>>(),
            BASE.to_vec()
        );
        assert!(unlocked.players.iter().all(|p| p.picks == 10));
    }

    #[test]
    fn formation_and_player_ties_are_broken_deterministically() {
        let positions = consensus_positions();
        let mut sample = lineups(5, "4-4-2");
        sample.extend(lineups(5, "4-3-3"));
        // 后卫 13 与 14 各被选 5 次：取 player_id 小的 13
        for (index, lineup) in sample.iter_mut().enumerate() {
            if index % 2 == 0 {
                lineup.1[4] = 14;
            }
        }
        let consensus = compute_consensus(&sample, &positions);
        assert_eq!(consensus.formation.as_deref(), Some("4-3-3")); // 平票按 FORMATIONS 顺序
        let df: Vec<i64> = consensus
            .players
            .iter()
            .filter(|p| p.group == Position::DF)
            .map(|p| p.player_id)
            .collect();
        assert_eq!(df, vec![10, 11, 12, 13]);
    }

    #[test]
    fn players_are_grouped_by_squad_position_not_by_placement() {
        let positions = consensus_positions();
        // 每套阵容都选了中场 23，而没有选前锋 32：前锋只剩 30、31 两人，需要补 1 人
        let mut sample = lineups(10, "4-3-3");
        for lineup in &mut sample {
            lineup.1[10] = 23;
        }
        let consensus = compute_consensus(&sample, &positions);
        assert_eq!(consensus.players.len(), 11);
        let mf: Vec<i64> = consensus
            .players
            .iter()
            .filter(|p| p.group == Position::MF)
            .map(|p| p.player_id)
            .collect();
        assert_eq!(mf, vec![20, 21, 22]);
        // 中场 23 虽然被选 10 次，但中场已满，只能作为补齐进入前锋位
        let fw: Vec<(i64, Position)> = consensus
            .players
            .iter()
            .filter(|p| p.group == Position::FW)
            .map(|p| (p.player_id, p.group))
            .collect();
        assert_eq!(
            fw,
            vec![(23, Position::FW), (30, Position::FW), (31, Position::FW)]
        );
    }

    #[test]
    fn shortages_are_filled_with_the_most_picked_remaining_players() {
        let positions = consensus_positions();
        // 3-5-2 需要 5 名中场，但大家只选了 3 名中场，其余是 2 名后卫 14、15 与未知位置 40
        let sample: Vec<(String, Vec<i64>)> = (0..10)
            .map(|index| {
                (
                    "3-5-2".to_string(),
                    vec![
                        1,
                        10,
                        11,
                        12,
                        14,
                        15,
                        20,
                        21,
                        22,
                        if index < 6 { 40 } else { 13 },
                        30,
                    ],
                )
            })
            .collect();
        let consensus = compute_consensus(&sample, &positions);
        let ids: std::collections::HashSet<i64> =
            consensus.players.iter().map(|p| p.player_id).collect();
        assert_eq!(consensus.players.len(), 11);
        assert_eq!(ids.len(), 11, "no duplicates");
        let groups = |g: Position| consensus.players.iter().filter(|p| p.group == g).count();
        assert_eq!(
            [
                groups(Position::GK),
                groups(Position::DF),
                groups(Position::MF),
                groups(Position::FW)
            ],
            [1, 3, 5, 2]
        );
        // 前锋只有 30 一人被选，第二个前锋位由剩余最热门的补上
        assert!(ids.contains(&30));
    }

    #[test]
    fn percentages_round_to_whole_numbers() {
        assert_eq!(percent(2, 3), 67);
        assert_eq!(percent(1, 3), 33);
        assert_eq!(percent(10, 10), 100);
        assert_eq!(percent(0, 0), 0);
    }
}
