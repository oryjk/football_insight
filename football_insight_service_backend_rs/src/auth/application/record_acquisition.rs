use crate::auth::{
    domain::acquisition::AcquisitionSource, ports::acquisition_repository::AcquisitionRepository,
};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use uuid::Uuid;

const MAX_IN_FLIGHT: usize = 128;
const WRITE_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Clone)]
pub struct RecordAcquisition {
    repository: Arc<dyn AcquisitionRepository>,
    slots: Arc<Semaphore>,
}
impl RecordAcquisition {
    pub fn new(repository: Arc<dyn AcquisitionRepository>) -> Self {
        Self {
            repository,
            slots: Arc::new(Semaphore::new(MAX_IN_FLIGHT)),
        }
    }
    pub fn record(&self, user_id: Uuid, source: Option<AcquisitionSource>) {
        let Some(source) = source else {
            return;
        };
        let Ok(permit) = self.slots.clone().try_acquire_owned() else {
            tracing::warn!("acquisition source capacity exceeded");
            return;
        };
        let repository = self.repository.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let work =
                AssertUnwindSafe(async move { repository.insert_first(user_id, source).await })
                    .catch_unwind();
            match tokio::time::timeout(WRITE_TIMEOUT, work).await {
                Ok(Ok(Ok(()))) => {}
                Ok(Ok(Err(_))) => tracing::warn!("acquisition source write failed"),
                Ok(Err(_)) => tracing::warn!("acquisition source write panicked"),
                Err(_) => tracing::warn!("acquisition source write timed out"),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct SlowRepository {
        started: AtomicUsize,
        mode: usize,
    }
    #[async_trait]
    impl AcquisitionRepository for SlowRepository {
        async fn insert_first(&self, _: Uuid, _: AcquisitionSource) -> anyhow::Result<()> {
            self.started.fetch_add(1, Ordering::SeqCst);
            match self.mode {
                0 => std::future::pending::<()>().await,
                1 => anyhow::bail!("missing table"),
                _ => panic!("fixture panic"),
            }
            Ok(())
        }
    }
    fn source() -> Option<AcquisitionSource> {
        AcquisitionSource::new(Some("douyin"), Some("test1"), Some(1047))
    }
    #[tokio::test]
    async fn acquisition_write_is_detached_bounded_times_out_and_releases_capacity() {
        let repo = Arc::new(SlowRepository {
            started: AtomicUsize::new(0),
            mode: 0,
        });
        let recorder = RecordAcquisition::new(repo.clone());
        recorder.record(Uuid::nil(), None);
        assert_eq!(recorder.slots.available_permits(), MAX_IN_FLIGHT);
        for _ in 0..(MAX_IN_FLIGHT + 1) {
            recorder.record(Uuid::nil(), source());
        }
        assert_eq!(recorder.slots.available_permits(), 0);
        tokio::time::timeout(Duration::from_secs(2), async {
            while repo.started.load(Ordering::SeqCst) < MAX_IN_FLIGHT {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(repo.started.load(Ordering::SeqCst), MAX_IN_FLIGHT);
        tokio::time::timeout(WRITE_TIMEOUT + Duration::from_secs(2), async {
            while recorder.slots.available_permits() < MAX_IN_FLIGHT {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(recorder.slots.available_permits(), MAX_IN_FLIGHT);
    }
    #[tokio::test]
    async fn acquisition_repository_error_and_panic_never_escape_to_registration() {
        for mode in [1, 2] {
            let repo = Arc::new(SlowRepository {
                started: AtomicUsize::new(0),
                mode,
            });
            let recorder = RecordAcquisition::new(repo.clone());
            recorder.record(Uuid::nil(), source());
            tokio::task::yield_now().await;
            assert_eq!(repo.started.load(Ordering::SeqCst), 1);
            assert_eq!(recorder.slots.available_permits(), MAX_IN_FLIGHT);
        }
    }
}
