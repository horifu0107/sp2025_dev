use kernel::model::{
    reservation::{Reservation, ReservationSpace},
    id::{SpaceId, ReservationId, UserId},
};
use sqlx::types::chrono::{DateTime, Utc};

// 貸し出し状態を確認するための型
// 蔵書が存在する場合はこの型にはまるレコードが存在し、
// その蔵書が貸出中の場合は reservation_id および user_id が None ではない値になる
// 蔵書が貸出中でない場合は reservation_id も user_id も None
pub struct ReservationStateRow {
    pub space_id: SpaceId,
    pub reminder_is_already:bool,
    pub reminder_at:DateTime<Utc>,
    pub reservation_id: ReservationId,
    pub user_id:UserId,
}

// 予約中の一覧を取得する際に使う型
pub struct ReservationRow {
    pub reservation_id: ReservationId,
    pub space_id: SpaceId,
    pub user_id: UserId,
    pub reservation_start_time: DateTime<Utc>,
    pub reservation_end_time: DateTime<Utc>,
    pub reserved_at: DateTime<Utc>,
    pub reminder_at: DateTime<Utc>,
    pub reminder_is_already:bool,
    pub space_name: String,
    pub is_active: bool,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}


impl From<ReservationRow> for Reservation {
    fn from(value: ReservationRow) -> Self {
        let ReservationRow {
            reservation_id,
            space_id,
            user_id,
            reservation_start_time,
            reservation_end_time,
            reminder_at,
            reserved_at,
            reminder_is_already,
            space_name,
            is_active,
            capacity,
            equipment,
            address,
        } = value;
        Reservation {
            reservation_id: reservation_id,
            reserved_by: user_id,
            reserved_at,
            reminder_at,
            reminder_is_already,
            // 未返却なので、returned_at は None を入れる
            returned_at: None,
            reservation_start_time,
            reservation_end_time,
            space: ReservationSpace {
                space_id,
                space_name,
                is_active,
                capacity,
                equipment,
                address,
            },
        }
    }
}

// 予約終了済みの予約一覧を取得する際に使う型
pub struct ReturnedReservationRow {
    pub reservation_id: ReservationId,
    pub space_id: SpaceId,
    pub user_id: UserId,
    pub is_cancel: bool,
    pub reminder_is_already: bool,
    pub reserved_at: DateTime<Utc>,
    pub reminder_at: DateTime<Utc>,
    pub returned_at: DateTime<Utc>,
    pub reservation_start_time:DateTime<Utc>,
    pub reservation_end_time:DateTime<Utc>,
    pub space_name: String,
    pub is_active: bool,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}

impl From<ReturnedReservationRow> for Reservation {
    fn from(value: ReturnedReservationRow) -> Self {
        let ReturnedReservationRow {
            reservation_id,
            space_id,
            user_id,
            is_cancel: _,
            reminder_is_already,
            reserved_at,
            reminder_at,
            returned_at,
            reservation_start_time,
            reservation_end_time,
            space_name,
            is_active,
            capacity,
            equipment,
            address,
        } = value;
        Reservation {
            reservation_id: reservation_id,
            reserved_by: user_id,
            reminder_is_already,
            reserved_at,
            reminder_at,
            // 返却済みなので returned_at には日時データが入る
            returned_at: Some(returned_at),
            reservation_start_time,
            reservation_end_time,
            space: ReservationSpace {
                space_id,
                space_name,
                is_active,
                capacity,
                equipment,
                address,
            },
        }
    }
}