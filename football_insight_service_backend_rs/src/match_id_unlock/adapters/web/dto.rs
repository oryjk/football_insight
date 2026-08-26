use serde::{Deserialize, Serialize};

use crate::{
    match_id_unlock::domain::match_id_unlock::MatchIdUnlockSource,
    payment::adapters::web::dto::WxPayParamsDto,
};

#[derive(Debug, Deserialize)]
pub struct MatchIdEntitlementQuery {
    pub match_id: i64,
}

#[derive(Debug, Serialize)]
pub struct MatchIdEntitlementResponse {
    pub unlocked: bool,
    pub via: Option<&'static str>,
    pub effective_tier: String,
}

impl From<crate::match_id_unlock::application::get_match_id_entitlement::MatchIdEntitlementView>
    for MatchIdEntitlementResponse
{
    fn from(
        value: crate::match_id_unlock::application::get_match_id_entitlement::MatchIdEntitlementView,
    ) -> Self {
        Self {
            unlocked: value.unlocked,
            via: value.via.map(|via| match via {
                MatchIdUnlockSource::Membership => "membership",
                MatchIdUnlockSource::Purchase => "purchase",
            }),
            effective_tier: value.effective_tier,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateMatchIdOrderRequest {
    pub match_id: i64,
}

#[derive(Debug, Serialize)]
pub struct CreateMatchIdOrderResponse {
    pub order_no: String,
    pub wx_pay_params: WxPayParamsDto,
}
