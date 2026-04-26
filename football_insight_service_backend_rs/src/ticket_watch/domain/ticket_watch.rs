#[derive(Debug, Clone)]
pub struct TicketWatchMatchSummary {
    pub match_id: i64,
    pub external_match_id: String,
    pub round_number: i32,
    pub sale_start_at: Option<String>,
    pub match_date: String,
    pub match_time: String,
    pub kickoff_at: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct TicketWatchCurrentMatchView {
    pub current_match: Option<TicketWatchMatchSummary>,
    pub group_ticket_active: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct TicketWatchCurrentBoardView {
    pub current_match: Option<TicketWatchMatchSummary>,
    pub group_ticket_active: bool,
    pub message: String,
    pub inventory: Vec<TicketWatchInventoryEntry>,
    pub block_interests: Vec<TicketWatchBlockInterest>,
    pub tracked_interests: Vec<TicketWatchTrackedInterest>,
}

#[derive(Debug, Clone)]
pub struct TicketWatchRegion {
    pub block_name: String,
    pub price: String,
    pub usable_count: i32,
    pub estate: i32,
}

#[derive(Debug, Clone)]
pub struct TicketWatchInventoryEntry {
    pub block_name: String,
    pub occurrences: i32,
    pub latest_time: String,
}

#[derive(Debug, Clone)]
pub struct TicketWatchBlockInterest {
    pub block_name: String,
    pub interested_user_count: i32,
    pub viewer_interested: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct TicketWatchTrackedInterest {
    pub block_name: String,
    pub started_at: String,
    pub first_inventory_at: Option<String>,
}
