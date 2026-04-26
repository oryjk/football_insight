pub struct LogoutUseCase;

impl LogoutUseCase {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
