use std::sync::Arc;

use thiserror::Error;

use crate::{
    auth::{
        domain::user::{AuthUser, has_valid_avatar_url, validate_custom_nickname},
        ports::{
            auth_repository::AuthRepository,
            avatar_storage_port::{AvatarStoragePort, parse_avatar_data_url},
            token_port::TokenPort,
        },
    },
    content_security::{ContentSecurityError, ContentSecurityPort, ModerationScene},
};

#[derive(Debug, Clone)]
pub struct UpdateUserProfileInput {
    pub access_token: String,
    pub display_name: String,
    pub avatar_data_url: Option<String>,
}

#[derive(Debug, Error)]
pub enum UpdateUserProfileError {
    #[error("请先登录后再设置头像和昵称")]
    Unauthorized,
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    ContentRejected(&'static str),
    #[error("内容安全校验服务暂不可用，请稍后重试")]
    ModerationUnavailable,
    #[error("{0}")]
    Internal(String),
}

pub struct UpdateUserProfileUseCase {
    repository: Arc<dyn AuthRepository>,
    token_port: Arc<dyn TokenPort>,
    avatar_storage: Arc<dyn AvatarStoragePort>,
    content_security: Arc<dyn ContentSecurityPort>,
}

impl UpdateUserProfileUseCase {
    pub fn new(
        repository: Arc<dyn AuthRepository>,
        token_port: Arc<dyn TokenPort>,
        avatar_storage: Arc<dyn AvatarStoragePort>,
        content_security: Arc<dyn ContentSecurityPort>,
    ) -> Self {
        Self {
            repository,
            token_port,
            avatar_storage,
            content_security,
        }
    }

    pub async fn execute(
        &self,
        input: UpdateUserProfileInput,
    ) -> Result<AuthUser, UpdateUserProfileError> {
        let claims = self
            .token_port
            .verify_token(input.access_token.trim())
            .map_err(|_| UpdateUserProfileError::Unauthorized)?;

        let user = self
            .repository
            .get_user_by_id(claims.sub)
            .await
            .map_err(|error| UpdateUserProfileError::Internal(error.to_string()))?
            .ok_or(UpdateUserProfileError::Unauthorized)?;

        let display_name = validate_custom_nickname(&input.display_name)
            .map_err(|error| UpdateUserProfileError::InvalidInput(localize_profile_error(&error.to_string())))?;

        let open_id = self
            .repository
            .get_user_mini_open_id(user.id)
            .await
            .map_err(|error| UpdateUserProfileError::Internal(error.to_string()))?;

        self.content_security
            .check_text(ModerationScene::Profile, open_id.as_deref(), &display_name)
            .await
            .map_err(|error| match error {
                ContentSecurityError::Rejected => {
                    UpdateUserProfileError::ContentRejected("昵称包含敏感或违规内容，请修改后再试")
                }
                ContentSecurityError::Unavailable => UpdateUserProfileError::ModerationUnavailable,
            })?;

        let avatar_url = match input
            .avatar_data_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(data_url) => {
                let parsed = parse_avatar_data_url(data_url, self.avatar_storage.max_bytes())
                    .map_err(|error| {
                        UpdateUserProfileError::InvalidInput(localize_profile_error(
                            &error.to_string(),
                        ))
                    })?;
                self.content_security
                    .check_image(&parsed.bytes, parsed.mime_type)
                    .await
                    .map_err(|error| match error {
                        ContentSecurityError::Rejected => UpdateUserProfileError::ContentRejected(
                            "头像未通过内容安全审核，请更换一张图片",
                        ),
                        ContentSecurityError::Unavailable => {
                            UpdateUserProfileError::ModerationUnavailable
                        }
                    })?;
                self.avatar_storage
                    .store_avatar(user.id, parsed)
                    .await
                    .map_err(|error| UpdateUserProfileError::Internal(error.to_string()))?
            }
            None => {
                let existing = user
                    .avatar_url
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| has_valid_avatar_url(Some(value)));
                existing
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| UpdateUserProfileError::InvalidInput("请先选择头像".to_string()))?
            }
        };

        self.repository
            .update_user_profile(user.id, &display_name, &avatar_url)
            .await
            .map_err(|error| UpdateUserProfileError::Internal(error.to_string()))
    }
}

fn localize_profile_error(message: &str) -> String {
    if message.contains("2 to 16 characters") {
        return "昵称需为 2 至 16 个字符".to_string();
    }
    if message.contains("unsupported characters") {
        return "昵称不能包含换行或特殊符号".to_string();
    }
    if message.contains("phone number") {
        return "昵称不能使用手机号".to_string();
    }
    if message.contains("custom nickname") {
        return "请设置一个自定义昵称".to_string();
    }
    if message.contains("avatar data url is required") {
        return "请先选择头像".to_string();
    }
    if message.contains("too large") {
        return "头像图片过大，请选择较小的图片".to_string();
    }
    if message.contains("avatar") {
        return "头像格式无效，请选择 JPG 或 PNG 图片".to_string();
    }
    message.to_string()
}
