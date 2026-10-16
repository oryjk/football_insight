use crate::auth::{
    domain::official_bind::*,
    ports::{
        official_bind_code_generator::OfficialBindCodeGenerator,
        official_bind_repository::OfficialBindRepository, token_port::TokenPort,
    },
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(thiserror::Error, Debug)]
pub enum CreateBindCodeError {
    #[error("not logged in")]
    Unauthorized,
    #[error("获取过于频繁，请稍后再试")]
    RateLimited,
    #[error("暂时无法获取绑定码，请稍后再试")]
    Unavailable,
}

pub struct CreatedOfficialBindCode {
    pub code: String,
    pub expires_in_seconds: i64,
}
pub struct CreateOfficialBindCodeUseCase {
    repository: Arc<dyn OfficialBindRepository>,
    tokens: Arc<dyn TokenPort>,
    generator: Arc<dyn OfficialBindCodeGenerator>,
}
impl CreateOfficialBindCodeUseCase {
    pub fn new(
        repository: Arc<dyn OfficialBindRepository>,
        tokens: Arc<dyn TokenPort>,
        generator: Arc<dyn OfficialBindCodeGenerator>,
    ) -> Self {
        Self {
            repository,
            tokens,
            generator,
        }
    }
    pub async fn execute(
        &self,
        token: &str,
    ) -> Result<CreatedOfficialBindCode, CreateBindCodeError> {
        let claims = self
            .tokens
            .verify_token(token)
            .map_err(|_| CreateBindCodeError::Unauthorized)?;
        for _ in 0..BIND_CODE_RETRIES {
            let code = self
                .generator
                .generate()
                .map_err(|_| CreateBindCodeError::Unavailable)?;
            let now = Utc::now();
            match self
                .repository
                .issue(
                    claims.sub,
                    &hash_bind_code(&code),
                    now,
                    now + Duration::seconds(BIND_CODE_TTL_SECONDS),
                )
                .await
                .map_err(|_| CreateBindCodeError::Unavailable)?
            {
                IssueBindCodeOutcome::Created => {
                    return Ok(CreatedOfficialBindCode {
                        code,
                        expires_in_seconds: BIND_CODE_TTL_SECONDS,
                    });
                }
                IssueBindCodeOutcome::Inactive => return Err(CreateBindCodeError::Unauthorized),
                IssueBindCodeOutcome::RateLimited => return Err(CreateBindCodeError::RateLimited),
                IssueBindCodeOutcome::Collision => continue,
            }
        }
        Err(CreateBindCodeError::Unavailable)
    }
}

pub struct CompleteOfficialBindUseCase {
    repository: Arc<dyn OfficialBindRepository>,
    attempts: Mutex<HashMap<[u8; 32], BindAttemptWindow>>,
}
impl CompleteOfficialBindUseCase {
    pub fn new(repository: Arc<dyn OfficialBindRepository>) -> Self {
        Self {
            repository,
            attempts: Mutex::new(HashMap::new()),
        }
    }
    pub async fn execute(&self, verified_open_id: &str, code: &str) -> OfficialBindDecision {
        if code.trim().is_empty() {
            return OfficialBindDecision::MissingCode;
        }
        if verified_open_id.trim().is_empty()
            || verified_open_id.trim() != verified_open_id
            || verified_open_id.len() > 128
        {
            return OfficialBindDecision::Invalid;
        }
        let key: [u8; 32] = Sha256::digest(verified_open_id.as_bytes()).into();
        let now = Utc::now();
        let started_at = {
            let Ok(mut attempts) = self.attempts.lock() else {
                return OfficialBindDecision::Error;
            };
            attempts.retain(|_, w| w.started_at + Duration::seconds(BIND_CODE_TTL_SECONDS) > now);
            let window = attempts
                .entry(key)
                .or_insert_with(|| BindAttemptWindow::new(now));
            if !window.reserve(now) {
                return OfficialBindDecision::RateLimited;
            }
            window.started_at
        };
        let decision = if let Some(code) = normalize_bind_code(code) {
            match self
                .repository
                .consume(&hash_bind_code(&code), verified_open_id)
                .await
            {
                Ok(ConsumeBindCodeOutcome::Success) => OfficialBindDecision::Success,
                Ok(ConsumeBindCodeOutcome::AlreadyBound) => OfficialBindDecision::AlreadyBound,
                Ok(ConsumeBindCodeOutcome::Invalid) => OfficialBindDecision::Invalid,
                Err(_) => OfficialBindDecision::Error,
            }
        } else {
            OfficialBindDecision::Invalid
        };
        if let Ok(mut attempts) = self.attempts.lock()
            && let Some(window) = attempts
                .get_mut(&key)
                .filter(|w| w.started_at == started_at)
        {
            window.finish(matches!(
                decision,
                OfficialBindDecision::Success | OfficialBindDecision::AlreadyBound
            ));
        }
        decision
    }
}
