use super::domain::*;
pub fn calculate(
    matches: &[RecapMatch],
    selected: &[i64],
    home_km: Option<f64>,
) -> Result<Summary, RecapError> {
    use std::collections::{HashMap, HashSet};
    if home_km.is_some_and(|km| !km.is_finite() || !(0.0..=500.0).contains(&km)) {
        return Err(RecapError::InvalidDistance);
    }
    let by_id: HashMap<_, _> = matches.iter().map(|m| (m.match_id, m)).collect();
    let selected: HashSet<_> = selected.iter().copied().collect();
    let mut cities = HashSet::new();
    let mut home_matches = 0;
    let mut away_km = 0;
    for id in &selected {
        let m = by_id.get(id).ok_or(RecapError::InvalidMatch)?;
        if m.is_home {
            home_matches += 1;
            cities.insert(CHENGDU.name);
        } else {
            let city = city_for_team(&m.opponent).ok_or(RecapError::UnknownCity)?;
            cities.insert(city.name);
            away_km += (great_circle_km(CHENGDU, city) * 2.0).round() as u32;
        }
    }
    if home_matches > 0 && home_km.is_none() {
        return Err(RecapError::MissingDistance);
    }
    let home_total = home_km.map(|km| (km * 2.0 * home_matches as f64).round() as u32);
    Ok(Summary {
        away_km,
        home_km: home_total,
        total_km: away_km + home_total.unwrap_or(0),
        matches: selected.len(),
        home_matches,
        away_matches: selected.len() - home_matches,
        cities: cities.len(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(id: i64, home: bool, opponent: &str) -> RecapMatch {
        RecapMatch {
            match_id: id,
            match_date: "2026-10-01".into(),
            opponent: opponent.into(),
            is_home: home,
        }
    }
    #[test]
    fn validates_distance_bounds_and_missing_home_distance() {
        for distance in [-1.0, 500.01, f64::NAN, f64::INFINITY] {
            assert_eq!(
                calculate(&[], &[], Some(distance)),
                Err(RecapError::InvalidDistance)
            );
        }
        assert_eq!(
            calculate(&[fixture(1, true, "上海申花")], &[1], None),
            Err(RecapError::MissingDistance)
        );
        for distance in [0.0, 500.0] {
            assert_eq!(
                calculate(&[fixture(1, true, "上海申花")], &[1], Some(distance))
                    .unwrap()
                    .home_km,
                Some((distance * 2.0) as u32)
            );
        }
    }
    #[test]
    fn duplicates_and_same_city_teams_count_once_per_match_and_city() {
        let matches = [
            fixture(1, true, "北京国安"),
            fixture(2, false, "上海申花"),
            fixture(3, false, "上海海港"),
        ];
        let result = calculate(&matches, &[1, 1, 2, 2, 3], Some(12.25)).unwrap();
        assert_eq!(result.matches, 3);
        assert_eq!(result.home_km, Some(25));
        assert_eq!(result.cities, 2);
        assert_eq!(
            result.away_km,
            calculate(&matches, &[2], None).unwrap().away_km * 2
        );
        assert_eq!(result.total_km, result.away_km + 25);
    }
    #[test]
    fn unknown_or_stale_selection_is_rejected() {
        assert_eq!(calculate(&[], &[99], None), Err(RecapError::InvalidMatch));
        assert_eq!(
            calculate(&[fixture(1, false, "未知球队")], &[1], None),
            Err(RecapError::UnknownCity)
        );
    }
    #[test]
    fn no_away_and_no_home_are_explicit() {
        let result = calculate(&[], &[], None).unwrap();
        assert_eq!(result.away_km, 0);
        assert_eq!(result.home_km, None);
        assert_eq!(result.cities, 0);
    }
    #[test]
    fn beijing_round_trip_is_about_3000km_and_yunnan_uses_yuxi() {
        let beijing = calculate(&[fixture(1, false, "北京国安")], &[1], None).unwrap();
        assert!((2900..3100).contains(&beijing.away_km));
        let yuxi = calculate(&[fixture(1, false, "云南玉昆")], &[1], None).unwrap();
        assert!((1400..1500).contains(&yuxi.away_km));
    }
}

use super::ports::{RecapCodePort, RecapRepository, Snapshot};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
#[derive(Debug, Serialize)]
pub struct RecapView {
    pub season: i32,
    pub matches: Vec<RecapMatch>,
    pub selected_match_ids: Vec<i64>,
    pub home_one_way_km: Option<f64>,
    pub summary: Summary,
}
pub struct RecapService {
    repository: Arc<dyn RecapRepository>,
    code: Arc<dyn RecapCodePort>,
}
impl RecapService {
    pub fn new(repository: Arc<dyn RecapRepository>, code: Arc<dyn RecapCodePort>) -> Self {
        Self { repository, code }
    }
    fn validate_season(season: i32) -> Result<(), RecapError> {
        use chrono::Datelike;
        if !(2026..=chrono::Utc::now().year()).contains(&season) {
            return Err(RecapError::InvalidSeason);
        }
        Ok(())
    }
    fn view(season: i32, snapshot: Snapshot) -> Result<RecapView, RecapError> {
        // Schedule reconciliation may retire earlier selections. They stay stored but never contribute.
        let selected: Vec<_> = snapshot
            .selected
            .into_iter()
            .filter(|id| snapshot.matches.iter().any(|m| m.match_id == *id))
            .collect();
        let summary = calculate(&snapshot.matches, &selected, snapshot.home_one_way_km)?;
        Ok(RecapView {
            season,
            matches: snapshot.matches,
            selected_match_ids: selected,
            home_one_way_km: snapshot.home_one_way_km,
            summary,
        })
    }
    pub async fn get(&self, user: Uuid, season: i32) -> Result<RecapView, RecapError> {
        Self::validate_season(season)?;
        Self::view(season, self.repository.load(user, season).await?)
    }
    pub async fn save(
        &self,
        user: Uuid,
        season: i32,
        selected: &[i64],
        home_km: Option<f64>,
    ) -> Result<RecapView, RecapError> {
        Self::validate_season(season)?;
        if selected.len() > 60 {
            return Err(RecapError::InvalidMatch);
        }
        calculate(&[], &[], home_km)?;
        let validate = |snapshot: &Snapshot| {
            calculate(
                &snapshot.matches,
                &snapshot.selected,
                snapshot.home_one_way_km,
            )
            .map(|_| ())
        };
        Self::view(
            season,
            self.repository
                .save(user, season, selected, home_km, &validate)
                .await?,
        )
    }
    pub async fn code(&self, season: i32) -> Result<CodeImage, RecapError> {
        Self::validate_season(season)?;
        self.code.png_base64(season).await
    }
}

#[cfg(test)]
mod service_tests {
    use super::*;
    #[test]
    fn reconciled_matches_disappear_from_saved_view() {
        let result = RecapService::view(
            2026,
            Snapshot {
                matches: vec![],
                selected: vec![99],
                home_one_way_km: Some(10.0),
            },
        )
        .unwrap();
        assert!(result.selected_match_ids.is_empty());
        assert_eq!(result.summary.total_km, 0);
    }
    #[test]
    fn spherical_formula_has_known_one_degree_baseline_and_symmetry() {
        let a = City {
            name: "a",
            latitude: 0.0,
            longitude: 0.0,
        };
        let b = City {
            name: "b",
            latitude: 0.0,
            longitude: 1.0,
        };
        assert!((great_circle_km(a, b) - 111.1949266).abs() < 0.00001);
        assert_eq!(great_circle_km(a, b), great_circle_km(b, a));
        assert_eq!(great_circle_km(a, a), 0.0);
    }
    #[test]
    fn all_sixteen_clubs_are_mapped_and_yuxi_is_not_kunming() {
        for team in [
            "北京国安",
            "青岛海牛",
            "上海申花",
            "天津津门虎",
            "山东泰山",
            "河南队",
            "浙江队",
            "云南玉昆",
            "大连英博",
            "重庆铜梁龙",
            "辽宁铁人",
            "上海海港",
            "成都蓉城",
            "深圳新鵬城",
            "武汉三镇",
            "青岛西海岸",
        ] {
            assert!(city_for_team(team).is_some(), "{team}");
        }
        assert_eq!(city_for_team("云南玉昆").unwrap().name, "玉溪");
        assert!(city_for_team("成都蓉城希拉谷").is_none());
    }
}

#[cfg(test)]
mod transactional_tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicBool, Ordering};
    struct ReconciledRepository {
        committed: AtomicBool,
    }
    #[async_trait]
    impl RecapRepository for ReconciledRepository {
        async fn load(&self, _: Uuid, _: i32) -> Result<Snapshot, RecapError> {
            Ok(Snapshot {
                matches: vec![RecapMatch {
                    match_id: 1,
                    match_date: "2026-01-01".into(),
                    opponent: "北京国安".into(),
                    is_home: false,
                }],
                selected: vec![],
                home_one_way_km: None,
            })
        }
        async fn save(
            &self,
            _: Uuid,
            _: i32,
            selected: &[i64],
            _: Option<f64>,
            validate: &crate::season_recap::ports::ValidateSnapshot<'_>,
        ) -> Result<Snapshot, RecapError> {
            let snapshot = Snapshot {
                matches: vec![RecapMatch {
                    match_id: 1,
                    match_date: "2026-01-01".into(),
                    opponent: "北京国安".into(),
                    is_home: true,
                }],
                selected: selected.to_vec(),
                home_one_way_km: None,
            };
            validate(&snapshot)?;
            self.committed.store(true, Ordering::SeqCst);
            Ok(snapshot)
        }
    }
    struct NoCode;
    #[async_trait]
    impl RecapCodePort for NoCode {
        async fn png_base64(&self, _: i32) -> Result<CodeImage, RecapError> {
            Err(RecapError::Unavailable)
        }
    }
    #[tokio::test]
    async fn reconciled_home_match_without_distance_never_commits() {
        let repo = Arc::new(ReconciledRepository {
            committed: AtomicBool::new(false),
        });
        let service = RecapService::new(repo.clone(), Arc::new(NoCode));
        assert_eq!(
            service
                .save(Uuid::nil(), 2026, &[1], None)
                .await
                .unwrap_err(),
            RecapError::MissingDistance
        );
        assert!(
            !repo.committed.load(Ordering::SeqCst),
            "validation must happen inside locked transaction before commit"
        );
    }
}
