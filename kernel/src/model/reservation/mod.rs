use crate::model::id::{SpaceId, ReservationId, UserId};
use chrono::{DateTime, Local};

pub mod event;

#[derive(Debug)]
pub struct Reservation {
    pub reservation_id: ReservationId,
    pub reserved_by: UserId,
    pub user_name:String,
    pub email:String,
    pub reminder_is_already: bool,
    pub reserved_at: DateTime<Local>,
    pub reminder_at:DateTime<Local>,
    pub returned_at: Option<DateTime<Local>>,
    pub reservation_start_time: DateTime<Local>,
    pub reservation_end_time: DateTime<Local>,
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