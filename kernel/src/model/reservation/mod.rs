use crate::model::id::{SpaceId, ReservationId, UserId};
use chrono::{DateTime, Utc};

pub mod event;

#[derive(Debug)]
pub struct Reservation {
    pub reservation_id: ReservationId,
    pub reserved_by: UserId,
    pub reminder_is_already: bool,
    pub reserved_at: DateTime<Utc>,
    pub reminder_at:DateTime<Utc>,
    pub returned_at: Option<DateTime<Utc>>,
    pub reservation_start_time: DateTime<Utc>,
    pub reservation_end_time: DateTime<Utc>,
    pub space: ReservationSpace,
}

#[derive(Debug)]
pub struct ReservationSpace {
    pub space_id: SpaceId,
    pub space_name: String,
    pub is_active: bool,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}