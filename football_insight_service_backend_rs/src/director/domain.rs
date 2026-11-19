use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_TEAM_ID: i64 = 77680;
pub const FOREIGN_QUOTA: usize = 5;
pub const MAX_DOMESTIC_INS: usize = 3;
pub const MAX_DOMESTIC_OUTS: usize = 3;
pub const MAX_MANIFESTO_CHARS: usize = 40;
pub const MAX_COMMENT_CHARS: usize = 50;
pub const MAX_CUSTOM_NAME_CHARS: usize = 20;
pub const MAX_CUSTOM_LEAGUE_CHARS: usize = 16;
pub const MAX_CUSTOM_CLUB_CHARS: usize = 24;
pub const MAX_COMMENTS_RETURNED: i64 = 30;
pub const CIRCLE_UNLOCK_MIN_MEMBERS: usize = 2;

const SHARE_CODE_LEN: usize = 8;
const SHARE_CODE_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const DOMESTIC_COUNTRIES: &[&str] = &[
    "中国",
    "china",
    "chn",
    "中国香港",
    "香港",
    "hong kong",
    "中国澳门",
    "澳门",
    "macau",
    "中国台湾",
    "台湾",
    "中华台北",
    "taiwan",
    "chinese taipei",
];

/// 根据 `f_i_squad_players.country` 判定是否为外援。
/// 空值或国内/港澳台（大小写不敏感）均判定为国内球员（`false`），其余非空国籍判定为外援（`true`）。
pub fn is_foreign_country(country: Option<&str>) -> bool {
    let Some(raw) = country else {
        return false;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lowered = trimmed.to_lowercase();
    !DOMESTIC_COUNTRIES.iter().any(|item| *item == lowered)
}

pub fn generate_share_code() -> String {
    let mut rng = rand::rng();
    (0..SHARE_CODE_LEN)
        .map(|_| {
            let idx = rng.random_range(0..SHARE_CODE_ALPHABET.len());
            SHARE_CODE_ALPHABET[idx] as char
        })
        .collect()
}

pub fn normalize_share_code(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_ascii_uppercase();
    if normalized.len() == SHARE_CODE_LEN
        && normalized
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
    {
        Some(normalized)
    } else {
        None
    }
}

pub fn validate_position(raw: &str) -> Option<&'static str> {
    match raw.trim().to_ascii_uppercase().as_str() {
        "GK" => Some("GK"),
        "DF" => Some("DF"),
        "MF" => Some("MF"),
        "FW" => Some("FW"),
        _ => None,
    }
}

pub fn normalize_custom_name_key(name: &str) -> String {
    format!("custom:{}", name.trim().to_lowercase())
}

pub fn validate_manifesto(raw: &str) -> Result<String, DirectorError> {
    let trimmed = raw.trim();
    if trimmed.chars().count() > MAX_MANIFESTO_CHARS {
        return Err(DirectorError::InvalidInput(format!(
            "总监宣言最多 {MAX_MANIFESTO_CHARS} 个字"
        )));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r')
    {
        return Err(DirectorError::InvalidInput(
            "总监宣言包含不支持的字符".into(),
        ));
    }
    Ok(trimmed.to_string())
}

pub fn validate_comment_content(raw: &str) -> Result<String, DirectorError> {
    let trimmed = raw.trim();
    let len = trimmed.chars().count();
    if len == 0 {
        return Err(DirectorError::InvalidInput("锐评内容不能为空".into()));
    }
    if len > MAX_COMMENT_CHARS {
        return Err(DirectorError::InvalidInput(format!(
            "锐评内容最多 {MAX_COMMENT_CHARS} 个字"
        )));
    }
    if trimmed
        .chars()
        .any(|c| c.is_control() && c != '\n' && c != '\r')
    {
        return Err(DirectorError::InvalidInput(
            "锐评内容包含不支持的字符".into(),
        ));
    }
    Ok(trimmed.to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlayerSource {
    Squad,
    Pool,
    Custom,
}

impl PlayerSource {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "squad" => Some(Self::Squad),
            "pool" => Some(Self::Pool),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanPlayerItem {
    pub key: String,
    pub source: PlayerSource,
    pub player_id: Option<i64>,
    pub candidate_code: Option<String>,
    pub player_name: String,
    pub position: String,
    pub country: String,
    pub is_foreign: bool,
    pub league_tag: String,
    pub current_club: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorWindow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub opens_at: DateTime<Utc>,
    pub closes_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorTeam {
    pub team_id: i64,
    pub team_name: String,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SquadPlayer {
    pub player_id: i64,
    pub team_id: i64,
    pub team_name: String,
    pub player_name: String,
    pub shirt_number: Option<i32>,
    pub position: Option<String>,
    pub country: Option<String>,
    pub avatar_url: Option<String>,
}

impl SquadPlayer {
    pub fn is_foreign(&self) -> bool {
        is_foreign_country(self.country.as_deref())
    }

    pub fn to_plan_item(&self) -> PlanPlayerItem {
        let is_foreign = self.is_foreign();
        PlanPlayerItem {
            key: format!("squad:{}", self.player_id),
            source: PlayerSource::Squad,
            player_id: Some(self.player_id),
            candidate_code: None,
            player_name: self.player_name.clone(),
            position: self.position.clone().unwrap_or_else(|| "MF".into()),
            country: self
                .country
                .clone()
                .filter(|c| !c.trim().is_empty())
                .unwrap_or_else(|| if is_foreign { "海外".into() } else { "中国".into() }),
            is_foreign,
            league_tag: "CSL".into(),
            current_club: self.team_name.clone(),
            avatar_url: self.avatar_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolCandidate {
    pub code: String,
    pub player_name: String,
    pub position: String,
    pub country: String,
    pub is_foreign: bool,
    pub league_tag: String,
    pub current_club: String,
    pub tags: Vec<String>,
    pub avatar_url: Option<String>,
}

impl PoolCandidate {
    pub fn to_plan_item(&self) -> PlanPlayerItem {
        PlanPlayerItem {
            key: format!("pool:{}", self.code),
            source: PlayerSource::Pool,
            player_id: None,
            candidate_code: Some(self.code.clone()),
            player_name: self.player_name.clone(),
            position: self.position.clone(),
            country: self.country.clone(),
            is_foreign: self.is_foreign,
            league_tag: self.league_tag.clone(),
            current_club: self.current_club.clone(),
            avatar_url: self.avatar_url.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateItem {
    pub key: String,
    pub source: PlayerSource,
    pub player_id: Option<i64>,
    pub candidate_code: Option<String>,
    pub player_name: String,
    pub position: String,
    pub country: String,
    pub is_foreign: bool,
    pub league_tag: String,
    pub current_club: String,
    pub tags: Vec<String>,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorAuthor {
    pub user_id: Uuid,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorPlan {
    pub id: Uuid,
    pub window_id: i64,
    pub window_code: String,
    pub window_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub share_code: String,
    pub circle_code: String,
    pub parent_share_code: Option<String>,
    pub author: DirectorAuthor,
    pub is_mine: bool,
    pub foreign_stays: Vec<PlanPlayerItem>,
    pub foreign_leaves: Vec<PlanPlayerItem>,
    pub foreign_ins: Vec<PlanPlayerItem>,
    pub domestic_ins: Vec<PlanPlayerItem>,
    pub domestic_outs: Vec<PlanPlayerItem>,
    pub style_tag: String,
    pub manifesto: String,
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub my_reaction: i16,
    pub comments_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorComment {
    pub id: Uuid,
    pub author: DirectorAuthor,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReactionSummary {
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub my_reaction: i16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveDisagreement {
    pub player: PlanPlayerItem,
    pub target_action: String,
    pub my_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanComparison {
    pub my_share_code: String,
    pub overlap_percent: u8,
    pub common_foreign: Vec<PlanPlayerItem>,
    pub only_in_target: Vec<PlanPlayerItem>,
    pub only_in_mine: Vec<PlanPlayerItem>,
    pub leave_disagreements: Vec<LeaveDisagreement>,
    pub summary_line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorPlanDetail {
    pub plan: DirectorPlan,
    pub comments: Vec<DirectorComment>,
    pub comparison: Option<PlanComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectorContext {
    pub window: DirectorWindow,
    pub team: DirectorTeam,
    pub foreign_quota: usize,
    pub max_domestic_ins: usize,
    pub max_domestic_outs: usize,
    pub squad_foreign: Vec<SquadPlayer>,
    pub squad_domestic: Vec<SquadPlayer>,
    pub my_plan: Option<DirectorPlan>,
    pub team_plan_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusForeignItem {
    pub player: PlanPlayerItem,
    pub pick_count: usize,
    pub pick_rate: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsensusLeaveItem {
    pub player: PlanPlayerItem,
    pub leave_count: usize,
    pub leave_rate: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CirclePlanSummary {
    pub share_code: String,
    pub author: DirectorAuthor,
    pub style_tag: String,
    pub manifesto: String,
    pub foreign_count: usize,
    pub likes_count: i64,
    pub dislikes_count: i64,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CircleConsensus {
    pub circle_code: String,
    pub window_name: String,
    pub team_id: i64,
    pub team_name: String,
    pub member_count: usize,
    pub unlocked: bool,
    pub top_foreign: Vec<ConsensusForeignItem>,
    pub top_leaves: Vec<ConsensusLeaveItem>,
    pub plans: Vec<CirclePlanSummary>,
}

/// 按设计文档 1.3 优先级自动计算风格标签。
pub fn compute_style_tag(
    foreign_stays: &[PlanPlayerItem],
    foreign_leaves: &[PlanPlayerItem],
    foreign_ins: &[PlanPlayerItem],
    domestic_ins: &[PlanPlayerItem],
) -> &'static str {
    let has_k_league_in = foreign_ins
        .iter()
        .any(|p| p.league_tag.to_ascii_uppercase().contains('K'));

    if has_k_league_in {
        if foreign_leaves.len() >= 3 && foreign_ins.len() >= 3 {
            return "推倒重建派";
        }
        return "K联赛淘宝流";
    }

    if foreign_leaves.len() >= 3 {
        return "推倒重建派";
    }

    if foreign_stays.len() >= 4 && foreign_ins.len() <= 1 {
        return "稳健微调派";
    }

    let total_ins = foreign_ins.len() + domestic_ins.len();
    if total_ins >= 2 {
        let fw_count = foreign_ins
            .iter()
            .chain(domestic_ins.iter())
            .filter(|p| p.position.eq_ignore_ascii_case("FW"))
            .count();
        if fw_count * 2 >= total_ins {
            return "锋线狂魔";
        }

        let def_count = foreign_ins
            .iter()
            .chain(domestic_ins.iter())
            .filter(|p| {
                p.position.eq_ignore_ascii_case("DF") || p.position.eq_ignore_ascii_case("GK")
            })
            .count();
        if def_count * 2 >= total_ins {
            return "铁血防线派";
        }
    }

    "务实均衡派"
}

/// 1v1 分歧报告纯函数：对比目标方案与当前用户方案。
pub fn compute_plan_comparison(target: &DirectorPlan, mine: &DirectorPlan) -> PlanComparison {
    let target_roster: Vec<&PlanPlayerItem> = target
        .foreign_stays
        .iter()
        .chain(target.foreign_ins.iter())
        .collect();
    let mine_roster: Vec<&PlanPlayerItem> = mine
        .foreign_stays
        .iter()
        .chain(mine.foreign_ins.iter())
        .collect();

    let mine_keys: HashSet<&str> = mine_roster.iter().map(|p| p.key.as_str()).collect();
    let target_keys: HashSet<&str> = target_roster.iter().map(|p| p.key.as_str()).collect();

    let mut common_foreign = Vec::new();
    let mut only_in_target = Vec::new();
    for item in &target_roster {
        if mine_keys.contains(item.key.as_str()) {
            common_foreign.push((*item).clone());
        } else {
            only_in_target.push((*item).clone());
        }
    }

    let mut only_in_mine = Vec::new();
    for item in &mine_roster {
        if !target_keys.contains(item.key.as_str()) {
            only_in_mine.push((*item).clone());
        }
    }

    let denom = target_roster.len().max(mine_roster.len()).max(1);
    let overlap_percent = ((common_foreign.len() * 100 + denom / 2) / denom).min(100) as u8;

    let mut target_actions: HashMap<&str, (&PlanPlayerItem, &'static str)> = HashMap::new();
    for item in &target.foreign_stays {
        target_actions.insert(item.key.as_str(), (item, "stay"));
    }
    for item in &target.foreign_leaves {
        target_actions.insert(item.key.as_str(), (item, "leave"));
    }

    let mut mine_actions: HashMap<&str, (&PlanPlayerItem, &'static str)> = HashMap::new();
    for item in &mine.foreign_stays {
        mine_actions.insert(item.key.as_str(), (item, "stay"));
    }
    for item in &mine.foreign_leaves {
        mine_actions.insert(item.key.as_str(), (item, "leave"));
    }

    let mut leave_disagreements = Vec::new();
    let mut seen_disagreements = HashSet::new();
    // 优先按目标方案离队 -> 留队顺序遍历，确保结果稳定
    for item in target
        .foreign_leaves
        .iter()
        .chain(target.foreign_stays.iter())
    {
        if !seen_disagreements.insert(item.key.as_str()) {
            continue;
        }
        if let (Some((_, target_action)), Some((_, my_action))) = (
            target_actions.get(item.key.as_str()),
            mine_actions.get(item.key.as_str()),
        ) && target_action != my_action
        {
            leave_disagreements.push(LeaveDisagreement {
                player: item.clone(),
                target_action: (*target_action).to_string(),
                my_action: (*my_action).to_string(),
            });
        }
    }

    let summary_line = if common_foreign.len() == target_roster.len()
        && target_roster.len() == mine_roster.len()
        && leave_disagreements.is_empty()
    {
        format!(
            "你们在全部 {} 个外援名额上完全一致，建队思路高度默契",
            common_foreign.len()
        )
    } else if let Some(first_diff) = leave_disagreements.first() {
        format!(
            "你们在 {} 个外援名额上达成一致，最大分歧在{}的去留",
            common_foreign.len(),
            first_diff.player.player_name
        )
    } else if common_foreign.is_empty() {
        "你们的外援人选完全不同，建队思路针锋相对".to_string()
    } else {
        format!(
            "你们在 {} 个外援名额上达成一致，其余引援方向各有侧重",
            common_foreign.len()
        )
    };

    PlanComparison {
        my_share_code: mine.share_code.clone(),
        overlap_percent,
        common_foreign,
        only_in_target,
        only_in_mine,
        leave_disagreements,
        summary_line,
    }
}

/// 好友圈共识聚合纯函数：满 2 人解锁 Top 5 外援与最想送走球员离队率。
pub fn compute_circle_consensus(
    circle_code: String,
    window_name: String,
    team_id: i64,
    team_name: String,
    plans: &[DirectorPlan],
) -> CircleConsensus {
    let member_count = plans.len();
    let unlocked = member_count >= CIRCLE_UNLOCK_MIN_MEMBERS;

    let (top_foreign, top_leaves) = if unlocked {
        let mut foreign_counts: BTreeMap<String, (PlanPlayerItem, usize)> = BTreeMap::new();
        let mut leave_counts: BTreeMap<String, (PlanPlayerItem, usize)> = BTreeMap::new();

        for plan in plans {
            let mut seen_foreign = HashSet::new();
            for item in plan.foreign_stays.iter().chain(plan.foreign_ins.iter()) {
                if seen_foreign.insert(item.key.as_str()) {
                    foreign_counts
                        .entry(item.key.clone())
                        .and_modify(|(_, count)| *count += 1)
                        .or_insert_with(|| (item.clone(), 1));
                }
            }

            let mut seen_leaves = HashSet::new();
            for item in plan.foreign_leaves.iter().chain(plan.domestic_outs.iter()) {
                if seen_leaves.insert(item.key.as_str()) {
                    leave_counts
                        .entry(item.key.clone())
                        .and_modify(|(_, count)| *count += 1)
                        .or_insert_with(|| (item.clone(), 1));
                }
            }
        }

        let mut foreign_list: Vec<ConsensusForeignItem> = foreign_counts
            .into_values()
            .map(|(player, pick_count)| {
                let pick_rate =
                    ((pick_count * 100 + member_count / 2) / member_count).min(100) as u8;
                ConsensusForeignItem {
                    player,
                    pick_count,
                    pick_rate,
                }
            })
            .collect();
        foreign_list.sort_by(|a, b| {
            b.pick_count
                .cmp(&a.pick_count)
                .then_with(|| a.player.player_name.cmp(&b.player.player_name))
                .then_with(|| a.player.key.cmp(&b.player.key))
        });
        foreign_list.truncate(5);

        let mut leave_list: Vec<ConsensusLeaveItem> = leave_counts
            .into_values()
            .map(|(player, leave_count)| {
                let leave_rate =
                    ((leave_count * 100 + member_count / 2) / member_count).min(100) as u8;
                ConsensusLeaveItem {
                    player,
                    leave_count,
                    leave_rate,
                }
            })
            .collect();
        leave_list.sort_by(|a, b| {
            b.leave_count
                .cmp(&a.leave_count)
                .then_with(|| a.player.player_name.cmp(&b.player.player_name))
                .then_with(|| a.player.key.cmp(&b.player.key))
        });
        leave_list.truncate(5);

        (foreign_list, leave_list)
    } else {
        (Vec::new(), Vec::new())
    };

    let summaries = plans
        .iter()
        .map(|plan| CirclePlanSummary {
            share_code: plan.share_code.clone(),
            author: plan.author.clone(),
            style_tag: plan.style_tag.clone(),
            manifesto: plan.manifesto.clone(),
            foreign_count: plan.foreign_stays.len() + plan.foreign_ins.len(),
            likes_count: plan.likes_count,
            dislikes_count: plan.dislikes_count,
            updated_at: plan.updated_at,
        })
        .collect();

    CircleConsensus {
        circle_code,
        window_name,
        team_id,
        team_name,
        member_count,
        unlocked,
        top_foreign,
        top_leaves,
        plans: summaries,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DirectorError {
    #[error("{0}")]
    InvalidInput(String),
    #[error("请登录后继续")]
    Unauthorized,
    #[error("{0}")]
    NotFound(String),
    #[error("内容包含违规信息，请修改后重试")]
    ContentRejected,
    #[error("内容安全检测服务暂不可用，请稍后重试")]
    ModerationUnavailable,
    /// 内部：分享码冲突，由 application 重试，不直接暴露给前端
    #[error("分享码冲突")]
    ShareCodeTaken,
    #[error("服务暂时不可用，请稍后重试")]
    Unavailable,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn foreign_item(key: &str, name: &str, position: &str, league_tag: &str) -> PlanPlayerItem {
        PlanPlayerItem {
            key: key.to_string(),
            source: PlayerSource::Pool,
            player_id: None,
            candidate_code: Some(key.to_string()),
            player_name: name.to_string(),
            position: position.to_string(),
            country: "巴西".to_string(),
            is_foreign: true,
            league_tag: league_tag.to_string(),
            current_club: "测试队".to_string(),
            avatar_url: None,
        }
    }

    fn domestic_item(key: &str, name: &str, position: &str) -> PlanPlayerItem {
        PlanPlayerItem {
            key: key.to_string(),
            source: PlayerSource::Squad,
            player_id: None,
            candidate_code: None,
            player_name: name.to_string(),
            position: position.to_string(),
            country: "中国".to_string(),
            is_foreign: false,
            league_tag: "CSL".to_string(),
            current_club: "测试队".to_string(),
            avatar_url: None,
        }
    }

    fn sample_plan(
        share_code: &str,
        stays: Vec<PlanPlayerItem>,
        leaves: Vec<PlanPlayerItem>,
        ins: Vec<PlanPlayerItem>,
    ) -> DirectorPlan {
        DirectorPlan {
            id: Uuid::new_v4(),
            window_id: 1,
            window_code: "2026-winter".into(),
            window_name: "2026 冬窗".into(),
            team_id: DEFAULT_TEAM_ID,
            team_name: "成都蓉城".into(),
            share_code: share_code.into(),
            circle_code: "CIRCLE01".into(),
            parent_share_code: None,
            author: DirectorAuthor {
                user_id: Uuid::new_v4(),
                nickname: format!("用户{share_code}"),
                avatar_url: None,
            },
            is_mine: false,
            foreign_stays: stays,
            foreign_leaves: leaves,
            foreign_ins: ins,
            domestic_ins: vec![],
            domestic_outs: vec![],
            style_tag: "务实均衡派".into(),
            manifesto: String::new(),
            likes_count: 0,
            dislikes_count: 0,
            my_reaction: 0,
            comments_count: 0,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn is_foreign_country_classifies_domestic_hk_mo_tw_and_foreign() {
        for domestic in [
            None,
            Some(""),
            Some("   "),
            Some("中国"),
            Some("China"),
            Some("china"),
            Some("CHN"),
            Some("chn"),
            Some("中国香港"),
            Some("香港"),
            Some("Hong Kong"),
            Some("hong kong"),
            Some("中国澳门"),
            Some("澳门"),
            Some("Macau"),
            Some("中国台湾"),
            Some("台湾"),
            Some("中华台北"),
            Some("Taiwan"),
            Some("Chinese Taipei"),
            Some("  中国  "),
        ] {
            assert!(
                !is_foreign_country(domestic),
                "expected domestic for {domestic:?}"
            );
        }

        for foreign in [
            Some("巴西"),
            Some("Brazil"),
            Some("韩国"),
            Some("South Korea"),
            Some("荷兰"),
            Some("克罗地亚"),
        ] {
            assert!(
                is_foreign_country(foreign),
                "expected foreign for {foreign:?}"
            );
        }
    }

    #[test]
    fn style_tag_follows_spec_priority() {
        let k1 = foreign_item("pool:k1", "塞西尼亚", "FW", "K1");
        let k2 = foreign_item("pool:k2", "金某某", "MF", "K2");
        let bra = foreign_item("pool:bra", "罗慕洛", "MF", "BRA");
        let fw1 = foreign_item("pool:fw1", "前锋1", "FW", "J1");
        let fw2 = foreign_item("pool:fw2", "前锋2", "FW", "J1");
        let df1 = foreign_item("pool:df1", "后卫1", "DF", "J1");
        let gk1 = domestic_item("pool:gk1", "门将1", "GK");
        let stay = foreign_item("squad:1", "费利佩", "FW", "CSL");

        // 1. K 联赛淘宝流（未同时满足离队>=3 且 新签>=3）
        assert_eq!(
            compute_style_tag(&[stay.clone()], &[stay.clone()], &[k1.clone()], &[]),
            "K联赛淘宝流"
        );
        // 1 特殊情况：有 K 联赛新签，但同时离队>=3 且新签>=3 -> 推倒重建派
        assert_eq!(
            compute_style_tag(
                &[],
                &[stay.clone(), stay.clone(), stay.clone()],
                &[k1.clone(), k2.clone(), bra.clone()],
                &[]
            ),
            "推倒重建派"
        );
        // 2. 离队>=3 -> 推倒重建派
        assert_eq!(
            compute_style_tag(
                &[stay.clone()],
                &[stay.clone(), stay.clone(), stay.clone()],
                &[bra.clone()],
                &[]
            ),
            "推倒重建派"
        );
        // 3. 留队>=4 且新签<=1 -> 稳健微调派
        assert_eq!(
            compute_style_tag(
                &[stay.clone(), stay.clone(), stay.clone(), stay.clone()],
                &[],
                &[bra.clone()],
                &[]
            ),
            "稳健微调派"
        );
        // 4. 新签前锋占比>=50% 且总数>=2 -> 锋线狂魔
        assert_eq!(
            compute_style_tag(&[stay.clone()], &[], &[fw1.clone(), bra.clone()], &[]),
            "锋线狂魔"
        );
        // 5. 新签后卫/门将占比>=50% 且总数>=2 -> 铁血防线派
        assert_eq!(
            compute_style_tag(&[stay.clone()], &[], &[df1.clone()], &[gk1.clone()]),
            "铁血防线派"
        );
        // 6. 兜底 -> 务实均衡派
        assert_eq!(
            compute_style_tag(&[stay.clone(), stay.clone()], &[], &[bra.clone()], &[]),
            "务实均衡派"
        );
        // 前锋优先于防线（50% FW + 50% DF）
        assert_eq!(
            compute_style_tag(&[], &[], &[fw2, df1], &[]),
            "锋线狂魔"
        );
    }

    #[test]
    fn plan_comparison_computes_overlap_and_leave_disagreements() {
        let p1 = foreign_item("squad:1", "费利佩", "FW", "CSL");
        let p2 = foreign_item("squad:2", "罗慕洛", "MF", "CSL");
        let p3 = foreign_item("squad:3", "提莫", "DF", "CSL");
        let p4 = foreign_item("pool:k1", "塞西尼亚", "FW", "K1");
        let p5 = foreign_item("pool:j1", "田中", "MF", "J1");
        let p6 = foreign_item("pool:bra", "卡洛斯", "FW", "BRA");
        let p7 = foreign_item("pool:eur", "米勒", "DF", "EUR");

        // target: stays=[p2, p3], leaves=[p1], ins=[p4, p5, p6] -> 5 foreign
        let target = sample_plan(
            "TARGET01",
            vec![p2.clone(), p3.clone()],
            vec![p1.clone()],
            vec![p4.clone(), p5.clone(), p6.clone()],
        );
        // mine: stays=[p1, p2, p3], leaves=[], ins=[p4, p7] -> 5 foreign
        // common: p2, p3, p4 (3/5 = 60%)
        let mine = sample_plan(
            "MINE0001",
            vec![p1.clone(), p2.clone(), p3.clone()],
            vec![],
            vec![p4.clone(), p7.clone()],
        );

        let cmp = compute_plan_comparison(&target, &mine);
        assert_eq!(cmp.my_share_code, "MINE0001");
        assert_eq!(cmp.overlap_percent, 60);
        assert_eq!(cmp.common_foreign.len(), 3);
        assert_eq!(cmp.only_in_target.len(), 2);
        assert_eq!(cmp.only_in_mine.len(), 2);
        assert_eq!(cmp.leave_disagreements.len(), 1);
        assert_eq!(cmp.leave_disagreements[0].player.player_name, "费利佩");
        assert_eq!(cmp.leave_disagreements[0].target_action, "leave");
        assert_eq!(cmp.leave_disagreements[0].my_action, "stay");
        assert_eq!(
            cmp.summary_line,
            "你们在 3 个外援名额上达成一致，最大分歧在费利佩的去留"
        );
    }

    #[test]
    fn circle_consensus_unlocks_at_two_members_and_ranks_top_picks_and_leaves() {
        let p1 = foreign_item("squad:1", "费利佩", "FW", "CSL");
        let p2 = foreign_item("squad:2", "罗慕洛", "MF", "CSL");
        let p3 = foreign_item("pool:k1", "塞西尼亚", "FW", "K1");

        let plan1 = sample_plan(
            "PLAN0001",
            vec![p2.clone()],
            vec![p1.clone()],
            vec![p3.clone()],
        );
        // 1 人未解锁
        let locked = compute_circle_consensus(
            "CIRCLE01".into(),
            "2026 冬窗".into(),
            DEFAULT_TEAM_ID,
            "成都蓉城".into(),
            std::slice::from_ref(&plan1),
        );
        assert_eq!(locked.member_count, 1);
        assert!(!locked.unlocked);
        assert!(locked.top_foreign.is_empty());
        assert!(locked.top_leaves.is_empty());

        // 2 人解锁
        let plan2 = sample_plan("PLAN0002", vec![p1.clone(), p2.clone()], vec![], vec![]);
        let unlocked = compute_circle_consensus(
            "CIRCLE01".into(),
            "2026 冬窗".into(),
            DEFAULT_TEAM_ID,
            "成都蓉城".into(),
            &[plan1, plan2],
        );
        assert_eq!(unlocked.member_count, 2);
        assert!(unlocked.unlocked);
        assert_eq!(unlocked.top_foreign[0].player.player_name, "罗慕洛");
        assert_eq!(unlocked.top_foreign[0].pick_count, 2);
        assert_eq!(unlocked.top_foreign[0].pick_rate, 100);
        assert_eq!(unlocked.top_leaves.len(), 1);
        assert_eq!(unlocked.top_leaves[0].player.player_name, "费利佩");
        assert_eq!(unlocked.top_leaves[0].leave_count, 1);
        assert_eq!(unlocked.top_leaves[0].leave_rate, 50);
    }

    #[test]
    fn share_code_generation_and_normalization_work() {
        for _ in 0..32 {
            let code = generate_share_code();
            assert_eq!(code.len(), 8);
            assert_eq!(normalize_share_code(&code), Some(code));
        }
        assert_eq!(
            normalize_share_code("  ab12cd34 "),
            Some("AB12CD34".to_string())
        );
        assert_eq!(normalize_share_code("short"), None);
        assert_eq!(normalize_share_code("AB12-D34"), None);
    }
}
