use std::sync::Arc;

use crate::reflux_subscription::ports::{
    email_sender::EmailSender, reflux_subscription_repository::RefluxSubscriptionRepository,
};

const MAX_EMAIL_ATTEMPTS: i32 = 3;

pub struct ProcessRefluxNotificationJobsUseCase {
    repository: Arc<dyn RefluxSubscriptionRepository>,
    email_sender: Arc<dyn EmailSender>,
}

impl ProcessRefluxNotificationJobsUseCase {
    pub fn new(
        repository: Arc<dyn RefluxSubscriptionRepository>,
        email_sender: Arc<dyn EmailSender>,
    ) -> Self {
        Self {
            repository,
            email_sender,
        }
    }

    pub async fn execute(&self, limit: i64) -> anyhow::Result<usize> {
        let jobs = self
            .repository
            .list_pending_notification_jobs(limit)
            .await?;
        let mut sent_count = 0usize;

        for job in jobs {
            let result = self
                .email_sender
                .send_html(&job.target.target, &job.subject, &job.body_html)
                .await;

            match result {
                Ok(()) => {
                    self.repository.mark_notification_job_sent(job.id).await?;
                    sent_count += 1;
                }
                Err(error) => {
                    let attempts = job.attempts + 1;
                    self.repository
                        .mark_notification_job_failed(job.id, attempts, &format!("{error:#}"))
                        .await?;
                }
            }
        }

        Ok(sent_count)
    }
}

pub fn should_retry_email_job(attempts: i32) -> bool {
    attempts < MAX_EMAIL_ATTEMPTS
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use uuid::Uuid;

    use super::{ProcessRefluxNotificationJobsUseCase, should_retry_email_job};
    use crate::reflux_subscription::{
        domain::subscription::{
            NotificationTarget, RefluxNotificationJob, RefluxSubscriptionPlan,
            UserRefluxSubscription,
        },
        ports::{
            email_sender::EmailSender,
            reflux_subscription_repository::{
                CreateNotificationJobInput, CreateRefluxSubscriptionInput,
                RefluxSubscriptionRepository,
            },
        },
    };

    #[derive(Default)]
    struct FakeRepository {
        jobs: Mutex<Vec<RefluxNotificationJob>>,
        sent: Mutex<Vec<Uuid>>,
        failed: Mutex<Vec<(Uuid, i32)>>,
    }

    #[async_trait]
    impl RefluxSubscriptionRepository for FakeRepository {
        async fn list_enabled_plans(&self) -> anyhow::Result<Vec<RefluxSubscriptionPlan>> {
            unreachable!()
        }

        async fn find_enabled_plan(
            &self,
            _team_code: &str,
            _plan_code: &str,
        ) -> anyhow::Result<Option<RefluxSubscriptionPlan>> {
            unreachable!()
        }

        async fn list_user_active_subscriptions(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Vec<UserRefluxSubscription>> {
            unreachable!()
        }

        async fn get_user_email_target(
            &self,
            _user_id: Uuid,
        ) -> anyhow::Result<Option<NotificationTarget>> {
            unreachable!()
        }

        async fn upsert_user_email_target(
            &self,
            _user_id: Uuid,
            _email: &str,
        ) -> anyhow::Result<NotificationTarget> {
            unreachable!()
        }

        async fn create_subscription(
            &self,
            _input: CreateRefluxSubscriptionInput,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn create_notification_job(
            &self,
            _input: CreateNotificationJobInput,
        ) -> anyhow::Result<()> {
            unreachable!()
        }

        async fn list_pending_notification_jobs(
            &self,
            _limit: i64,
        ) -> anyhow::Result<Vec<RefluxNotificationJob>> {
            Ok(self.jobs.lock().expect("jobs").clone())
        }

        async fn mark_notification_job_sent(&self, job_id: Uuid) -> anyhow::Result<()> {
            self.sent.lock().expect("sent").push(job_id);
            Ok(())
        }

        async fn mark_notification_job_failed(
            &self,
            job_id: Uuid,
            attempts: i32,
            _error: &str,
        ) -> anyhow::Result<()> {
            self.failed.lock().expect("failed").push((job_id, attempts));
            Ok(())
        }
    }

    struct FakeEmailSender {
        should_fail: bool,
    }

    #[async_trait]
    impl EmailSender for FakeEmailSender {
        async fn send_html(
            &self,
            _to: &str,
            _subject: &str,
            _body_html: &str,
        ) -> anyhow::Result<()> {
            if self.should_fail {
                anyhow::bail!("smtp failed");
            }
            Ok(())
        }
    }

    fn job(id: Uuid) -> RefluxNotificationJob {
        RefluxNotificationJob {
            id,
            target: NotificationTarget {
                id: Uuid::new_v4(),
                user_id: Uuid::new_v4(),
                channel: "email".to_string(),
                target: "user@example.com".to_string(),
                is_active: true,
            },
            subject: "subject".to_string(),
            body_html: "<p>body</p>".to_string(),
            attempts: 0,
        }
    }

    #[tokio::test]
    async fn execute_marks_jobs_sent_after_successful_email() {
        let job_id = Uuid::new_v4();
        let repository = Arc::new(FakeRepository {
            jobs: Mutex::new(vec![job(job_id)]),
            ..Default::default()
        });
        let use_case = ProcessRefluxNotificationJobsUseCase::new(
            repository.clone(),
            Arc::new(FakeEmailSender { should_fail: false }),
        );

        let count = use_case.execute(20).await.expect("process jobs");

        assert_eq!(count, 1);
        assert_eq!(repository.sent.lock().expect("sent").as_slice(), [job_id]);
    }

    #[tokio::test]
    async fn execute_records_failed_attempts() {
        let job_id = Uuid::new_v4();
        let repository = Arc::new(FakeRepository {
            jobs: Mutex::new(vec![job(job_id)]),
            ..Default::default()
        });
        let use_case = ProcessRefluxNotificationJobsUseCase::new(
            repository.clone(),
            Arc::new(FakeEmailSender { should_fail: true }),
        );

        let count = use_case.execute(20).await.expect("process jobs");

        assert_eq!(count, 0);
        assert_eq!(
            repository.failed.lock().expect("failed").as_slice(),
            [(job_id, 1)]
        );
    }

    #[test]
    fn retries_email_jobs_until_three_attempts() {
        assert!(should_retry_email_job(0));
        assert!(should_retry_email_job(2));
        assert!(!should_retry_email_job(3));
    }
}
