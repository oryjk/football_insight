use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};

use crate::reflux_subscription::ports::email_sender::EmailSender;

#[derive(Clone)]
pub struct SmtpEmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
}

impl SmtpEmailSender {
    pub fn new(config: SmtpEmailSenderConfig) -> anyhow::Result<Self> {
        let from = config.from.parse::<Mailbox>()?;
        let credentials = Credentials::new(config.username, config.password);
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.host)?
            .port(config.port)
            .credentials(credentials)
            .build();

        Ok(Self { mailer, from })
    }
}

#[derive(Clone)]
pub struct SmtpEmailSenderConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
}

#[async_trait::async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send_html(&self, to: &str, subject: &str, body_html: &str) -> anyhow::Result<()> {
        let message = Message::builder()
            .from(self.from.clone())
            .to(to.parse::<Mailbox>()?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(body_html.to_string())?;

        self.mailer.send(message).await?;
        Ok(())
    }
}
