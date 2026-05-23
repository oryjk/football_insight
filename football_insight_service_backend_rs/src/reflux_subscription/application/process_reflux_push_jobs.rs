use std::sync::Arc;

use crate::{
    push_notification::ports::{
        device_token_repository::DeviceTokenRepository,
        push_sender::{PushPayload, PushSender},
    },
    reflux_subscription::ports::reflux_subscription_repository::RefluxSubscriptionRepository,
};

pub struct ProcessRefluxPushJobsUseCase {
    reflux_repository: Arc<dyn RefluxSubscriptionRepository>,
    token_repository: Arc<dyn DeviceTokenRepository>,
    push_sender: Arc<dyn PushSender>,
}

impl ProcessRefluxPushJobsUseCase {
    pub fn new(
        reflux_repository: Arc<dyn RefluxSubscriptionRepository>,
        token_repository: Arc<dyn DeviceTokenRepository>,
        push_sender: Arc<dyn PushSender>,
    ) -> Self {
        Self {
            reflux_repository,
            token_repository,
            push_sender,
        }
    }

    pub async fn execute(&self, limit: i64) -> anyhow::Result<usize> {
        let jobs = self
            .reflux_repository
            .list_pending_notification_jobs(limit)
            .await?;
        let mut pushed_count = 0usize;

        for job in &jobs {
            let payload = PushPayload {
                title: job.subject.clone(),
                body: extract_text_from_html(&job.body_html),
                data: serde_json::json!({
                    "kind": "reflux_alert",
                    "job_id": job.id.to_string(),
                }),
            };

            let tokens = self
                .token_repository
                .list_by_user(job.target.user_id)
                .await?;
            if tokens.is_empty() {
                continue;
            }

            let token_strings: Vec<String> =
                tokens.iter().map(|t| t.device_token.clone()).collect();
            match self.push_sender.send_batch(&token_strings, &payload).await {
                Ok(()) => {
                    pushed_count += token_strings.len();
                }
                Err(error) => {
                    tracing::warn!(
                        job_id = %job.id,
                        user_id = %job.target.user_id,
                        error = %error,
                        "failed to send push for reflux notification job"
                    );
                }
            }
        }

        Ok(pushed_count)
    }
}

fn extract_text_from_html(html: &str) -> String {
    let text = html
        .replace("<br>", " ")
        .replace("<br/>", " ")
        .replace("<br />", " ");
    static HTML_TAG_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = HTML_TAG_RE.get_or_init(|| regex::Regex::new(r"<[^>]+>").expect("valid regex"));
    let text = re.replace_all(&text, "").to_string();
    let decoded = text
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'");
    decoded.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::ProcessRefluxPushJobsUseCase;
    use crate::{
        push_notification::{
            domain::device_token::DeviceToken,
            ports::{
                device_token_repository::DeviceTokenRepository,
                push_sender::{PushPayload, PushSender},
            },
        },
        reflux_subscription::{
            domain::subscription::{
                NotificationTarget, RefluxNotificationJob, RefluxSubscriptionPlan,
                UserRefluxSubscription,
            },
            ports::reflux_subscription_repository::{
                CreateNotificationJobInput, CreateRefluxSubscriptionInput,
                RefluxSubscriptionRepository,
            },
        },
    };

    struct FakeRefluxRepository {
        jobs: Mutex<Vec<RefluxNotificationJob>>,
    }

    impl Default for FakeRefluxRepository {
        fn default() -> Self {
            Self {
                jobs: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl RefluxSubscriptionRepository for FakeRefluxRepository {
        async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>> {
            Ok(vec![])
        }
        async fn find_enabled_plan(
            &self,
            _team_code: &str,
            _plan_code: &str,
        ) -> anyhow::Result<Option<RefluxSubscriptionPlan>> {
            Ok(None)
        }
        async fn list_user_active_subscriptions(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<UserRefluxSubscription>> {
            Ok(vec![])
        }
        async fn get_user_email_target(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<NotificationTarget>> {
            Ok(None)
        }
        async fn upsert_user_email_target(
            &self,
            _user_id: Uuid,
            _email: &str,
        ) -> anyhow::Result<NotificationTarget> {
            unimplemented!()
        }
        async fn create_subscription(
            &self,
            _input: CreateRefluxSubscriptionInput,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }
        async fn create_notification_job(
            &self,
            _input: CreateNotificationJobInput,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }
        async fn list_pending_notification_jobs(
            &self,
            _limit: i64,
        ) -> anyhow::Result<Vec<RefluxNotificationJob>> {
            Ok(self.jobs.lock().expect("jobs").clone())
        }
    }

    struct FakeTokenRepository {
        tokens: Mutex<Vec<DeviceToken>>,
    }

    impl Default for FakeTokenRepository {
        fn default() -> Self {
            Self {
                tokens: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl DeviceTokenRepository for FakeTokenRepository {
        async fn upsert(
            &self,
            _user_id: Uuid,
            _device_token: &str,
            _platform: &str,
        ) -> anyhow::Result<()> {
            unimplemented!()
        }
        async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<DeviceToken>> {
            let tokens = self.tokens.lock().expect("tokens");
            Ok(tokens
                .iter()
                .filter(|t| t.user_id == user_id)
                .cloned()
                .collect())
        }
        async fn list_by_users(&self, _user_ids: &[Uuid]) -> anyhow::Result<Vec<DeviceToken>> {
            unimplemented!()
        }
        async fn delete(&self, _device_token: &str) -> anyhow::Result<()> {
            unimplemented!()
        }
    }

    struct FakePushSender {
        sent: Mutex<Vec<String>>,
    }

    impl Default for FakePushSender {
        fn default() -> Self {
            Self {
                sent: Mutex::new(vec![]),
            }
        }
    }

    #[async_trait]
    impl PushSender for FakePushSender {
        async fn send(&self, device_token: &str, _payload: &PushPayload) -> anyhow::Result<()> {
            self.sent
                .lock()
                .expect("sent")
                .push(device_token.to_string());
            Ok(())
        }
    }

    fn make_job(user_id: Uuid) -> RefluxNotificationJob {
        RefluxNotificationJob {
            id: Uuid::new_v4(),
            target: NotificationTarget {
                id: Uuid::new_v4(),
                user_id,
                channel: "email".to_string(),
                target: "user@example.com".to_string(),
                is_active: true,
            },
            subject: "回流提醒".to_string(),
            body_html: "<p>test body</p>".to_string(),
            attempts: 0,
        }
    }

    fn make_token(id: i64, user_id: Uuid) -> DeviceToken {
        DeviceToken {
            id,
            user_id,
            device_token: format!("token_{}", id),
            platform: "jpush".to_string(),
        }
    }

    #[tokio::test]
    async fn sends_push_to_user_devices_for_pending_jobs() {
        let user_id = Uuid::new_v4();
        let reflux_repo = Arc::new(FakeRefluxRepository {
            jobs: Mutex::new(vec![make_job(user_id)]),
        });
        let token_repo = Arc::new(FakeTokenRepository {
            tokens: Mutex::new(vec![make_token(1, user_id), make_token(2, user_id)]),
        });
        let push_sender = Arc::new(FakePushSender::default());

        let use_case =
            ProcessRefluxPushJobsUseCase::new(reflux_repo, token_repo, push_sender.clone());

        let count = use_case.execute(50).await.expect("execute");
        assert_eq!(count, 2);
        assert_eq!(push_sender.sent.lock().expect("sent").len(), 2);
    }

    #[tokio::test]
    async fn skips_users_without_device_tokens() {
        let user_id = Uuid::new_v4();
        let reflux_repo = Arc::new(FakeRefluxRepository {
            jobs: Mutex::new(vec![make_job(user_id)]),
        });
        let token_repo = Arc::new(FakeTokenRepository::default());
        let push_sender = Arc::new(FakePushSender::default());

        let use_case =
            ProcessRefluxPushJobsUseCase::new(reflux_repo, token_repo, push_sender.clone());

        let count = use_case.execute(50).await.expect("execute");
        assert_eq!(count, 0);
        assert!(push_sender.sent.lock().expect("sent").is_empty());
    }

    #[tokio::test]
    async fn continues_on_push_failure() {
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let reflux_repo = Arc::new(FakeRefluxRepository {
            jobs: Mutex::new(vec![make_job(user_a), make_job(user_b)]),
        });
        let token_repo = Arc::new(FakeTokenRepository {
            tokens: Mutex::new(vec![make_token(1, user_a), make_token(2, user_b)]),
        });

        struct FailingPushSender;
        #[async_trait]
        impl PushSender for FailingPushSender {
            async fn send(
                &self,
                _device_token: &str,
                _payload: &PushPayload,
            ) -> anyhow::Result<()> {
                anyhow::bail!("push failed");
            }
        }

        let use_case =
            ProcessRefluxPushJobsUseCase::new(reflux_repo, token_repo, Arc::new(FailingPushSender));

        let count = use_case.execute(50).await.expect("execute");
        assert_eq!(count, 0);
    }

    #[test]
    fn strips_html_tags_from_body() {
        let result = super::extract_text_from_html("<p>Hello <b>world</b></p>");
        assert_eq!(result, "Hello world");
    }

    #[test]
    fn decodes_html_entities() {
        let result = super::extract_text_from_html("&lt;alert&gt; &amp; &quot;test&quot;");
        assert_eq!(result, "<alert> & \"test\"");
    }
}
