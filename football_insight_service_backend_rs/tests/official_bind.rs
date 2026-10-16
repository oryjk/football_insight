use chrono::{Duration, TimeZone, Utc};
use football_insight_service_backend_rs::auth::domain::official_bind::{
    BindAttemptWindow, can_issue_bind_code, encode_bind_code, hash_bind_code, is_bind_code_live,
    normalize_bind_code,
};

#[test]
fn official_bind_code_format_normalization_hash_and_boundaries() {
    assert_eq!(normalize_bind_code(" a b c 2 3 4 "), Some("ABC234".into()));
    for invalid in [
        "",
        "ABC23",
        "ABC2345",
        "ABCI23",
        "ABCO23",
        "ABC023",
        "ABC123",
        "绑定ABC234",
        "ＡBC234",
    ] {
        assert!(normalize_bind_code(invalid).is_none());
    }
    assert_eq!(
        hash_bind_code("ABC234"),
        "8c640c4e71f90160b2b3615af86739e6b15ddc877ae79e18aada753565f756c4"
    );
    assert_eq!(hash_bind_code(" abc234 "), hash_bind_code("ABC234"));
    assert_eq!(encode_bind_code([0, 1, 2, 29, 30, 31]), "ABC789");
    assert_eq!(encode_bind_code([32, 33, 34, 61, 62, 63]), "ABC789");
    for _ in 0..256 {
        let code = SecureOfficialBindCodeGenerator.generate().unwrap();
        assert_eq!(code.len(), 6);
        assert_eq!(normalize_bind_code(&code).as_ref(), Some(&code));
    }
    let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
    assert!(is_bind_code_live(None, now + Duration::seconds(600), now));
    assert!(!is_bind_code_live(None, now, now));
    assert!(!is_bind_code_live(None, now - Duration::seconds(1), now));
    assert!(!is_bind_code_live(
        Some(now),
        now + Duration::seconds(600),
        now
    ));
    assert!(can_issue_bind_code(4));
    assert!(!can_issue_bind_code(5));
    assert!(!can_issue_bind_code(6));
}

#[test]
fn official_bind_failures_and_inflight_attempts_are_bounded_and_expire() {
    let now = Utc.with_ymd_and_hms(2026, 10, 16, 12, 0, 0).unwrap();
    let mut window = BindAttemptWindow::new(now);
    for _ in 0..5 {
        assert!(window.reserve(now));
    }
    assert!(!window.reserve(now));
    for _ in 0..5 {
        window.finish(false);
    }
    assert!(!window.reserve(now + Duration::seconds(599)));
    assert!(window.reserve(now + Duration::seconds(600)));
    window.finish(true);
    assert!(window.reserve(now + Duration::seconds(600)));
}

use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::DateTime;
use football_insight_service_backend_rs::auth::{
    adapters::{
        security::{
            jwt_token_port::JwtTokenPort,
            official_bind_code_generator::SecureOfficialBindCodeGenerator,
        },
        web::official_bind_handlers::official_bind_code_routes,
    },
    application::official_bind::{
        CompleteOfficialBindUseCase, CreateBindCodeError, CreateOfficialBindCodeUseCase,
    },
    domain::official_bind::*,
    ports::{
        official_bind_code_generator::OfficialBindCodeGenerator,
        official_bind_repository::OfficialBindRepository, token_port::TokenPort,
    },
};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use uuid::Uuid;

#[derive(Default)]
struct Codes {
    records: Vec<(String, DateTime<Utc>, bool)>,
    openid: Option<String>,
    inactive: bool,
    fail: bool,
    collisions: usize,
    calls: usize,
}
#[derive(Default)]
struct BindFake(Mutex<Codes>);
#[async_trait]
impl OfficialBindRepository for BindFake {
    async fn issue(
        &self,
        _: Uuid,
        hash: &str,
        _: DateTime<Utc>,
        expires: DateTime<Utc>,
    ) -> anyhow::Result<IssueBindCodeOutcome> {
        let mut s = self.0.lock().unwrap();
        s.calls += 1;
        if s.fail {
            anyhow::bail!("unavailable");
        }
        if s.inactive {
            return Ok(IssueBindCodeOutcome::Inactive);
        }
        if s.collisions > 0 {
            s.collisions -= 1;
            return Ok(IssueBindCodeOutcome::Collision);
        }
        if !can_issue_bind_code(s.records.len() as i64) {
            return Ok(IssueBindCodeOutcome::RateLimited);
        }
        for r in &mut s.records {
            r.2 = true;
        }
        s.records.push((hash.into(), expires, false));
        Ok(IssueBindCodeOutcome::Created)
    }
    async fn consume(&self, hash: &str, openid: &str) -> anyhow::Result<ConsumeBindCodeOutcome> {
        let mut s = self.0.lock().unwrap();
        s.calls += 1;
        if s.fail {
            anyhow::bail!("unavailable");
        }
        if s.inactive {
            return Ok(ConsumeBindCodeOutcome::Invalid);
        }
        let Some(row) = s
            .records
            .iter_mut()
            .find(|r| r.0 == hash && is_bind_code_live(r.2.then(Utc::now), r.1, Utc::now()))
        else {
            return Ok(ConsumeBindCodeOutcome::Invalid);
        };
        row.2 = true;
        if s.openid.as_deref() == Some(openid) {
            return Ok(ConsumeBindCodeOutcome::AlreadyBound);
        }
        s.openid = Some(openid.into());
        Ok(ConsumeBindCodeOutcome::Success)
    }
}
fn tokens() -> Arc<JwtTokenPort> {
    Arc::new(JwtTokenPort::new("official-bind-test-secret".into()))
}
fn token(t: &JwtTokenPort) -> String {
    t.issue_token(Uuid::new_v4(), "fixture", Utc::now() + Duration::hours(1))
        .unwrap()
}
#[tokio::test]
async fn official_bind_entropy_failure_never_issues_a_code() {
    use football_insight_service_backend_rs::auth::ports::official_bind_code_generator::{
        OfficialBindCodeGenerator, SecureRandomUnavailable,
    };
    struct Unavailable;
    impl OfficialBindCodeGenerator for Unavailable {
        fn generate(&self) -> Result<String, SecureRandomUnavailable> {
            Err(SecureRandomUnavailable)
        }
    }
    let repository = Arc::new(BindFake::default());
    let tokens = tokens();
    let create = CreateOfficialBindCodeUseCase::new(
        repository.clone(),
        tokens.clone(),
        Arc::new(Unavailable),
    );
    assert!(matches!(
        create.execute(&token(&tokens)).await,
        Err(CreateBindCodeError::Unavailable)
    ));
    assert!(repository.0.lock().unwrap().records.is_empty());
    assert_eq!(repository.0.lock().unwrap().calls, 0);
}
#[tokio::test]
async fn official_bind_http_requires_jwt_no_store_rate_limit_and_does_not_expose_user() {
    let repository = Arc::new(BindFake::default());
    let tokens = tokens();
    let app = official_bind_code_routes(Arc::new(CreateOfficialBindCodeUseCase::new(
        repository.clone(),
        tokens.clone(),
        Arc::new(SecureOfficialBindCodeGenerator),
    )));
    for auth in [None, Some("invalid")] {
        let mut req = Request::post("/api/v1/auth/wechat-official-bind-code");
        if let Some(auth) = auth {
            req = req.header("Authorization", format!("Bearer {auth}"));
        }
        let response = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["cache-control"], "no-store");
    }
    assert_eq!(repository.0.lock().unwrap().calls, 0);
    let token = token(&tokens);
    for n in 0..6 {
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/v1/auth/wechat-official-bind-code")
                    .header("Authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()["cache-control"], "no-store");
        assert_eq!(
            response.status(),
            if n < 5 {
                StatusCode::OK
            } else {
                StatusCode::TOO_MANY_REQUESTS
            }
        );
        if n < 5 {
            let json: serde_json::Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap())
                    .unwrap();
            assert_eq!(json.as_object().unwrap().len(), 2);
            assert_eq!(json["expires_in_seconds"], 600);
            assert!(normalize_bind_code(json["code"].as_str().unwrap()).is_some());
        }
    }
}
#[tokio::test]
async fn official_bind_reissue_revokes_old_code_and_repeat_or_concurrent_use_has_one_success() {
    let repository = Arc::new(BindFake::default());
    let tokens = tokens();
    let token = token(&tokens);
    let create = CreateOfficialBindCodeUseCase::new(
        repository.clone(),
        tokens,
        Arc::new(SecureOfficialBindCodeGenerator),
    );
    let first = create.execute(&token).await.unwrap();
    let second = create.execute(&token).await.unwrap();
    let bind = CompleteOfficialBindUseCase::new(repository.clone());
    assert_eq!(
        bind.execute("official-fixture", &first.code).await,
        OfficialBindDecision::Invalid
    );
    let lower = second.code.to_lowercase();
    let (one, two) = tokio::join!(
        bind.execute("official-fixture", &lower),
        bind.execute("another-fixture", &second.code)
    );
    assert_eq!(
        [one, two]
            .iter()
            .filter(|d| **d == OfficialBindDecision::Success)
            .count(),
        1
    );
    assert_eq!(
        [one, two]
            .iter()
            .filter(|d| **d == OfficialBindDecision::Invalid)
            .count(),
        1
    );
    assert_eq!(
        repository
            .0
            .lock()
            .unwrap()
            .records
            .iter()
            .filter(|r| r.2)
            .count(),
        2
    );
}
#[tokio::test]
async fn official_bind_same_openid_consumes_fresh_code_and_different_openid_overwrites() {
    let repository = Arc::new(BindFake::default());
    let tokens = tokens();
    let token = token(&tokens);
    let create = CreateOfficialBindCodeUseCase::new(
        repository.clone(),
        tokens,
        Arc::new(SecureOfficialBindCodeGenerator),
    );
    let bind = CompleteOfficialBindUseCase::new(repository.clone());
    for (openid, expected) in [
        ("first", OfficialBindDecision::Success),
        ("first", OfficialBindDecision::AlreadyBound),
        ("second", OfficialBindDecision::Success),
    ] {
        let code = create.execute(&token).await.unwrap().code;
        assert_eq!(bind.execute(openid, &code).await, expected);
        assert_eq!(
            bind.execute(openid, &code).await,
            OfficialBindDecision::Invalid
        );
    }
    assert_eq!(
        repository.0.lock().unwrap().openid.as_deref(),
        Some("second")
    );
}
#[tokio::test]
async fn official_bind_failed_validation_and_repository_errors_never_write_and_throttle() {
    for scenario in [
        "invalid",
        "expired",
        "used",
        "inactive",
        "error",
        "empty_openid",
    ] {
        let repository = Arc::new(BindFake::default());
        {
            let mut s = repository.0.lock().unwrap();
            s.records.push((
                hash_bind_code("ABC234"),
                Utc::now() + Duration::minutes(if scenario == "expired" { -1 } else { 10 }),
                scenario == "used",
            ));
            s.inactive = scenario == "inactive";
            s.fail = scenario == "error";
        }
        let before = repository.0.lock().unwrap().records.clone();
        let bind = CompleteOfficialBindUseCase::new(repository.clone());
        let decision = bind
            .execute(
                if scenario == "empty_openid" {
                    ""
                } else {
                    "fixture"
                },
                if scenario == "invalid" {
                    "BAD000"
                } else {
                    "ABC234"
                },
            )
            .await;
        assert_eq!(
            decision,
            if scenario == "error" {
                OfficialBindDecision::Error
            } else {
                OfficialBindDecision::Invalid
            }
        );
        let s = repository.0.lock().unwrap();
        assert_eq!(s.records, before);
        assert!(s.openid.is_none());
    }
    let repository = Arc::new(BindFake::default());
    let bind = CompleteOfficialBindUseCase::new(repository.clone());
    assert_eq!(
        bind.execute("fixture", "").await,
        OfficialBindDecision::MissingCode
    );
    for _ in 0..5 {
        assert_eq!(
            bind.execute("fixture", "ABC234").await,
            OfficialBindDecision::Invalid
        );
    }
    let calls = repository.0.lock().unwrap().calls;
    assert_eq!(
        bind.execute("fixture", "ABC234").await,
        OfficialBindDecision::RateLimited
    );
    assert_eq!(repository.0.lock().unwrap().calls, calls);
}
#[tokio::test]
async fn official_bind_creation_retries_collisions_and_rejects_inactive() {
    let repository = Arc::new(BindFake::default());
    let tokens = tokens();
    let token = token(&tokens);
    let create = CreateOfficialBindCodeUseCase::new(
        repository.clone(),
        tokens,
        Arc::new(SecureOfficialBindCodeGenerator),
    );
    repository.0.lock().unwrap().collisions = 3;
    assert!(create.execute(&token).await.is_ok());
    assert_eq!(repository.0.lock().unwrap().calls, 4);
    repository.0.lock().unwrap().inactive = true;
    assert!(matches!(
        create.execute(&token).await,
        Err(CreateBindCodeError::Unauthorized)
    ));
    repository.0.lock().unwrap().inactive = false;
    repository.0.lock().unwrap().collisions = 99;
    assert!(matches!(
        create.execute(&token).await,
        Err(CreateBindCodeError::Unavailable)
    ));
    assert_eq!(repository.0.lock().unwrap().records.len(), 1);
}
