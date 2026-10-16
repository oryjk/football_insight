use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};

pub const BIND_CODE_TTL_SECONDS: i64 = 600;
pub const BIND_CODE_HOURLY_LIMIT: i64 = 5;
pub const BIND_CODE_RETRIES: usize = 5;
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub fn normalize_bind_code(input: &str) -> Option<String> {
    let value: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    (value.len() == 6 && value.bytes().all(|b| ALPHABET.contains(&b))).then_some(value)
}

pub fn hash_bind_code(input: &str) -> String {
    let normalized: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

pub fn encode_bind_code(bytes: [u8; 6]) -> String {
    // The alphabet has exactly 32 symbols, so masking gives an unbiased distribution.
    bytes
        .into_iter()
        .map(|b| ALPHABET[usize::from(b & 31)] as char)
        .collect()
}

pub fn can_issue_bind_code(count: i64) -> bool {
    count < BIND_CODE_HOURLY_LIMIT
}
pub fn is_bind_code_live(
    used_at: Option<DateTime<Utc>>,
    expires_at: DateTime<Utc>,
    now: DateTime<Utc>,
) -> bool {
    used_at.is_none() && expires_at > now
}

pub struct BindAttemptWindow {
    pub started_at: DateTime<Utc>,
    failures: usize,
    in_flight: usize,
}
impl BindAttemptWindow {
    pub fn new(now: DateTime<Utc>) -> Self {
        Self {
            started_at: now,
            failures: 0,
            in_flight: 0,
        }
    }
    pub fn reserve(&mut self, now: DateTime<Utc>) -> bool {
        if self.started_at + Duration::seconds(BIND_CODE_TTL_SECONDS) <= now {
            *self = Self::new(now);
        }
        if self.failures + self.in_flight >= 5 {
            return false;
        }
        self.in_flight += 1;
        true
    }
    pub fn finish(&mut self, success: bool) {
        self.in_flight = self.in_flight.saturating_sub(1);
        if success {
            self.failures = 0;
        } else {
            self.failures += 1;
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IssueBindCodeOutcome {
    Created,
    RateLimited,
    Inactive,
    Collision,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConsumeBindCodeOutcome {
    Success,
    AlreadyBound,
    Invalid,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OfficialBindDecision {
    Success,
    AlreadyBound,
    Invalid,
    RateLimited,
    MissingCode,
    Error,
}
impl OfficialBindDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::AlreadyBound => "already_bound",
            Self::Invalid => "invalid",
            Self::RateLimited => "rate_limited",
            Self::MissingCode => "missing_code",
            Self::Error => "error",
        }
    }
}
