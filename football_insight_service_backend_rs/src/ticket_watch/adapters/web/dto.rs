use serde::Deserialize;
use serde::Serialize;

use crate::ticket_watch::domain::ticket_watch::{
    TicketWatchBlockInterest, TicketWatchCurrentBoardView, TicketWatchCurrentMatchView,
    TicketWatchInventoryEntry, TicketWatchMatchSummary, TicketWatchRegion,
    TicketWatchTrackedInterest,
};

#[derive(Debug, Serialize)]
pub struct TicketWatchMatchSummaryDto {
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

impl From<TicketWatchMatchSummary> for TicketWatchMatchSummaryDto {
    fn from(value: TicketWatchMatchSummary) -> Self {
        Self {
            match_id: value.match_id,
            external_match_id: value.external_match_id,
            round_number: value.round_number,
            sale_start_at: value.sale_start_at,
            match_date: value.match_date,
            match_time: value.match_time,
            kickoff_at: value.kickoff_at,
            home_team_name: value.home_team_name,
            away_team_name: value.away_team_name,
            is_current: value.is_current,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchCurrentMatchDto {
    pub current_match: Option<TicketWatchMatchSummaryDto>,
    pub group_ticket_active: bool,
    pub message: String,
}

impl From<TicketWatchCurrentMatchView> for TicketWatchCurrentMatchDto {
    fn from(value: TicketWatchCurrentMatchView) -> Self {
        Self {
            current_match: value.current_match.map(Into::into),
            group_ticket_active: value.group_ticket_active,
            message: value.message,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchCurrentBoardDto {
    pub current_match: Option<TicketWatchMatchSummaryDto>,
    pub group_ticket_active: bool,
    pub message: String,
    pub inventory: Vec<TicketWatchInventoryEntryDto>,
    pub block_interests: Vec<TicketWatchBlockInterestDto>,
    pub tracked_interests: Vec<TicketWatchTrackedInterestDto>,
}

impl From<TicketWatchCurrentBoardView> for TicketWatchCurrentBoardDto {
    fn from(value: TicketWatchCurrentBoardView) -> Self {
        Self {
            current_match: value.current_match.map(Into::into),
            group_ticket_active: value.group_ticket_active,
            message: value.message,
            inventory: value.inventory.into_iter().map(Into::into).collect(),
            block_interests: value.block_interests.into_iter().map(Into::into).collect(),
            tracked_interests: value.tracked_interests.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchRegionDto {
    pub block_name: String,
    pub price: String,
    pub usable_count: i32,
    pub estate: i32,
}

impl From<TicketWatchRegion> for TicketWatchRegionDto {
    fn from(value: TicketWatchRegion) -> Self {
        Self {
            block_name: value.block_name,
            price: value.price,
            usable_count: value.usable_count,
            estate: value.estate,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchInventoryEntryDto {
    pub block_name: String,
    pub occurrences: i32,
    pub latest_time: String,
}

impl From<TicketWatchInventoryEntry> for TicketWatchInventoryEntryDto {
    fn from(value: TicketWatchInventoryEntry) -> Self {
        Self {
            block_name: value.block_name,
            occurrences: value.occurrences,
            latest_time: value.latest_time,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchBlockInterestDto {
    pub block_name: String,
    pub interested_user_count: i32,
    pub viewer_interested: bool,
}

impl From<TicketWatchBlockInterest> for TicketWatchBlockInterestDto {
    fn from(value: TicketWatchBlockInterest) -> Self {
        Self {
            block_name: value.block_name,
            interested_user_count: value.interested_user_count,
            viewer_interested: value.viewer_interested,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TicketWatchTrackedInterestDto {
    pub block_name: String,
    pub started_at: String,
    pub first_inventory_at: Option<String>,
}

impl From<TicketWatchTrackedInterest> for TicketWatchTrackedInterestDto {
    fn from(value: TicketWatchTrackedInterest) -> Self {
        Self {
            block_name: value.block_name,
            started_at: value.started_at,
            first_inventory_at: value.first_inventory_at,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ToggleTicketWatchBlockInterestRequest {
    pub block_name: String,
}
