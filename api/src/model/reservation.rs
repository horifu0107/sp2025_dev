use chrono::{DateTime,Local};
use kernel::model::{
    reservation::{Reservation, ReservationSpace},
    id::{SpaceId, ReservationId, UserId},

};
use garde::Validate;
use serde::{Deserialize, Serialize};
use derive_new::new;


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationsResponse {
    pub items: Vec<ReservationResponse>,
}

impl From<Vec<Reservation>> for ReservationsResponse {
    fn from(value: Vec<Reservation>) -> Self {
        Self {
            items: value.into_iter().map(ReservationResponse::from).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateReservationRequest {
    #[garde(skip)]
    pub reservation_start_time: DateTime<Local>,
    #[garde(skip)] 
    pub reservation_end_time: DateTime<Local>,
}

// 蔵書データの更新用の型を追加する
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReservationRequest {
    #[garde(skip)]
    pub reservation_start_time: DateTime<Local>,
    #[garde(skip)] 
    pub reservation_end_time: DateTime<Local>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationResponse {
    pub reservation_id: ReservationId,
    pub reserved_by: UserId,
    pub user_name: String,
    pub email: String,
    pub reminder_is_already: bool,
    pub reserved_at: DateTime<Local>,
    pub reminder_at: DateTime<Local>,
    pub returned_at: Option<DateTime<Local>>,
    pub reservation_start_time:DateTime<Local>,
    pub reservation_end_time:DateTime<Local>,
    pub space: ReservationSpaceResponse,
}

impl From<Reservation> for ReservationResponse {
    fn from(value: Reservation) -> Self {
        let Reservation {
            reservation_id,
            reserved_by,
            user_name,
            email,
            reminder_is_already,
            reserved_at,
            reminder_at,
            returned_at,
            reservation_start_time,
            reservation_end_time,
            space,
        } = value;
        Self {
            reservation_id,
            reserved_by,
            user_name,
            email,
            reminder_is_already,
            reserved_at,
            reminder_at,
            returned_at,
            reservation_start_time,
            reservation_end_time,
            space: space.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReservationSpaceResponse {
    pub space_id: SpaceId,
    pub space_name: String,
    pub is_active: bool,
    pub capacity: i32,
    pub equipment: String,
    pub address:String,
}

impl From<ReservationSpace> for ReservationSpaceResponse {
    fn from(value: ReservationSpace) -> Self {
        let ReservationSpace {
            space_id,
            space_name,
            is_active,
            capacity,
            equipment,
            address,
        } = value;
        Self {
            space_id: space_id,
            space_name,
            is_active,
            capacity,
            equipment,
            address,
        }
    }
}