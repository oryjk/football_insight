use serde::Serialize;
#[derive(Clone, Debug, Serialize)]
pub struct RecapMatch {
    pub match_id: i64,
    pub match_date: String,
    pub opponent: String,
    pub is_home: bool,
}
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RecapError {
    #[error("主场单边距离请输入 0–500 公里的数字")]
    InvalidDistance,
    #[error("请填写主场单边距离")]
    MissingDistance,
    #[error("部分比赛已取消、未结束或不属于成都蓉城，请刷新后重选")]
    InvalidMatch,
    #[error("球队城市资料暂未就绪，请稍后再试")]
    UnknownCity,
    #[error("暂不支持该赛季")]
    InvalidSeason,
    #[error("赛季总结暂不可用，请稍后重试")]
    Unavailable,
}
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Summary {
    pub away_km: u32,
    pub home_km: Option<u32>,
    pub total_km: u32,
    pub matches: usize,
    pub home_matches: usize,
    pub away_matches: usize,
    pub cities: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct City {
    pub name: &'static str,
    pub latitude: f64,
    pub longitude: f64,
}
pub const CHENGDU: City = City {
    name: "成都",
    latitude: 30.66667,
    longitude: 104.06667,
};
// GeoNames city centres (CC BY 4.0); IDs and verified club stadium sources in validation report.
pub fn city_for_team(team: &str) -> Option<City> {
    let (name, latitude, longitude) = match team {
        "成都蓉城" => return Some(CHENGDU),
        "北京国安" => ("北京", 39.90750, 116.39723),
        "青岛海牛" | "青岛西海岸" => ("青岛", 36.06488, 120.38042),
        "上海申花" | "上海海港" => ("上海", 31.22222, 121.45806),
        "天津津门虎" => ("天津", 39.14222, 117.17667),
        "山东泰山" => ("济南", 36.66833, 116.99722),
        "河南队" | "河南" => ("郑州", 34.75778, 113.64861),
        "浙江队" | "浙江" => ("杭州", 30.29365, 120.16142),
        "云南玉昆" => ("玉溪", 24.35500, 102.54222),
        "大连英博" => ("大连", 38.91222, 121.60222),
        "重庆铜梁龙" => ("重庆", 29.56026, 106.55771),
        "辽宁铁人" => ("沈阳", 41.79222, 123.43278),
        "深圳新鵬城" | "深圳新鹏城" => ("深圳", 22.54554, 114.06830),
        "武汉三镇" => ("武汉", 30.58333, 114.26667),
        _ => return None,
    };
    Some(City {
        name,
        latitude,
        longitude,
    })
}
pub fn great_circle_km(from: City, to: City) -> f64 {
    let lat = (to.latitude - from.latitude).to_radians();
    let lon = (to.longitude - from.longitude).to_radians();
    let a = (lat / 2.0).sin().powi(2)
        + from.latitude.to_radians().cos()
            * to.latitude.to_radians().cos()
            * (lon / 2.0).sin().powi(2);
    6371.0 * 2.0 * a.clamp(0.0, 1.0).sqrt().asin()
}

#[derive(Clone, Debug, Serialize)]
pub struct CodeImage {
    pub image_base64: String,
    pub mime_type: String,
}
