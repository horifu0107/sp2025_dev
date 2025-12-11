use kernel::model::{id::{SpaceId,UserId,ReservationId},
    user::{SpaceOwner,ReservationUser}, 
    space::{Space,Reservation}};

pub struct SpaceRow {
    pub space_id: SpaceId,
    pub space_name: String,
    pub owner_name: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
    pub owned_by:UserId,
}
use chrono::{DateTime, Local};

impl From<SpaceRow> for Space {
    fn from(value: SpaceRow) -> Self {
        let SpaceRow {
            space_id,
            space_name,
            is_active,
            capacity,
            description,
            equipment,
            address,
            owner_name,
            owned_by,
        } = value;
        Space {
            space_id,
            space_name,
            is_active,
            capacity,
            description,
            equipment,
            address,
            owner: SpaceOwner {
                owner_id:owned_by,
                owner_name:owner_name,
            },
            reservation: None, // ★ 追加
        }
    }
}

// From トレイトの実装の代わりに、引数をとる into_space メソッドを定義し実装する
impl SpaceRow {
    pub fn into_space(self, reservation: Option<Reservation>) -> Space {
        let SpaceRow {
            space_id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            owned_by,
            owner_name,
        } = self;
        Space {
            space_id: space_id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            owner: SpaceOwner{
                owner_id:owned_by,
                owner_name:owner_name,
            },
            reservation,
        }
    }
}

// ページネーション用の adapter 内部の型
pub struct PaginatedSpaceRow {
    pub total: i64,
    pub space_id: SpaceId,
}

// 貸し出し情報を格納する型を新規追加
pub struct SpaceReservationRow {
    pub reservation_id: ReservationId,
    pub space_id: SpaceId,
    pub user_id: UserId,
    pub user_name: String,
    pub reserved_at: DateTime<Local>,
}

// Reservation 型に変換する From トレイト実装を追加
impl From<SpaceReservationRow> for Reservation {
    fn from(value: SpaceReservationRow) -> Self {
        let SpaceReservationRow {
            reservation_id,
            space_id: _,
            user_id,
            user_name,
            reserved_at,
        } = value;
        Reservation {
            reservation_id,
            reserved_by: ReservationUser {
                user_id: user_id,
                user_name: user_name,
            },
            reserved_at,
        }
    }
}