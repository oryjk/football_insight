use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use chrono::{DateTime, Duration, Utc};
use football_insight_service_backend_rs::{
    auth::{
        adapters::{
            integration::minio_avatar_storage_port::MinioOrInlineAvatarStoragePort,
            security::jwt_token_port::JwtTokenPort,
            web::{
                handlers::{AuthWebConfig, get_current_user_handler, update_profile_handler},
            },
        },
        application::{
            get_current_user::GetCurrentUserUseCase,
            update_user_profile::UpdateUserProfileUseCase,
        },
        domain::{
            user::{AuthUser, StoredAuthUser, resolve_profile_completed, sanitize_public_display_name},
            wechat::{IssuedInviteCode, WechatOauthProfile},
        },
        ports::{auth_repository::AuthRepository, token_port::TokenPort},
    },
    content_security::{ContentSecurityError, ContentSecurityPort, ModerationScene},
    daily_topics::{
        adapters::web::routes::telegraph_interactions_routes,
        application::telegraph_interactions::TelegraphInteractionsService,
        domain::interactions::{
            CreateTelegraphCommentInput, ReactionMutation, TelegraphComment,
            TelegraphCommentAuthor, TelegraphCommentLikeSummary, TelegraphCommentReplyRef,
            TelegraphCommentsPage, TelegraphCommentsPosition, TelegraphInteractionError,
            TelegraphReactionKind, TelegraphReactionSummary,
        },
        ports::daily_topics_repository::TelegraphInteractionsRepository,
    },
};
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

struct MockModeration;

#[async_trait]
impl ContentSecurityPort for MockModeration {
    async fn check_text(
        &self,
        _scene: ModerationScene,
        _open_id: Option<&str>,
        text: &str,
    ) -> Result<(), ContentSecurityError> {
        if text.contains("违禁词") {
            Err(ContentSecurityError::Rejected)
        } else {
            Ok(())
        }
    }

    async fn check_image(&self, bytes: &[u8], _mime_type: &str) -> Result<(), ContentSecurityError> {
        if bytes.windows(3).any(|w| w == b"BAD") {
            Err(ContentSecurityError::Rejected)
        } else {
            Ok(())
        }
    }
}

struct InMemoryUsers {
    users: Mutex<HashMap<Uuid, AuthUser>>,
    custom_profiles: Mutex<HashMap<Uuid, bool>>,
}

impl InMemoryUsers {
    fn new(user: AuthUser) -> Self {
        let mut users = HashMap::new();
        users.insert(user.id, user);
        Self {
            users: Mutex::new(users),
            custom_profiles: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl AuthRepository for InMemoryUsers {
    async fn create_user_with_invite(&self, _: &str, _: &str, _: &str) -> anyhow::Result<AuthUser> {
        unreachable!()
    }
    async fn find_user_by_account_identifier(&self, _: &str) -> anyhow::Result<Option<StoredAuthUser>> {
        unreachable!()
    }
    async fn find_user_by_wechat_open_id(&self, _: &str) -> anyhow::Result<Option<AuthUser>> {
        unreachable!()
    }
    async fn bind_wechat_to_user(&self, _: Uuid, _: &WechatOauthProfile) -> anyhow::Result<AuthUser> {
        unreachable!()
    }
    async fn create_user_with_invite_and_wechat(&self, _: &str, _: &str, _: &str, _: &WechatOauthProfile) -> anyhow::Result<AuthUser> {
        unreachable!()
    }
    async fn create_user_with_invite_and_mini_program_wechat(&self, _: &str, _: &WechatOauthProfile) -> anyhow::Result<AuthUser> {
        unreachable!()
    }
    async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<AuthUser>> {
        Ok(self.users.lock().unwrap().get(&user_id).cloned())
    }
    async fn issue_invite_code_for_wechat_follower(&self, _: &str) -> anyhow::Result<IssuedInviteCode> {
        unreachable!()
    }
    async fn mark_wechat_follower_unsubscribed(&self, _: &str) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn reset_password_with_invite(&self, _: &str, _: &str, _: &str) -> anyhow::Result<()> {
        unreachable!()
    }
    async fn has_custom_profile(&self, user_id: Uuid) -> anyhow::Result<Option<bool>> {
        Ok(Some(*self.custom_profiles.lock().unwrap().get(&user_id).unwrap_or(&false)))
    }
    async fn update_user_profile(
        &self,
        user_id: Uuid,
        display_name: &str,
        avatar_url: &str,
    ) -> anyhow::Result<AuthUser> {
        let mut guard = self.users.lock().unwrap();
        let user = guard.get_mut(&user_id).unwrap();
        user.display_name = Some(display_name.to_string());
        user.avatar_url = Some(avatar_url.to_string());
        user.updated_at = Utc::now();
        self.custom_profiles.lock().unwrap().insert(user_id, true);
        Ok(user.clone())
    }
}

struct StoredComment {
    id: Uuid,
    topic_id: String,
    entry_id: String,
    user_id: Uuid,
    reply_to_comment_id: Option<Uuid>,
    content: String,
    status: String,
    created_at: DateTime<Utc>,
}

struct InMemoryInteractions {
    users: Arc<InMemoryUsers>,
    comments_enabled: Mutex<bool>,
    reactions: Mutex<HashMap<(String, Uuid), TelegraphReactionKind>>,
    comment_likes: Mutex<HashSet<(Uuid, Uuid)>>,
    comments: Mutex<Vec<StoredComment>>,
    rate_limited_users: Mutex<Vec<Uuid>>,
}

impl InMemoryInteractions {
    fn new(users: Arc<InMemoryUsers>) -> Self {
        Self {
            users,
            comments_enabled: Mutex::new(true),
            reactions: Mutex::new(HashMap::new()),
            comment_likes: Mutex::new(HashSet::new()),
            comments: Mutex::new(Vec::new()),
            rate_limited_users: Mutex::new(Vec::new()),
        }
    }

    fn reaction_summary(&self, entry_id: &str, viewer: Uuid) -> TelegraphReactionSummary {
        let map = self.reactions.lock().unwrap();
        let likes_count = map
            .iter()
            .filter(|((eid, _), kind)| eid == entry_id && **kind == TelegraphReactionKind::Like)
            .count() as u64;
        let dislikes_count = map
            .iter()
            .filter(|((eid, _), kind)| eid == entry_id && **kind == TelegraphReactionKind::Dislike)
            .count() as u64;
        let mine = map.get(&(entry_id.to_string(), viewer)).copied();
        TelegraphReactionSummary {
            entry_id: entry_id.to_string(),
            likes_count,
            dislikes_count,
            liked_by_me: mine == Some(TelegraphReactionKind::Like),
            disliked_by_me: mine == Some(TelegraphReactionKind::Dislike),
        }
    }
}

#[async_trait]
impl TelegraphInteractionsRepository for InMemoryInteractions {
    async fn set_entry_reaction(
        &self,
        _team_id: i64,
        topic_id: &str,
        entry_id: &str,
        user_id: Uuid,
        mutation: ReactionMutation,
    ) -> Result<TelegraphReactionSummary, TelegraphInteractionError> {
        if topic_id != "topic-1" || entry_id != "entry-1" {
            return Err(TelegraphInteractionError::TopicOrEntryNotFound);
        }
        {
            let mut map = self.reactions.lock().unwrap();
            match mutation {
                ReactionMutation::Set(kind) => {
                    map.insert((entry_id.to_string(), user_id), kind);
                }
                ReactionMutation::Clear(kind) => {
                    if map.get(&(entry_id.to_string(), user_id)).copied() == Some(kind) {
                        map.remove(&(entry_id.to_string(), user_id));
                    }
                }
            }
        }
        Ok(self.reaction_summary(entry_id, user_id))
    }

    async fn list_entry_comments(
        &self,
        _team_id: i64,
        topic_id: &str,
        entry_id: &str,
        viewer_user_id: Option<Uuid>,
        after: Option<&TelegraphCommentsPosition>,
        limit: usize,
    ) -> Result<TelegraphCommentsPage, TelegraphInteractionError> {
        if topic_id != "topic-1" || entry_id != "entry-1" {
            return Err(TelegraphInteractionError::TopicOrEntryNotFound);
        }
        if !*self.comments_enabled.lock().unwrap() {
            return Ok(TelegraphCommentsPage {
                comments_enabled: false,
                total_count: 0,
                items: Vec::new(),
                next_cursor: None,
                has_more: false,
            });
        }
        let comments = self.comments.lock().unwrap();
        let comment_likes = self.comment_likes.lock().unwrap();
        let users = self.users.users.lock().unwrap();
        let mut visible: Vec<&StoredComment> = comments
            .iter()
            .filter(|c| c.topic_id == topic_id && c.entry_id == entry_id && c.status == "visible")
            .collect();
        visible.sort_by(|a, b| b.created_at.cmp(&a.created_at).then_with(|| b.id.cmp(&a.id)));
        let total_count = visible.len() as u64;
        let filtered: Vec<TelegraphComment> = visible
            .into_iter()
            .filter(|c| {
                after.is_none_or(|pos| {
                    c.created_at < pos.created_at
                        || (c.created_at == pos.created_at && c.id < pos.id)
                })
            })
            .take(limit + 1)
            .map(|c| {
                let author = users.get(&c.user_id).unwrap();
                let reply_to = c.reply_to_comment_id.and_then(|rid| {
                    comments
                        .iter()
                        .find(|rc| rc.id == rid && rc.status == "visible")
                        .and_then(|rc| users.get(&rc.user_id))
                        .map(|ru| TelegraphCommentReplyRef {
                            comment_id: rid,
                            author_display_name: sanitize_public_display_name(
                                ru.display_name.as_deref(),
                            ),
                        })
                });
                let likes_count = comment_likes
                    .iter()
                    .filter(|(cid, _)| *cid == c.id)
                    .count() as u64;
                let liked_by_me = viewer_user_id.is_some_and(|uid| comment_likes.contains(&(c.id, uid)));
                let replies_count = comments
                    .iter()
                    .filter(|child| {
                        child.reply_to_comment_id == Some(c.id)
                            && child.entry_id == c.entry_id
                            && child.status == "visible"
                    })
                    .count() as u64;
                TelegraphComment {
                    id: c.id,
                    topic_id: c.topic_id.clone(),
                    entry_id: c.entry_id.clone(),
                    author: TelegraphCommentAuthor {
                        user_id: author.id,
                        display_name: sanitize_public_display_name(author.display_name.as_deref()),
                        avatar_url: author.avatar_url.clone(),
                    },
                    reply_to,
                    content: c.content.clone(),
                    likes_count,
                    liked_by_me,
                    replies_count,
                    is_mine: viewer_user_id == Some(c.user_id),
                    created_at: c.created_at,
                }
            })
            .collect();
        let has_more = filtered.len() > limit;
        let mut items = filtered;
        items.truncate(limit);
        let next_cursor = if has_more {
            items.last().map(|c| TelegraphCommentsPosition {
                created_at: c.created_at,
                id: c.id,
            })
        } else {
            None
        };
        Ok(TelegraphCommentsPage {
            comments_enabled: true,
            total_count,
            items,
            next_cursor,
            has_more,
        })
    }

    async fn check_comment_eligibility(
        &self,
        _team_id: i64,
        topic_id: &str,
        entry_id: &str,
        user_id: Uuid,
        reply_to_comment_id: Option<Uuid>,
    ) -> Result<Option<String>, TelegraphInteractionError> {
        if !*self.comments_enabled.lock().unwrap() {
            return Err(TelegraphInteractionError::CommentsDisabled);
        }
        let user = self
            .users
            .users
            .lock()
            .unwrap()
            .get(&user_id)
            .cloned()
            .ok_or(TelegraphInteractionError::Unauthorized)?;
        let custom = *self
            .users
            .custom_profiles
            .lock()
            .unwrap()
            .get(&user_id)
            .unwrap_or(&false);
        if !resolve_profile_completed(
            user.display_name.as_deref(),
            &user.account_identifier,
            user.avatar_url.as_deref(),
            Some(custom),
        ) {
            return Err(TelegraphInteractionError::ProfileIncomplete);
        }
        if self.rate_limited_users.lock().unwrap().contains(&user_id) {
            return Err(TelegraphInteractionError::RateLimited);
        }
        if topic_id != "topic-1" || entry_id != "entry-1" {
            return Err(TelegraphInteractionError::TopicOrEntryNotFound);
        }
        if let Some(reply_id) = reply_to_comment_id {
            let valid = self
                .comments
                .lock()
                .unwrap()
                .iter()
                .any(|c| c.id == reply_id && c.entry_id == entry_id && c.status == "visible");
            if !valid {
                return Err(TelegraphInteractionError::InvalidInput("回复目标无效"));
            }
        }
        Ok(None)
    }

    async fn insert_entry_comment(
        &self,
        input: &CreateTelegraphCommentInput,
    ) -> Result<(TelegraphComment, u64), TelegraphInteractionError> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        {
            let mut comments = self.comments.lock().unwrap();
            comments.push(StoredComment {
                id,
                topic_id: input.topic_id.clone(),
                entry_id: input.entry_id.clone(),
                user_id: input.user_id,
                reply_to_comment_id: input.reply_to_comment_id,
                content: input.content.clone(),
                status: "visible".to_string(),
                created_at: now,
            });
        }
        let page = self
            .list_entry_comments(
                input.team_id,
                &input.topic_id,
                &input.entry_id,
                Some(input.user_id),
                None,
                50,
            )
            .await?;
        let created = page.items.into_iter().find(|c| c.id == id).unwrap();
        Ok((created, page.total_count))
    }

    async fn delete_entry_comment(
        &self,
        _team_id: i64,
        topic_id: &str,
        entry_id: &str,
        comment_id: Uuid,
        user_id: Uuid,
    ) -> Result<u64, TelegraphInteractionError> {
        let mut comments = self.comments.lock().unwrap();
        let Some(target) = comments.iter_mut().find(|c| {
            c.id == comment_id
                && c.topic_id == topic_id
                && c.entry_id == entry_id
                && c.user_id == user_id
                && c.status != "deleted"
        }) else {
            return Err(TelegraphInteractionError::TopicOrEntryNotFound);
        };
        target.status = "deleted".to_string();
        let remaining = comments
            .iter()
            .filter(|c| c.topic_id == topic_id && c.entry_id == entry_id && c.status == "visible")
            .count() as u64;
        Ok(remaining)
    }

    async fn set_comment_like(
        &self,
        _team_id: i64,
        topic_id: &str,
        entry_id: &str,
        comment_id: Uuid,
        user_id: Uuid,
        active: bool,
    ) -> Result<TelegraphCommentLikeSummary, TelegraphInteractionError> {
        if !*self.comments_enabled.lock().unwrap() {
            return Err(TelegraphInteractionError::CommentsDisabled);
        }
        let exists = self.comments.lock().unwrap().iter().any(|c| {
            c.id == comment_id
                && c.topic_id == topic_id
                && c.entry_id == entry_id
                && c.status == "visible"
        });
        if !exists {
            return Err(TelegraphInteractionError::TopicOrEntryNotFound);
        }
        let mut comment_likes = self.comment_likes.lock().unwrap();
        if active {
            comment_likes.insert((comment_id, user_id));
        } else {
            comment_likes.remove(&(comment_id, user_id));
        }
        let likes_count = comment_likes
            .iter()
            .filter(|(cid, _)| *cid == comment_id)
            .count() as u64;
        let liked_by_me = comment_likes.contains(&(comment_id, user_id));
        Ok(TelegraphCommentLikeSummary {
            comment_id,
            likes_count,
            liked_by_me,
        })
    }
}

fn valid_png_data_url() -> String {
    let bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}

fn bad_png_data_url() -> String {
    let bytes = b"\x89PNG\r\n\x1a\nBAD";
    format!("data:image/png;base64,{}", STANDARD.encode(bytes))
}

fn build_test_app() -> (
    Router,
    String,
    Uuid,
    Arc<InMemoryInteractions>,
) {
    let user_id = Uuid::new_v4();
    let user = AuthUser {
        id: user_id,
        account_identifier: "13800138000".to_string(),
        display_name: Some("13800138000".to_string()),
        invite_code: None,
        avatar_url: None,
        has_wechat_binding: false,
        membership_tier: "V3".to_string(),
        membership_expires_at: None,
        membership_benefits_enabled: true,
        ticket_watch_poll_interval_seconds: 120,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let users = Arc::new(InMemoryUsers::new(user));
    let token_port: Arc<dyn TokenPort> = Arc::new(JwtTokenPort::new("test-jwt-secret".to_string()));
    let token = token_port
        .issue_token(user_id, "13800138000", Utc::now() + Duration::hours(1))
        .unwrap();
    let moderation: Arc<dyn ContentSecurityPort> = Arc::new(MockModeration);
    let get_me = Arc::new(GetCurrentUserUseCase::new(users.clone(), token_port.clone()));
    let update_profile = Arc::new(UpdateUserProfileUseCase::new(
        users.clone(),
        token_port.clone(),
        Arc::new(MinioOrInlineAvatarStoragePort::new(None)),
        moderation.clone(),
    ));
    let interactions_repo = Arc::new(InMemoryInteractions::new(users));
    let interactions_svc = Arc::new(TelegraphInteractionsService::new(
        interactions_repo.clone(),
        moderation,
    ));

    let auth_me = get_me.clone();
    let auth_me_for_put = get_me;
    let _cfg = AuthWebConfig {
        wechat_app_id: "wx-test".into(),
    };
    let app = Router::new()
        .route(
            "/api/v1/auth/me",
            axum::routing::get(move |headers| get_current_user_handler(headers, auth_me.clone())),
        )
        .route(
            "/api/v1/auth/me/profile",
            axum::routing::put(move |headers, payload| {
                update_profile_handler(
                    headers,
                    update_profile.clone(),
                    auth_me_for_put.clone(),
                    payload,
                )
            }),
        )
        .merge(telegraph_interactions_routes(interactions_svc, token_port));

    (app, token, user_id, interactions_repo)
}

async fn send(
    app: Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(tok) = token {
        builder = builder.header("Authorization", format!("Bearer {tok}"));
    }
    let req = if let Some(json_body) = body {
        builder
            .header("content-type", "application/json")
            .body(Body::from(json_body.to_string()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = to_bytes(resp.into_body(), 1_000_000).await.unwrap();
    let parsed = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, parsed)
}

#[tokio::test]
async fn profile_gate_enforces_auth_valid_nickname_avatar_and_moderation() {
    let (app, token, _, _) = build_test_app();

    // Unauthenticated
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        None,
        Some(json!({ "display_name": "成都球迷", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // /me initially reports profile_completed = false for phone-number user
    let (status, me) = send(app.clone(), "GET", "/api/v1/auth/me", Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(me["profile_completed"], false);

    // Rejects phone-number nickname
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "13800138000", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Rejects missing avatar
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "凤凰山老哥" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Rejects moderated nickname / image
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "违禁词昵称", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "凤凰山老哥", "avatar_data_url": bad_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Valid update returns profile_completed = true and updates user
    let (status, updated) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "凤凰山老哥", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(updated["profile_completed"], true);
    assert_eq!(updated["display_name"], "凤凰山老哥");
    assert!(updated["avatar_url"].as_str().unwrap().starts_with("data:image/png;base64,"));

    // Subsequent nickname-only update reuses existing avatar
    let (status, renamed) = send(
        app,
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "南看台12号" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(renamed["display_name"], "南看台12号");
    assert_eq!(renamed["profile_completed"], true);
}

#[tokio::test]
async fn reactions_require_login_and_switch_mutually_exclusively() {
    let (app, token, _, _) = build_test_app();
    let like_uri = "/api/v1/daily-topics/topic-1/entries/entry-1/like";
    let dislike_uri = "/api/v1/daily-topics/topic-1/entries/entry-1/dislike";

    assert_eq!(
        send(app.clone(), "PUT", like_uri, None, None).await.0,
        StatusCode::UNAUTHORIZED
    );

    // Like succeeds even before setting avatar/nickname
    let (status, res) = send(app.clone(), "PUT", like_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["likes_count"], 1);
    assert_eq!(res["dislikes_count"], 0);
    assert_eq!(res["liked_by_me"], true);
    assert_eq!(res["disliked_by_me"], false);

    // Switch to dislike clears like
    let (status, res) = send(app.clone(), "PUT", dislike_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["likes_count"], 0);
    assert_eq!(res["dislikes_count"], 1);
    assert_eq!(res["liked_by_me"], false);
    assert_eq!(res["disliked_by_me"], true);

    // Clear dislike
    let (status, res) = send(app, "DELETE", dislike_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["likes_count"], 0);
    assert_eq!(res["dislikes_count"], 0);
    assert_eq!(res["disliked_by_me"], false);
}

#[tokio::test]
async fn comments_enforce_profile_gate_moderation_rate_limit_kill_switch_and_delete() {
    let (app, token, user_id, repo) = build_test_app();
    let comments_uri = "/api/v1/daily-topics/topic-1/entries/entry-1/comments";

    // Posting comment before completing profile -> 403 PROFILE_INCOMPLETE
    let (status, body) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "支持！"})),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "PROFILE_INCOMPLETE");

    // Complete profile
    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "蓉城铁粉", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Content moderation rejection
    let (status, body) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "包含违禁词的内容" })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "CONTENT_REJECTED");

    // Valid comment created
    let (status, created) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "这笔转会太关键了！" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(created["comments_count"], 1);
    assert_eq!(created["comment"]["is_mine"], true);
    assert_eq!(created["comment"]["author"]["display_name"], "蓉城铁粉");
    assert!(created["comment"]["author"].get("user_id").is_none());
    let first_id = created["comment"]["id"].as_str().unwrap().to_string();

    // Reply comment
    let (status, reply_created) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "确实关键", "reply_to_comment_id": first_id })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(reply_created["comments_count"], 2);
    assert_eq!(
        reply_created["comment"]["reply_to"]["author_display_name"],
        "蓉城铁粉"
    );

    // Guest read with limit=1 pagination
    let (status, page1) = send(app.clone(), "GET", &format!("{comments_uri}?limit=1"), None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page1["total_count"], 2);
    assert_eq!(page1["has_more"], true);
    assert_eq!(page1["items"][0]["is_mine"], false);
    assert!(page1["items"][0]["author"].get("user_id").is_none());
    let cursor = page1["next_cursor"].as_str().unwrap();
    let (status, page2) = send(
        app.clone(),
        "GET",
        &format!("{comments_uri}?limit=1&cursor={cursor}"),
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page2["has_more"], false);
    assert_eq!(page2["items"].as_array().unwrap().len(), 1);

    // Rate limit -> 429 RATE_LIMITED
    repo.rate_limited_users.lock().unwrap().push(user_id);
    let (status, body) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "太快了" })),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body["code"], "RATE_LIMITED");
    repo.rate_limited_users.lock().unwrap().clear();

    // Delete own comment
    let (status, del) = send(
        app.clone(),
        "DELETE",
        &format!("{comments_uri}/{first_id}"),
        Some(&token),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(del["comments_count"], 1);

    // Kill switch -> 403 COMMENTS_DISABLED & empty list
    *repo.comments_enabled.lock().unwrap() = false;
    let (status, body) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "关停后测试" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "COMMENTS_DISABLED");
    let (status, list_disabled) = send(app, "GET", comments_uri, None, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(list_disabled["comments_enabled"], false);
    assert_eq!(list_disabled["items"], json!([]));
}

#[tokio::test]
async fn comment_likes_require_login_and_replies_count_tracks_child_replies() {
    let (app, token, _, _) = build_test_app();
    let comments_uri = "/api/v1/daily-topics/topic-1/entries/entry-1/comments";

    let (status, _) = send(
        app.clone(),
        "PUT",
        "/api/v1/auth/me/profile",
        Some(&token),
        Some(json!({ "display_name": "成都球迷", "avatar_data_url": valid_png_data_url() })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, created) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "主帅战术安排很明确" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let comment_id = created["comment"]["id"].as_str().unwrap().to_string();
    let like_uri = format!("{comments_uri}/{comment_id}/like");

    // Guest cannot like comment
    assert_eq!(
        send(app.clone(), "PUT", &like_uri, None, None).await.0,
        StatusCode::UNAUTHORIZED
    );

    // Reply increments parent replies_count
    let (status, _) = send(
        app.clone(),
        "POST",
        comments_uri,
        Some(&token),
        Some(json!({ "content": "确实很清晰", "reply_to_comment_id": comment_id })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Like and idempotent re-like
    let (status, liked) = send(app.clone(), "PUT", &like_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(liked["comment_id"], comment_id);
    assert_eq!(liked["likes_count"], 1);
    assert_eq!(liked["liked_by_me"], true);

    let (status, liked_again) = send(app.clone(), "PUT", &like_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(liked_again["likes_count"], 1);
    assert_eq!(liked_again["liked_by_me"], true);

    // List reflects likes_count, liked_by_me, and replies_count
    let (status, page) = send(app.clone(), "GET", comments_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    let root = page["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == comment_id)
        .unwrap();
    assert_eq!(root["likes_count"], 1);
    assert_eq!(root["liked_by_me"], true);
    assert_eq!(root["replies_count"], 1);

    // Unlike
    let (status, unliked) = send(app, "DELETE", &like_uri, Some(&token), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(unliked["likes_count"], 0);
    assert_eq!(unliked["liked_by_me"], false);
}
