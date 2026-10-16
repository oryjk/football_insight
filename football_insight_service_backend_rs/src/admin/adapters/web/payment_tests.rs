use crate::{
    admin::{
        adapters::{
            security::{
                jwt_admin_token_port::JwtAdminTokenPort,
                role_based_admin_authorization::RoleBasedAdminAuthorization,
            },
            web::payment_routes::admin_payment_routes,
        },
        application::{
            admin_auth_service::AdminAuthService, admin_payment_service::AdminPaymentService,
        },
        domain::{
            admin_auth::{AdminAccount, AdminSession},
            admin_payment::*,
        },
        ports::{
            admin_auth_repository::AdminAuthRepository,
            admin_payment_repository::AdminPaymentRepository, admin_token_port::AdminTokenPort,
        },
    },
    auth::ports::password_port::PasswordPort,
};
use async_trait::async_trait;
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chrono::{DateTime, Duration, Utc};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
use uuid::Uuid;

struct AuthFake {
    account: AdminAccount,
    session_id: Uuid,
    revoked: bool,
}
#[async_trait]
impl AdminAuthRepository for AuthFake {
    async fn find_by_username(&self, _: &str) -> anyhow::Result<Option<AdminAccount>> {
        panic!("read-only endpoint must not log in")
    }
    async fn create_session(&self, _: AdminSession) -> anyhow::Result<()> {
        panic!("read-only endpoint must not write sessions")
    }
    async fn find_active_account_for_session(
        &self,
        id: Uuid,
        session: Uuid,
        _: DateTime<Utc>,
    ) -> anyhow::Result<Option<AdminAccount>> {
        Ok(
            (!self.revoked && id == self.account.id && session == self.session_id)
                .then(|| self.account.clone()),
        )
    }
    async fn revoke_session(&self, _: Uuid, _: Uuid, _: DateTime<Utc>) -> anyhow::Result<bool> {
        panic!("read-only endpoint must not revoke sessions")
    }
}
struct PasswordFake;
impl PasswordPort for PasswordFake {
    fn hash_password(&self, _: &str) -> anyhow::Result<String> {
        panic!("no password writes")
    }
    fn verify_password(&self, _: &str, _: &str) -> anyhow::Result<bool> {
        panic!("no password checks")
    }
}
#[derive(Default)]
struct PaymentFake {
    calls: AtomicUsize,
}
#[async_trait]
impl AdminPaymentRepository for PaymentFake {
    async fn funnel(&self, _: &PaymentRange) -> Result<Vec<FunnelCount>, AdminPaymentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
    async fn terminals(
        &self,
        _: &PaymentRange,
    ) -> Result<Vec<AttemptTerminalCount>, AdminPaymentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
    async fn errors(&self, _: &PaymentRange) -> Result<Vec<PaymentErrorCount>, AdminPaymentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
    async fn timeline(&self, _: &str) -> Result<Vec<PaymentTimelineEvent>, AdminPaymentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
    async fn stuck(&self, _: i64) -> Result<Vec<StuckPaymentOrder>, AdminPaymentError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![])
    }
}
fn app(role: &str, revoked: bool) -> (axum::Router, String, Arc<PaymentFake>) {
    let now = Utc::now();
    let id = Uuid::new_v4();
    let session = Uuid::new_v4();
    let token_port = Arc::new(JwtAdminTokenPort::new("payment-admin-test".into()));
    // JWT role is deliberately stale; authorization must use the current persisted role.
    let token = token_port
        .issue_token(id, session, "owner", "owner", now + Duration::hours(1))
        .unwrap();
    let auth = Arc::new(AdminAuthService::new(
        Arc::new(AuthFake {
            account: AdminAccount {
                id,
                username: "owner".into(),
                password_hash: "unused".into(),
                display_name: "Owner".into(),
                role: role.into(),
                status: "active".into(),
                created_at: now,
                updated_at: now,
            },
            session_id: session,
            revoked,
        }),
        Arc::new(PasswordFake),
        token_port,
        Arc::new(RoleBasedAdminAuthorization),
        Duration::hours(1),
        Arc::new(Utc::now),
    ));
    let repository = Arc::new(PaymentFake::default());
    (
        admin_payment_routes(Arc::new(AdminPaymentService::new(repository.clone())), auth),
        token,
        repository,
    )
}
const PATHS: [&str; 4] = [
    "/api/v1/admin/payment/funnel?from=2026-10-16T00:00:00Z&to=2026-10-17T00:00:00Z",
    "/api/v1/admin/payment/errors?from=2026-10-16T00:00:00Z&to=2026-10-17T00:00:00Z",
    "/api/v1/admin/payment/orders/test-order/events",
    "/api/v1/admin/payment/stuck-orders",
];
#[tokio::test]
async fn every_route_requires_admin_auth_before_querying_and_disables_cache() {
    let (router, _, repository) = app("owner", false);
    for path in PATHS {
        let response = router
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response.headers()["cache-control"], "no-store");
    }
    assert_eq!(repository.calls.load(Ordering::SeqCst), 0);
}
#[tokio::test]
async fn valid_owner_reads_all_four_surfaces_without_auth_mutations() {
    let (router, token, repository) = app("owner", false);
    for path in PATHS {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["cache-control"], "no-store");
        let bytes = to_bytes(response.into_body(), 65536).await.unwrap();
        assert!(serde_json::from_slice::<serde_json::Value>(&bytes).is_ok());
    }
    assert_eq!(repository.calls.load(Ordering::SeqCst), 5);
}
#[tokio::test]
async fn persisted_unknown_roles_and_revoked_sessions_are_rejected() {
    for (role, revoked, status) in [
        ("viewer", false, StatusCode::FORBIDDEN),
        ("owner", true, StatusCode::UNAUTHORIZED),
    ] {
        let (router, token, repository) = app(role, revoked);
        let response = router
            .oneshot(
                Request::builder()
                    .uri(PATHS[3])
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(repository.calls.load(Ordering::SeqCst), 0);
    }
}
#[tokio::test]
async fn invalid_query_is_bad_request_only_after_auth_and_never_reads_repository() {
    let (router, token, repository) = app("admin", false);
    for path in [
        "/api/v1/admin/payment/funnel?from=bad&to=bad",
        "/api/v1/admin/payment/stuck-orders?older_than_minutes=0",
        "/api/v1/admin/payment/errors?from=2026-10-16T00:00:00Z&to=2026-10-17T00:00:00Z&scenario=other",
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("authorization", format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
    assert_eq!(repository.calls.load(Ordering::SeqCst), 0);
}
