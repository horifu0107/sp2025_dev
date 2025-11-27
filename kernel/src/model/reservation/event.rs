use crate::model::id::{SpaceId, ReservationId, UserId};
use chrono::{DateTime, Utc};
use derive_new::new;

#[derive(new)]
pub struct CreateReservation {
    pub space_id: SpaceId,
    pub reserved_by: UserId,
    pub reserved_at: DateTime<Utc>,
    pub reservation_start_time: DateTime<Utc>,
    pub reservation_end_time: DateTime<Utc>,
    pub reminder_at: DateTime<Utc>,
    pub reminder_is_already: bool,
}

#[derive(new)]
pub struct UpdateReturned {
    pub reservation_id: ReservationId,
    pub space_id: SpaceId,
    pub returned_by: UserId,
    pub is_cancel: bool,
    pub returned_at: DateTime<Utc>,
    pub reservation_start_time: DateTime<Utc>,
    pub reservation_end_time: DateTime<Utc>,
    pub reminder_at: DateTime<Utc>,
}