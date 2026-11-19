use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::content_security::{ContentSecurityError, ContentSecurityPort, ModerationScene};

use super::{
    domain::{
        CandidateItem, CircleConsensus, DEFAULT_TEAM_ID, DirectorComment, DirectorContext,
        DirectorError, DirectorPlanDetail, FOREIGN_QUOTA, MAX_COMMENTS_RETURNED,
        MAX_CUSTOM_CLUB_CHARS, MAX_CUSTOM_LEAGUE_CHARS, MAX_CUSTOM_NAME_CHARS, MAX_DOMESTIC_INS,
        MAX_DOMESTIC_OUTS, PlanPlayerItem, PlayerSource, ReactionSummary, SquadPlayer,
        compute_circle_consensus, compute_plan_comparison, compute_style_tag, generate_share_code,
        normalize_custom_name_key, normalize_share_code, validate_comment_content,
        validate_manifesto, validate_position,
    },
    ports::{DirectorRepository, UpsertPlanRecord},
};

const SHARE_CODE_RETRIES: usize = 5;

pub type Clock = Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct IncomingPlayerInput {
    pub source: String,
    pub candidate_code: Option<String>,
    pub player_id: Option<i64>,
    pub player_name: Option<String>,
    pub position: Option<String>,
    pub league_tag: Option<String>,
    pub current_club: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SavePlanInput {
    pub team_id: i64,
    pub circle_code: Option<String>,
    pub parent_share_code: Option<String>,
    pub foreign_stay_ids: Vec<i64>,
    pub foreign_leave_ids: Vec<i64>,
    pub foreign_ins: Vec<IncomingPlayerInput>,
    pub domestic_ins: Vec<IncomingPlayerInput>,
    pub domestic_outs: Vec<i64>,
    pub manifesto: String,
}

pub struct DirectorService {
    repository: Arc<dyn DirectorRepository>,
    content_security: Arc<dyn ContentSecurityPort>,
    clock: Clock,
}

impl DirectorService {
    pub fn new(
        repository: Arc<dyn DirectorRepository>,
        content_security: Arc<dyn ContentSecurityPort>,
        clock: Clock,
    ) -> Self {
        Self {
            repository,
            content_security,
            clock,
        }
    }

    pub async fn get_context(
        &self,
        team_id: Option<i64>,
        viewer: Option<Uuid>,
    ) -> Result<DirectorContext, DirectorError> {
        let resolved_team_id = match team_id {
            Some(id) if id > 0 => id,
            Some(_) => return Err(DirectorError::InvalidInput("team_id 无效".into())),
            None => {
                if let Some(uid) = viewer {
                    self.repository
                        .user_profile(uid)
                        .await?
                        .filter(|p| p.active)
                        .and_then(|p| p.favorite_team_id)
                        .unwrap_or(DEFAULT_TEAM_ID)
                } else {
                    DEFAULT_TEAM_ID
                }
            }
        };

        let window = self
            .repository
            .current_window((self.clock)())
            .await?
            .ok_or_else(|| DirectorError::NotFound("暂无可用转会窗口".into()))?;

        let team = self
            .repository
            .find_team(resolved_team_id)
            .await?
            .ok_or_else(|| DirectorError::NotFound("球队不存在".into()))?;

        let squad = self.repository.team_active_squad(team.team_id).await?;
        let mut squad_foreign = Vec::new();
        let mut squad_domestic = Vec::new();
        for player in squad {
            if player.is_foreign() {
                squad_foreign.push(player);
            } else {
                squad_domestic.push(player);
            }
        }

        let my_plan = if let Some(uid) = viewer {
            self.repository
                .find_plan_by_user(window.id, team.team_id, uid, Some(uid))
                .await?
        } else {
            None
        };

        let team_plan_count = self
            .repository
            .count_team_plans(window.id, team.team_id)
            .await?;

        Ok(DirectorContext {
            window,
            team,
            foreign_quota: FOREIGN_QUOTA,
            max_domestic_ins: MAX_DOMESTIC_INS,
            max_domestic_outs: MAX_DOMESTIC_OUTS,
            squad_foreign,
            squad_domestic,
            my_plan,
            team_plan_count,
        })
    }

    pub async fn list_candidates(
        &self,
        team_id: Option<i64>,
        is_foreign: bool,
        league: Option<&str>,
        keyword: Option<&str>,
    ) -> Result<Vec<CandidateItem>, DirectorError> {
        let resolved_team_id = match team_id {
            Some(id) if id > 0 => id,
            Some(_) => return Err(DirectorError::InvalidInput("team_id 无效".into())),
            None => DEFAULT_TEAM_ID,
        };

        let normalized_league = league
            .map(|s| s.trim().to_ascii_uppercase())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "ALL".into());

        if !matches!(
            normalized_league.as_str(),
            "ALL"
                | "K"
                | "K1"
                | "K2"
                | "J1"
                | "EUR"
                | "ACL"
                | "CL1"
                | "SPL"
                | "MLS"
                | "BRA"
                | "CSL"
                | "FREE"
                | "OTHER"
        ) {
            return Err(DirectorError::InvalidInput("联赛筛选参数无效".into()));
        }

        let trimmed_kw = keyword.map(str::trim).filter(|s| !s.is_empty());

        let pool_rows = self
            .repository
            .list_pool_candidates(
                resolved_team_id,
                is_foreign,
                &normalized_league,
                trimmed_kw,
                100,
            )
            .await?;

        let mut items = Vec::with_capacity(pool_rows.len());
        let mut seen_keys = HashSet::new();

        for row in pool_rows {
            let key = format!("pool:{}", row.code);
            if seen_keys.insert(key.clone()) {
                items.push(CandidateItem {
                    key,
                    source: PlayerSource::Pool,
                    player_id: None,
                    candidate_code: Some(row.code),
                    player_name: row.player_name,
                    position: row.position,
                    country: row.country,
                    is_foreign: row.is_foreign,
                    league_tag: row.league_tag,
                    current_club: row.current_club,
                    tags: row.tags,
                    avatar_url: row.avatar_url,
                });
            }
        }

        if trimmed_kw.is_some() || normalized_league == "CSL" {
            let squad_rows = self
                .repository
                .other_teams_active_squad(resolved_team_id, trimmed_kw, 80)
                .await?;
            let mut added_squad = 0usize;
            for player in squad_rows {
                if player.is_foreign() != is_foreign {
                    continue;
                }
                let plan_item = player.to_plan_item();
                if seen_keys.insert(plan_item.key.clone()) {
                    items.push(CandidateItem {
                        key: plan_item.key,
                        source: PlayerSource::Squad,
                        player_id: plan_item.player_id,
                        candidate_code: None,
                        player_name: plan_item.player_name,
                        position: plan_item.position,
                        country: plan_item.country,
                        is_foreign: plan_item.is_foreign,
                        league_tag: plan_item.league_tag,
                        current_club: plan_item.current_club,
                        tags: vec!["中超现役".into()],
                        avatar_url: plan_item.avatar_url,
                    });
                    added_squad += 1;
                    if added_squad >= 40 {
                        break;
                    }
                }
            }
        }

        Ok(items)
    }

    pub async fn save_my_plan(
        &self,
        user_id: Uuid,
        input: SavePlanInput,
    ) -> Result<DirectorPlanDetail, DirectorError> {
        if input.team_id <= 0 {
            return Err(DirectorError::InvalidInput("team_id 无效".into()));
        }

        let profile = self
            .repository
            .user_profile(user_id)
            .await?
            .filter(|p| p.active)
            .ok_or(DirectorError::Unauthorized)?;

        let window = self
            .repository
            .current_window((self.clock)())
            .await?
            .ok_or_else(|| DirectorError::NotFound("暂无可用转会窗口".into()))?;

        let _team = self
            .repository
            .find_team(input.team_id)
            .await?
            .ok_or_else(|| DirectorError::NotFound("球队不存在".into()))?;

        let squad = self.repository.team_active_squad(input.team_id).await?;
        let mut squad_foreign_map: HashMap<i64, SquadPlayer> = HashMap::new();
        let mut squad_domestic_map: HashMap<i64, SquadPlayer> = HashMap::new();
        for player in squad {
            if player.is_foreign() {
                squad_foreign_map.insert(player.player_id, player);
            } else {
                squad_domestic_map.insert(player.player_id, player);
            }
        }

        // 1. 校验现役外援留/走（去重、互斥、属于本队现役外援）
        let mut stay_ids_seen = HashSet::new();
        let mut foreign_stays = Vec::with_capacity(input.foreign_stay_ids.len());
        for id in &input.foreign_stay_ids {
            if !stay_ids_seen.insert(*id) {
                return Err(DirectorError::InvalidInput("留队外援列表存在重复球员".into()));
            }
            let player = squad_foreign_map
                .get(id)
                .ok_or_else(|| DirectorError::InvalidInput("留队外援必须属于本队现役外援".into()))?;
            foreign_stays.push(player.to_plan_item());
        }

        let mut leave_ids_seen = HashSet::new();
        let mut foreign_leaves = Vec::with_capacity(input.foreign_leave_ids.len());
        for id in &input.foreign_leave_ids {
            if !leave_ids_seen.insert(*id) {
                return Err(DirectorError::InvalidInput("离队外援列表存在重复球员".into()));
            }
            if stay_ids_seen.contains(id) {
                return Err(DirectorError::InvalidInput("同一外援不能同时选择留队与离队".into()));
            }
            let player = squad_foreign_map
                .get(id)
                .ok_or_else(|| DirectorError::InvalidInput("离队外援必须属于本队现役外援".into()))?;
            foreign_leaves.push(player.to_plan_item());
        }

        // 2. 校验 5 外援名额上限（stays + ins <= 5 且 >= 1）
        let total_foreign = foreign_stays.len() + input.foreign_ins.len();
        if total_foreign == 0 {
            return Err(DirectorError::InvalidInput(
                "至少需要保留或新签 1 名外援才能发布方案".into(),
            ));
        }
        if total_foreign > FOREIGN_QUOTA {
            return Err(DirectorError::InvalidInput(format!(
                "外援名额不能超过 {FOREIGN_QUOTA} 人（当前留队 {} + 新签 {}）",
                foreign_stays.len(),
                input.foreign_ins.len()
            )));
        }

        // 3. 校验国内补强 / 国内离队数量上限
        if input.domestic_ins.len() > MAX_DOMESTIC_INS {
            return Err(DirectorError::InvalidInput(format!(
                "国内引援最多选择 {MAX_DOMESTIC_INS} 人"
            )));
        }
        if input.domestic_outs.len() > MAX_DOMESTIC_OUTS {
            return Err(DirectorError::InvalidInput(format!(
                "国内离队建议最多选择 {MAX_DOMESTIC_OUTS} 人"
            )));
        }

        let mut domestic_out_ids_seen = HashSet::new();
        let mut domestic_outs = Vec::with_capacity(input.domestic_outs.len());
        for id in &input.domestic_outs {
            if !domestic_out_ids_seen.insert(*id) {
                return Err(DirectorError::InvalidInput("国内离队列表存在重复球员".into()));
            }
            let player = squad_domestic_map.get(id).ok_or_else(|| {
                DirectorError::InvalidInput("国内离队建议必须属于本队现役国内球员".into())
            })?;
            domestic_outs.push(player.to_plan_item());
        }

        // 4. 解析外援补强与国内补强，并收集自由提名文本用于内容安全检测
        let mut moderated_segments: Vec<String> = Vec::new();

        let mut foreign_keys: HashSet<String> = foreign_stays
            .iter()
            .chain(foreign_leaves.iter())
            .map(|p| p.key.clone())
            .collect();
        let mut foreign_ins = Vec::with_capacity(input.foreign_ins.len());
        for incoming in &input.foreign_ins {
            let item = self
                .resolve_incoming_player(
                    incoming,
                    input.team_id,
                    true,
                    &mut moderated_segments,
                )
                .await?;
            if !foreign_keys.insert(item.key.clone()) {
                return Err(DirectorError::InvalidInput(format!(
                    "外援 {} 重复选择",
                    item.player_name
                )));
            }
            foreign_ins.push(item);
        }

        let mut domestic_keys: HashSet<String> = domestic_outs.iter().map(|p| p.key.clone()).collect();
        let mut domestic_ins = Vec::with_capacity(input.domestic_ins.len());
        for incoming in &input.domestic_ins {
            let item = self
                .resolve_incoming_player(
                    incoming,
                    input.team_id,
                    false,
                    &mut moderated_segments,
                )
                .await?;
            if !domestic_keys.insert(item.key.clone()) {
                return Err(DirectorError::InvalidInput(format!(
                    "国内球员 {} 重复选择",
                    item.player_name
                )));
            }
            domestic_ins.push(item);
        }

        // 5. 校验宣言长度并执行内容安全检测
        let manifesto = validate_manifesto(&input.manifesto)?;
        if !manifesto.is_empty() {
            moderated_segments.push(manifesto.clone());
        }
        if !moderated_segments.is_empty() {
            let combined_text = moderated_segments.join("\n");
            self.check_text_moderation(profile.wx_open_id.as_deref(), &combined_text)
                .await?;
        }

        // 6. 自动计算风格标签
        let style_tag =
            compute_style_tag(&foreign_stays, &foreign_leaves, &foreign_ins, &domestic_ins)
                .to_string();

        // 7. 保存方案（已有则沿用原 share_code / circle_code；新建则按需继承 circle_code 并在碰撞时重试）
        let saved_share_code = if let Some(existing) = self
            .repository
            .find_user_plan_codes(window.id, input.team_id, user_id)
            .await?
        {
            let record = UpsertPlanRecord {
                window_id: window.id,
                team_id: input.team_id,
                user_id,
                share_code: existing.share_code,
                circle_code: existing.circle_code,
                parent_share_code: existing.parent_share_code,
                foreign_stays,
                foreign_leaves,
                foreign_ins,
                domestic_ins,
                domestic_outs,
                style_tag,
                manifesto,
            };
            self.repository.upsert_plan(&record).await?
        } else {
            let parent_share_code = input
                .parent_share_code
                .as_deref()
                .and_then(normalize_share_code);
            let inherited_circle = self
                .resolve_inherited_circle_code(
                    window.id,
                    input.team_id,
                    input.circle_code.as_deref(),
                    parent_share_code.as_deref(),
                )
                .await?;

            let mut last_error = DirectorError::Unavailable;
            let mut saved = None;
            for _ in 0..SHARE_CODE_RETRIES {
                let share_code = generate_share_code();
                let circle_code = inherited_circle
                    .clone()
                    .unwrap_or_else(|| share_code.clone());
                let record = UpsertPlanRecord {
                    window_id: window.id,
                    team_id: input.team_id,
                    user_id,
                    share_code,
                    circle_code,
                    parent_share_code: parent_share_code.clone(),
                    foreign_stays: foreign_stays.clone(),
                    foreign_leaves: foreign_leaves.clone(),
                    foreign_ins: foreign_ins.clone(),
                    domestic_ins: domestic_ins.clone(),
                    domestic_outs: domestic_outs.clone(),
                    style_tag: style_tag.clone(),
                    manifesto: manifesto.clone(),
                };
                match self.repository.upsert_plan(&record).await {
                    Ok(code) => {
                        saved = Some(code);
                        break;
                    }
                    Err(DirectorError::ShareCodeTaken) => {
                        last_error = DirectorError::ShareCodeTaken;
                    }
                    Err(error) => return Err(error),
                }
            }
            saved.ok_or(last_error)?
        };

        self.get_plan_detail(&saved_share_code, Some(user_id)).await
    }

    pub async fn get_plan_detail(
        &self,
        share_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<DirectorPlanDetail, DirectorError> {
        let code = normalize_share_code(share_code)
            .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
        let plan = self
            .repository
            .find_plan_by_share_code(&code, viewer)
            .await?
            .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
        let comments = self
            .repository
            .list_plan_comments(plan.id, MAX_COMMENTS_RETURNED)
            .await?;

        let comparison = if let Some(uid) = viewer {
            if uid != plan.author.user_id {
                self.repository
                    .find_plan_by_user(plan.window_id, plan.team_id, uid, Some(uid))
                    .await?
                    .filter(|mine| mine.share_code != plan.share_code)
                    .map(|mine| compute_plan_comparison(&plan, &mine))
            } else {
                None
            }
        } else {
            None
        };

        Ok(DirectorPlanDetail {
            plan,
            comments,
            comparison,
        })
    }

    pub async fn get_circle_consensus(
        &self,
        circle_code: &str,
        viewer: Option<Uuid>,
    ) -> Result<CircleConsensus, DirectorError> {
        let code = normalize_share_code(circle_code)
            .ok_or_else(|| DirectorError::NotFound("好友圈不存在".into()))?;
        let plans = self.repository.list_circle_plans(&code, viewer).await?;
        let first = plans
            .first()
            .ok_or_else(|| DirectorError::NotFound("好友圈不存在".into()))?;
        Ok(compute_circle_consensus(
            code,
            first.window_name.clone(),
            first.team_id,
            first.team_name.clone(),
            &plans,
        ))
    }

    pub async fn set_reaction(
        &self,
        share_code: &str,
        user_id: Uuid,
        value: i16,
    ) -> Result<ReactionSummary, DirectorError> {
        if !matches!(value, -1 | 0 | 1) {
            return Err(DirectorError::InvalidInput(
                "表态值必须为 1、-1 或 0".into(),
            ));
        }
        let code = normalize_share_code(share_code)
            .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
        self.repository
            .user_profile(user_id)
            .await?
            .filter(|p| p.active)
            .ok_or(DirectorError::Unauthorized)?;
        self.repository
            .set_plan_reaction(&code, user_id, value)
            .await
    }

    pub async fn create_comment(
        &self,
        share_code: &str,
        user_id: Uuid,
        content: &str,
    ) -> Result<DirectorComment, DirectorError> {
        let code = normalize_share_code(share_code)
            .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
        let trimmed = validate_comment_content(content)?;
        let profile = self
            .repository
            .user_profile(user_id)
            .await?
            .filter(|p| p.active)
            .ok_or(DirectorError::Unauthorized)?;

        self.check_text_moderation(profile.wx_open_id.as_deref(), &trimmed)
            .await?;

        self.repository
            .insert_plan_comment(&code, user_id, &trimmed)
            .await
    }

    async fn resolve_incoming_player(
        &self,
        incoming: &IncomingPlayerInput,
        team_id: i64,
        expected_foreign: bool,
        moderated_segments: &mut Vec<String>,
    ) -> Result<PlanPlayerItem, DirectorError> {
        let source = PlayerSource::parse(&incoming.source)
            .ok_or_else(|| DirectorError::InvalidInput("引援来源类型无效".into()))?;

        match source {
            PlayerSource::Pool => {
                let code = incoming
                    .candidate_code
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| DirectorError::InvalidInput("缺少候选球员编码".into()))?;
                let candidate = self
                    .repository
                    .find_pool_candidate(code)
                    .await?
                    .ok_or_else(|| {
                        DirectorError::InvalidInput(format!("候选池中不存在球员 {code}"))
                    })?;
                if candidate.is_foreign != expected_foreign {
                    return Err(DirectorError::InvalidInput(if expected_foreign {
                        "该候选球员不属于外援".into()
                    } else {
                        "该候选球员不属于国内球员".into()
                    }));
                }
                Ok(candidate.to_plan_item())
            }
            PlayerSource::Squad => {
                let player_id = incoming
                    .player_id
                    .filter(|id| *id > 0)
                    .ok_or_else(|| DirectorError::InvalidInput("缺少现役球员 ID".into()))?;
                let player = self
                    .repository
                    .find_active_squad_player(player_id, team_id)
                    .await?
                    .ok_or_else(|| {
                        DirectorError::InvalidInput("所选球员不是其他中超球队现役球员".into())
                    })?;
                if player.is_foreign() != expected_foreign {
                    return Err(DirectorError::InvalidInput(if expected_foreign {
                        "所选现役球员不属于外援".into()
                    } else {
                        "所选现役球员不属于国内球员".into()
                    }));
                }
                Ok(player.to_plan_item())
            }
            PlayerSource::Custom => {
                let name = incoming
                    .player_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .ok_or_else(|| DirectorError::InvalidInput("自由提名球员必须填写姓名".into()))?;
                if name.chars().count() > MAX_CUSTOM_NAME_CHARS {
                    return Err(DirectorError::InvalidInput(format!(
                        "球员姓名最多 {MAX_CUSTOM_NAME_CHARS} 个字"
                    )));
                }
                if name.chars().any(|c| c.is_control()) {
                    return Err(DirectorError::InvalidInput(
                        "球员姓名包含不支持的字符".into(),
                    ));
                }

                let position = incoming
                    .position
                    .as_deref()
                    .and_then(validate_position)
                    .ok_or_else(|| {
                        DirectorError::InvalidInput("自由提名球员位置必须为 GK/DF/MF/FW".into())
                    })?;

                let league_tag = incoming
                    .league_tag
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        if expected_foreign {
                            "OTHER".into()
                        } else {
                            "CSL".into()
                        }
                    });
                if league_tag.chars().count() > MAX_CUSTOM_LEAGUE_CHARS
                    || league_tag.chars().any(|c| c.is_control())
                {
                    return Err(DirectorError::InvalidInput("联赛标签格式无效".into()));
                }

                let current_club = incoming
                    .current_club
                    .as_deref()
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "自由身".into());
                if current_club.chars().count() > MAX_CUSTOM_CLUB_CHARS
                    || current_club.chars().any(|c| c.is_control())
                {
                    return Err(DirectorError::InvalidInput("所属俱乐部格式无效".into()));
                }

                moderated_segments.push(name.to_string());
                if current_club != "自由身" {
                    moderated_segments.push(current_club.clone());
                }

                Ok(PlanPlayerItem {
                    key: normalize_custom_name_key(name),
                    source: PlayerSource::Custom,
                    player_id: None,
                    candidate_code: None,
                    player_name: name.to_string(),
                    position: position.to_string(),
                    country: if expected_foreign {
                        "海外".into()
                    } else {
                        "中国".into()
                    },
                    is_foreign: expected_foreign,
                    league_tag,
                    current_club,
                    avatar_url: None,
                })
            }
        }
    }

    async fn resolve_inherited_circle_code(
        &self,
        window_id: i64,
        team_id: i64,
        requested_circle_code: Option<&str>,
        parent_share_code: Option<&str>,
    ) -> Result<Option<String>, DirectorError> {
        if let Some(code) = requested_circle_code.and_then(normalize_share_code)
            && self
                .repository
                .circle_exists(window_id, team_id, &code)
                .await?
        {
            return Ok(Some(code));
        }
        if let Some(pcode) = parent_share_code
            && let Some(circle) = self
                .repository
                .find_circle_code_by_share_code(window_id, team_id, pcode)
                .await?
        {
            return Ok(Some(circle));
        }
        Ok(None)
    }

    async fn check_text_moderation(
        &self,
        open_id: Option<&str>,
        text: &str,
    ) -> Result<(), DirectorError> {
        self.content_security
            .check_text(ModerationScene::Comment, open_id, text)
            .await
            .map_err(|error| match error {
                ContentSecurityError::Rejected => DirectorError::ContentRejected,
                ContentSecurityError::Unavailable => DirectorError::ModerationUnavailable,
            })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::TimeZone;

    use super::*;
    use crate::director::{
        domain::{DirectorAuthor, DirectorPlan, DirectorTeam, DirectorWindow, PoolCandidate},
        ports::{DirectorUserProfile, ExistingPlanCodes},
    };

    pub(crate) const FAN_A: Uuid = Uuid::from_u128(0x1111_1111_1111_1111_1111_1111_1111_1111);
    pub(crate) const FAN_B: Uuid = Uuid::from_u128(0x2222_2222_2222_2222_2222_2222_2222_2222);

    pub(crate) struct MockContentSecurity {
        pub result: Mutex<Result<(), ContentSecurityError>>,
        pub checked: Mutex<Vec<String>>,
    }

    impl MockContentSecurity {
        pub(crate) fn allow() -> Arc<Self> {
            Arc::new(Self {
                result: Mutex::new(Ok(())),
                checked: Mutex::new(Vec::new()),
            })
        }

        pub(crate) fn reject() -> Arc<Self> {
            Arc::new(Self {
                result: Mutex::new(Err(ContentSecurityError::Rejected)),
                checked: Mutex::new(Vec::new()),
            })
        }
    }

    #[async_trait]
    impl ContentSecurityPort for MockContentSecurity {
        async fn check_text(
            &self,
            _scene: ModerationScene,
            _open_id: Option<&str>,
            content: &str,
        ) -> Result<(), ContentSecurityError> {
            self.checked.lock().unwrap().push(content.to_string());
            self.result.lock().unwrap().clone()
        }

        async fn check_image(
            &self,
            _bytes: &[u8],
            _mime_type: &str,
        ) -> Result<(), ContentSecurityError> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct StoreState {
        plans: Vec<DirectorPlan>,
        comments: HashMap<Uuid, Vec<DirectorComment>>,
        reactions: HashMap<(Uuid, Uuid), i16>,
    }

    pub(crate) struct MockDirectorRepo {
        state: Mutex<StoreState>,
        pub force_unavailable: bool,
    }

    impl MockDirectorRepo {
        pub(crate) fn new() -> Arc<Self> {
            Arc::new(Self {
                state: Mutex::new(StoreState::default()),
                force_unavailable: false,
            })
        }

        fn sample_window() -> DirectorWindow {
            DirectorWindow {
                id: 1,
                code: "2026-winter".into(),
                name: "2026 冬窗".into(),
                opens_at: Utc.with_ymd_and_hms(2026, 10, 20, 0, 0, 0).unwrap(),
                closes_at: Utc.with_ymd_and_hms(2027, 3, 1, 0, 0, 0).unwrap(),
            }
        }

        fn sample_squad() -> Vec<SquadPlayer> {
            vec![
                SquadPlayer {
                    player_id: 1001,
                    team_id: DEFAULT_TEAM_ID,
                    team_name: "成都蓉城".into(),
                    player_name: "费利佩".into(),
                    shirt_number: Some(21),
                    position: Some("FW".into()),
                    country: Some("巴西".into()),
                    avatar_url: None,
                },
                SquadPlayer {
                    player_id: 1002,
                    team_id: DEFAULT_TEAM_ID,
                    team_name: "成都蓉城".into(),
                    player_name: "罗慕洛".into(),
                    shirt_number: Some(10),
                    position: Some("MF".into()),
                    country: Some("巴西".into()),
                    avatar_url: None,
                },
                SquadPlayer {
                    player_id: 1003,
                    team_id: DEFAULT_TEAM_ID,
                    team_name: "成都蓉城".into(),
                    player_name: "提莫".into(),
                    shirt_number: Some(5),
                    position: Some("DF".into()),
                    country: Some("荷兰".into()),
                    avatar_url: None,
                },
                SquadPlayer {
                    player_id: 2001,
                    team_id: DEFAULT_TEAM_ID,
                    team_name: "成都蓉城".into(),
                    player_name: "韦世豪".into(),
                    shirt_number: Some(7),
                    position: Some("FW".into()),
                    country: Some("中国".into()),
                    avatar_url: None,
                },
                SquadPlayer {
                    player_id: 2002,
                    team_id: DEFAULT_TEAM_ID,
                    team_name: "成都蓉城".into(),
                    player_name: "周定洋".into(),
                    shirt_number: Some(8),
                    position: Some("MF".into()),
                    country: Some("中国台湾".into()),
                    avatar_url: None,
                },
            ]
        }

        fn sample_pool() -> Vec<PoolCandidate> {
            vec![
                PoolCandidate {
                    code: "k1-cesinha".into(),
                    player_name: "塞西尼亚".into(),
                    position: "FW".into(),
                    country: "巴西".into(),
                    is_foreign: true,
                    league_tag: "K1".into(),
                    current_club: "大邱FC".into(),
                    tags: vec!["K联赛核心".into()],
                    avatar_url: None,
                },
                PoolCandidate {
                    code: "j1-tanaka".into(),
                    player_name: "田中".into(),
                    position: "MF".into(),
                    country: "日本".into(),
                    is_foreign: true,
                    league_tag: "J1".into(),
                    current_club: "川崎前锋".into(),
                    tags: vec!["亚外".into()],
                    avatar_url: None,
                },
                PoolCandidate {
                    code: "csl-dom-1".into(),
                    player_name: "林良铭".into(),
                    position: "FW".into(),
                    country: "中国".into(),
                    is_foreign: false,
                    league_tag: "CSL".into(),
                    current_club: "北京国安".into(),
                    tags: vec!["国脚".into()],
                    avatar_url: None,
                },
            ]
        }
    }

    #[async_trait]
    impl DirectorRepository for MockDirectorRepo {
        async fn current_window(
            &self,
            _now: DateTime<Utc>,
        ) -> Result<Option<DirectorWindow>, DirectorError> {
            if self.force_unavailable {
                return Err(DirectorError::Unavailable);
            }
            Ok(Some(Self::sample_window()))
        }

        async fn find_team(&self, team_id: i64) -> Result<Option<DirectorTeam>, DirectorError> {
            if self.force_unavailable {
                return Err(DirectorError::Unavailable);
            }
            if team_id == DEFAULT_TEAM_ID {
                Ok(Some(DirectorTeam {
                    team_id,
                    team_name: "成都蓉城".into(),
                    logo_url: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn team_active_squad(&self, team_id: i64) -> Result<Vec<SquadPlayer>, DirectorError> {
            if team_id == DEFAULT_TEAM_ID {
                Ok(Self::sample_squad())
            } else {
                Ok(vec![])
            }
        }

        async fn other_teams_active_squad(
            &self,
            _exclude_team_id: i64,
            _keyword: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<SquadPlayer>, DirectorError> {
            Ok(vec![SquadPlayer {
                player_id: 3001,
                team_id: 77681,
                team_name: "上海海港".into(),
                player_name: "奥斯卡".into(),
                shirt_number: Some(8),
                position: Some("MF".into()),
                country: Some("巴西".into()),
                avatar_url: None,
            }])
        }

        async fn find_active_squad_player(
            &self,
            player_id: i64,
            exclude_team_id: i64,
        ) -> Result<Option<SquadPlayer>, DirectorError> {
            if player_id == 3001 && exclude_team_id != 77681 {
                Ok(Some(SquadPlayer {
                    player_id: 3001,
                    team_id: 77681,
                    team_name: "上海海港".into(),
                    player_name: "奥斯卡".into(),
                    shirt_number: Some(8),
                    position: Some("MF".into()),
                    country: Some("巴西".into()),
                    avatar_url: None,
                }))
            } else {
                Ok(None)
            }
        }

        async fn list_pool_candidates(
            &self,
            _team_id: i64,
            is_foreign: bool,
            league: &str,
            keyword: Option<&str>,
            _limit: i64,
        ) -> Result<Vec<PoolCandidate>, DirectorError> {
            Ok(Self::sample_pool()
                .into_iter()
                .filter(|c| c.is_foreign == is_foreign)
                .filter(|c| league == "ALL" || c.league_tag.eq_ignore_ascii_case(league))
                .filter(|c| {
                    keyword.is_none_or(|kw| {
                        c.player_name.contains(kw) || c.current_club.contains(kw)
                    })
                })
                .collect())
        }

        async fn find_pool_candidate(
            &self,
            code: &str,
        ) -> Result<Option<PoolCandidate>, DirectorError> {
            Ok(Self::sample_pool().into_iter().find(|c| c.code == code))
        }

        async fn count_team_plans(&self, window_id: i64, team_id: i64) -> Result<i64, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .filter(|p| p.window_id == window_id && p.team_id == team_id)
                .count() as i64)
        }

        async fn find_user_plan_codes(
            &self,
            window_id: i64,
            team_id: i64,
            user_id: Uuid,
        ) -> Result<Option<ExistingPlanCodes>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .find(|p| {
                    p.window_id == window_id && p.team_id == team_id && p.author.user_id == user_id
                })
                .map(|p| ExistingPlanCodes {
                    share_code: p.share_code.clone(),
                    circle_code: p.circle_code.clone(),
                    parent_share_code: p.parent_share_code.clone(),
                }))
        }

        async fn circle_exists(
            &self,
            window_id: i64,
            team_id: i64,
            circle_code: &str,
        ) -> Result<bool, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state.plans.iter().any(|p| {
                p.window_id == window_id && p.team_id == team_id && p.circle_code == circle_code
            }))
        }

        async fn find_circle_code_by_share_code(
            &self,
            window_id: i64,
            team_id: i64,
            share_code: &str,
        ) -> Result<Option<String>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .find(|p| {
                    p.window_id == window_id && p.team_id == team_id && p.share_code == share_code
                })
                .map(|p| p.circle_code.clone()))
        }

        async fn upsert_plan(&self, record: &UpsertPlanRecord) -> Result<String, DirectorError> {
            let mut state = self.state.lock().unwrap();
            if let Some(existing) = state.plans.iter_mut().find(|p| {
                p.window_id == record.window_id
                    && p.team_id == record.team_id
                    && p.author.user_id == record.user_id
            }) {
                existing.foreign_stays = record.foreign_stays.clone();
                existing.foreign_leaves = record.foreign_leaves.clone();
                existing.foreign_ins = record.foreign_ins.clone();
                existing.domestic_ins = record.domestic_ins.clone();
                existing.domestic_outs = record.domestic_outs.clone();
                existing.style_tag = record.style_tag.clone();
                existing.manifesto = record.manifesto.clone();
                existing.updated_at = Utc::now();
                return Ok(existing.share_code.clone());
            }

            let plan = DirectorPlan {
                id: Uuid::new_v4(),
                window_id: record.window_id,
                window_code: "2026-winter".into(),
                window_name: "2026 冬窗".into(),
                team_id: record.team_id,
                team_name: "成都蓉城".into(),
                share_code: record.share_code.clone(),
                circle_code: record.circle_code.clone(),
                parent_share_code: record.parent_share_code.clone(),
                author: DirectorAuthor {
                    user_id: record.user_id,
                    nickname: if record.user_id == FAN_A {
                        "老王".into()
                    } else {
                        "小李".into()
                    },
                    avatar_url: None,
                },
                is_mine: false,
                foreign_stays: record.foreign_stays.clone(),
                foreign_leaves: record.foreign_leaves.clone(),
                foreign_ins: record.foreign_ins.clone(),
                domestic_ins: record.domestic_ins.clone(),
                domestic_outs: record.domestic_outs.clone(),
                style_tag: record.style_tag.clone(),
                manifesto: record.manifesto.clone(),
                likes_count: 0,
                dislikes_count: 0,
                my_reaction: 0,
                comments_count: 0,
                updated_at: Utc::now(),
            };
            state.plans.push(plan);
            Ok(record.share_code.clone())
        }

        async fn find_plan_by_share_code(
            &self,
            share_code: &str,
            viewer_user_id: Option<Uuid>,
        ) -> Result<Option<DirectorPlan>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .find(|p| p.share_code == share_code)
                .map(|p| {
                    let mut cloned = p.clone();
                    cloned.is_mine = viewer_user_id == Some(cloned.author.user_id);
                    cloned.my_reaction = viewer_user_id
                        .and_then(|uid| state.reactions.get(&(cloned.id, uid)).copied())
                        .unwrap_or(0);
                    cloned.comments_count = state
                        .comments
                        .get(&cloned.id)
                        .map(|c| c.len() as i64)
                        .unwrap_or(0);
                    cloned
                }))
        }

        async fn find_plan_by_user(
            &self,
            window_id: i64,
            team_id: i64,
            user_id: Uuid,
            viewer_user_id: Option<Uuid>,
        ) -> Result<Option<DirectorPlan>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .find(|p| {
                    p.window_id == window_id && p.team_id == team_id && p.author.user_id == user_id
                })
                .map(|p| {
                    let mut cloned = p.clone();
                    cloned.is_mine = viewer_user_id == Some(cloned.author.user_id);
                    cloned
                }))
        }

        async fn list_plan_comments(
            &self,
            plan_id: Uuid,
            _limit: i64,
        ) -> Result<Vec<DirectorComment>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state.comments.get(&plan_id).cloned().unwrap_or_default())
        }

        async fn list_circle_plans(
            &self,
            circle_code: &str,
            viewer_user_id: Option<Uuid>,
        ) -> Result<Vec<DirectorPlan>, DirectorError> {
            let state = self.state.lock().unwrap();
            Ok(state
                .plans
                .iter()
                .filter(|p| p.circle_code == circle_code)
                .map(|p| {
                    let mut cloned = p.clone();
                    cloned.is_mine = viewer_user_id == Some(cloned.author.user_id);
                    cloned
                })
                .collect())
        }

        async fn set_plan_reaction(
            &self,
            share_code: &str,
            user_id: Uuid,
            value: i16,
        ) -> Result<ReactionSummary, DirectorError> {
            let mut state = self.state.lock().unwrap();
            let plan_id = state
                .plans
                .iter()
                .find(|p| p.share_code == share_code)
                .map(|p| p.id)
                .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
            if value == 0 {
                state.reactions.remove(&(plan_id, user_id));
            } else {
                state.reactions.insert((plan_id, user_id), value);
            }
            let likes_count = state
                .reactions
                .iter()
                .filter(|((pid, _), v)| *pid == plan_id && **v == 1)
                .count() as i64;
            let dislikes_count = state
                .reactions
                .iter()
                .filter(|((pid, _), v)| *pid == plan_id && **v == -1)
                .count() as i64;
            if let Some(plan) = state.plans.iter_mut().find(|p| p.id == plan_id) {
                plan.likes_count = likes_count;
                plan.dislikes_count = dislikes_count;
            }
            Ok(ReactionSummary {
                likes_count,
                dislikes_count,
                my_reaction: value,
            })
        }

        async fn insert_plan_comment(
            &self,
            share_code: &str,
            user_id: Uuid,
            content: &str,
        ) -> Result<DirectorComment, DirectorError> {
            let mut state = self.state.lock().unwrap();
            let plan_id = state
                .plans
                .iter()
                .find(|p| p.share_code == share_code)
                .map(|p| p.id)
                .ok_or_else(|| DirectorError::NotFound("方案不存在".into()))?;
            let comment = DirectorComment {
                id: Uuid::new_v4(),
                author: DirectorAuthor {
                    user_id,
                    nickname: "小李".into(),
                    avatar_url: None,
                },
                content: content.to_string(),
                created_at: Utc::now(),
            };
            state
                .comments
                .entry(plan_id)
                .or_default()
                .insert(0, comment.clone());
            Ok(comment)
        }

        async fn user_profile(
            &self,
            user_id: Uuid,
        ) -> Result<Option<DirectorUserProfile>, DirectorError> {
            if user_id == FAN_A || user_id == FAN_B {
                Ok(Some(DirectorUserProfile {
                    active: true,
                    favorite_team_id: Some(DEFAULT_TEAM_ID),
                    wx_open_id: Some("openid-test".into()),
                }))
            } else {
                Ok(None)
            }
        }
    }

    pub(crate) fn build_test_service(
        repo: Arc<MockDirectorRepo>,
        security: Arc<MockContentSecurity>,
    ) -> DirectorService {
        DirectorService::new(repo, security, Arc::new(Utc::now))
    }

    #[tokio::test]
    async fn save_plan_enforces_foreign_quota_and_deduplication() {
        let repo = MockDirectorRepo::new();
        let sec = MockContentSecurity::allow();
        let svc = build_test_service(repo, sec);

        // 0 foreign -> rejected
        let empty = SavePlanInput {
            team_id: DEFAULT_TEAM_ID,
            circle_code: None,
            parent_share_code: None,
            foreign_stay_ids: vec![],
            foreign_leave_ids: vec![1001, 1002, 1003],
            foreign_ins: vec![],
            domestic_ins: vec![],
            domestic_outs: vec![],
            manifesto: String::new(),
        };
        assert!(matches!(
            svc.save_my_plan(FAN_A, empty).await,
            Err(DirectorError::InvalidInput(_))
        ));

        // 6 foreign (3 stays + 3 ins) -> rejected
        let too_many = SavePlanInput {
            team_id: DEFAULT_TEAM_ID,
            circle_code: None,
            parent_share_code: None,
            foreign_stay_ids: vec![1001, 1002, 1003],
            foreign_leave_ids: vec![],
            foreign_ins: vec![
                IncomingPlayerInput {
                    source: "pool".into(),
                    candidate_code: Some("k1-cesinha".into()),
                    player_id: None,
                    player_name: None,
                    position: None,
                    league_tag: None,
                    current_club: None,
                },
                IncomingPlayerInput {
                    source: "pool".into(),
                    candidate_code: Some("j1-tanaka".into()),
                    player_id: None,
                    player_name: None,
                    position: None,
                    league_tag: None,
                    current_club: None,
                },
                IncomingPlayerInput {
                    source: "custom".into(),
                    candidate_code: None,
                    player_id: None,
                    player_name: Some("新外援".into()),
                    position: Some("FW".into()),
                    league_tag: Some("K2".into()),
                    current_club: Some("釜山".into()),
                },
            ],
            domestic_ins: vec![],
            domestic_outs: vec![],
            manifesto: String::new(),
        };
        assert!(matches!(
            svc.save_my_plan(FAN_A, too_many).await,
            Err(DirectorError::InvalidInput(_))
        ));

        // stay + leave overlap -> rejected
        let overlap = SavePlanInput {
            team_id: DEFAULT_TEAM_ID,
            circle_code: None,
            parent_share_code: None,
            foreign_stay_ids: vec![1001],
            foreign_leave_ids: vec![1001],
            foreign_ins: vec![],
            domestic_ins: vec![],
            domestic_outs: vec![],
            manifesto: String::new(),
        };
        assert!(matches!(
            svc.save_my_plan(FAN_A, overlap).await,
            Err(DirectorError::InvalidInput(_))
        ));
    }

    #[tokio::test]
    async fn save_plan_and_comment_call_content_security_and_block_rejected_text() {
        let repo = MockDirectorRepo::new();
        let sec = MockContentSecurity::reject();
        let svc = build_test_service(repo.clone(), sec.clone());

        let plan_with_manifesto = SavePlanInput {
            team_id: DEFAULT_TEAM_ID,
            circle_code: None,
            parent_share_code: None,
            foreign_stay_ids: vec![1001],
            foreign_leave_ids: vec![],
            foreign_ins: vec![],
            domestic_ins: vec![],
            domestic_outs: vec![],
            manifesto: "违规宣言测试".into(),
        };
        assert_eq!(
            svc.save_my_plan(FAN_A, plan_with_manifesto).await,
            Err(DirectorError::ContentRejected)
        );
        assert_eq!(sec.checked.lock().unwrap().len(), 1);

        // Allow plan creation, then reject comment
        *sec.result.lock().unwrap() = Ok(());
        let saved = svc
            .save_my_plan(
                FAN_A,
                SavePlanInput {
                    team_id: DEFAULT_TEAM_ID,
                    circle_code: None,
                    parent_share_code: None,
                    foreign_stay_ids: vec![1001],
                    foreign_leave_ids: vec![],
                    foreign_ins: vec![],
                    domestic_ins: vec![],
                    domestic_outs: vec![],
                    manifesto: "合理补强".into(),
                },
            )
            .await
            .unwrap();

        *sec.result.lock().unwrap() = Err(ContentSecurityError::Rejected);
        assert_eq!(
            svc.create_comment(&saved.plan.share_code, FAN_B, "违规锐评")
                .await,
            Err(DirectorError::ContentRejected)
        );
    }

    #[tokio::test]
    async fn circle_inheritance_comparison_and_consensus_flow() {
        let repo = MockDirectorRepo::new();
        let sec = MockContentSecurity::allow();
        let svc = build_test_service(repo, sec);

        // Fan A creates plan A (stays: 1002, leaves: 1001, ins: k1-cesinha)
        let plan_a = svc
            .save_my_plan(
                FAN_A,
                SavePlanInput {
                    team_id: DEFAULT_TEAM_ID,
                    circle_code: None,
                    parent_share_code: None,
                    foreign_stay_ids: vec![1002],
                    foreign_leave_ids: vec![1001],
                    foreign_ins: vec![IncomingPlayerInput {
                        source: "pool".into(),
                        candidate_code: Some("k1-cesinha".into()),
                        player_id: None,
                        player_name: None,
                        position: None,
                        league_tag: None,
                        current_club: None,
                    }],
                    domestic_ins: vec![],
                    domestic_outs: vec![2001],
                    manifesto: "K联赛淘宝！".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(plan_a.plan.style_tag, "K联赛淘宝流");
        assert_eq!(plan_a.plan.share_code, plan_a.plan.circle_code);

        // Fan B joins via Fan A's circle_code (stays: 1001, 1002)
        let plan_b = svc
            .save_my_plan(
                FAN_B,
                SavePlanInput {
                    team_id: DEFAULT_TEAM_ID,
                    circle_code: Some(plan_a.plan.circle_code.clone()),
                    parent_share_code: Some(plan_a.plan.share_code.clone()),
                    foreign_stay_ids: vec![1001, 1002],
                    foreign_leave_ids: vec![],
                    foreign_ins: vec![],
                    domestic_ins: vec![],
                    domestic_outs: vec![],
                    manifesto: "留下费利佩".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(plan_b.plan.circle_code, plan_a.plan.circle_code);

        // Fan B views Fan A's plan -> gets 1v1 comparison with leave disagreement on 费利佩
        let detail_for_b = svc
            .get_plan_detail(&plan_a.plan.share_code, Some(FAN_B))
            .await
            .unwrap();
        let cmp = detail_for_b.comparison.expect("comparison should exist");
        assert_eq!(cmp.my_share_code, plan_b.plan.share_code);
        assert_eq!(cmp.overlap_percent, 50);
        assert_eq!(cmp.leave_disagreements.len(), 1);
        assert_eq!(cmp.leave_disagreements[0].player.player_name, "费利佩");

        // Circle consensus now unlocked (2 members)
        let consensus = svc
            .get_circle_consensus(&plan_a.plan.circle_code, Some(FAN_B))
            .await
            .unwrap();
        assert!(consensus.unlocked);
        assert_eq!(consensus.member_count, 2);
        assert_eq!(consensus.top_foreign[0].player.player_name, "罗慕洛");
        assert_eq!(consensus.top_foreign[0].pick_rate, 100);
    }
}
