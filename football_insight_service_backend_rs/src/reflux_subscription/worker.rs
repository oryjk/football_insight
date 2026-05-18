use std::{sync::Arc, time::Duration};

use sqlx::PgPool;

use crate::{
    config::AppConfig,
    reflux_subscription::{
        adapters::{
            integration::smtp_email_sender::{SmtpEmailSender, SmtpEmailSenderConfig},
            persistence::postgres_reflux_subscription_repository::PostgresRefluxSubscriptionRepository,
        },
        application::{
            process_reflux_notification_jobs::ProcessRefluxNotificationJobsUseCase,
            process_reflux_notifications::ProcessRefluxNotificationsUseCase,
        },
    },
    ticket_watch::adapters::integration::http_ticket_monitor_port::HttpTicketMonitorPort,
};

pub fn spawn_reflux_notification_worker(pool: PgPool, config: &AppConfig) {
    if !config.reflux_notification_worker.enabled {
        tracing::info!("reflux notification worker disabled");
        return;
    }

    let Some(smtp_config) = config.smtp_email.clone() else {
        tracing::warn!("reflux notification worker enabled but FI_SMTP_* is incomplete");
        return;
    };

    let email_sender = match SmtpEmailSender::new(SmtpEmailSenderConfig {
        host: smtp_config.host,
        port: smtp_config.port,
        username: smtp_config.username,
        password: smtp_config.password,
        from: smtp_config.from,
    }) {
        Ok(sender) => Arc::new(sender),
        Err(error) => {
            tracing::error!(error = %error, "failed to initialize reflux smtp email sender");
            return;
        }
    };
    let repository = Arc::new(PostgresRefluxSubscriptionRepository::new(pool));
    let enqueue_use_case = Arc::new(ProcessRefluxNotificationsUseCase::new(
        repository.clone(),
        Arc::new(HttpTicketMonitorPort::new(config.ticket_monitor_base_url.clone())),
    ));
    let send_use_case = Arc::new(ProcessRefluxNotificationJobsUseCase::new(
        repository.clone(),
        email_sender,
    ));
    let poll_seconds = config.reflux_notification_worker.poll_seconds;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(poll_seconds));

        loop {
            interval.tick().await;

            match enqueue_use_case.execute().await {
                Ok(created_count) => {
                    if created_count > 0 {
                        tracing::info!(created_count, "created reflux notification jobs");
                    }
                }
                Err(error) => {
                    tracing::error!(error = %error, "failed to enqueue reflux notification jobs");
                }
            }

            match send_use_case.execute(50).await {
                Ok(sent_count) => {
                    if sent_count > 0 {
                        tracing::info!(sent_count, "processed reflux notification email jobs");
                    }
                }
                Err(error) => {
                    tracing::error!(error = %error, "failed to process reflux notification email jobs");
                }
            }
        }
    });
}
