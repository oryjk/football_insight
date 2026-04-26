#[derive(Debug, Clone)]
pub struct RoundReference {
    pub season: i32,
    pub round_number: i32,
    pub finalized_at: Option<String>,
    pub status: String,
    pub total_matches: i32,
    pub completed_matches: i32,
}
