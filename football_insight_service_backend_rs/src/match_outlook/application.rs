use super::{domain::*, ports::OutlookRepository};
use std::sync::Arc;
pub struct OutlookService {
    repository: Arc<dyn OutlookRepository>,
}
impl OutlookService {
    pub fn new(repository: Arc<dyn OutlookRepository>) -> Self {
        Self { repository }
    }
    pub async fn get(&self, id: i64) -> Result<OutlookResponse, ReadError> {
        if id <= 0 {
            return Err(ReadError::Invalid);
        }
        let outlook = self.repository.find(id).await?;
        Ok(OutlookResponse {
            status: if outlook.is_some() {
                "ready"
            } else {
                "unavailable"
            },
            outlook,
        })
    }
    pub async fn record(&self, season: i32) -> Result<Record, ReadError> {
        if !(2014..=2100).contains(&season) {
            return Err(ReadError::Invalid);
        }
        self.repository.record(season).await
    }
}
#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use async_trait::async_trait;
    pub(crate) struct Fake;
    #[async_trait]
    impl OutlookRepository for Fake {
        async fn find(&self, id: i64) -> Result<Option<Outlook>, ReadError> {
            if id == 1 {
                Ok(Some(Outlook {
                    match_id: 1,
                    home_win: 0.5,
                    draw: 0.25,
                    away_win: 0.25,
                    tau: Some(0.7),
                    model_version: "elo-v2".into(),
                    generated_at: chrono::Utc::now(),
                    kickoff_at: chrono::Utc::now(),
                    frozen: true,
                    outcome: Some(0),
                    schedule_changed: false,
                }))
            } else {
                Ok(None)
            }
        }
        async fn record(&self, season: i32) -> Result<Record, ReadError> {
            Ok(Record {
                season,
                matches: 0,
                hits: 0,
                accuracy: None,
                rps_model: None,
                rps_baseline: None,
                rps_improvement: None,
            })
        }
    }
    #[tokio::test]
    async fn stored_values_and_explicit_absence_are_returned_without_model_calculation() {
        let s = OutlookService::new(Arc::new(Fake));
        let value = s.get(1).await.unwrap();
        assert_eq!(value.status, "ready");
        assert_eq!(value.outlook.unwrap().home_win, 0.5);
        let absent = s.get(2).await.unwrap();
        assert_eq!(absent.status, "unavailable");
        assert!(absent.outlook.is_none());
        assert_eq!(s.record(2026).await.unwrap().matches, 0);
    }
    #[tokio::test]
    async fn invalid_parameters_are_rejected() {
        let s = OutlookService::new(Arc::new(Fake));
        for id in [0, -1] {
            assert_eq!(s.get(id).await.unwrap_err(), ReadError::Invalid);
        }
        for year in [2013, 2101] {
            assert!(s.record(year).await.is_err());
        }
    }
}
