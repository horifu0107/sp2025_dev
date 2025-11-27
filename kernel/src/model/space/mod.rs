pub mod event;
use super::{id::{SpaceId,ReservationId}, user::{SpaceOwner,ReservationUser}};
use chrono::{DateTime,Utc};

#[derive(Debug)]
pub struct Space {
    pub space_id: SpaceId,
    pub space_name: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
    pub owner: SpaceOwner,
    pub reservation: Option<Reservation>,
}

// ページネーションの範囲を指定するための設定値を格納する型
#[derive(Debug)]
pub struct SpaceListOptions {
    pub limit: i64,
    pub offset: i64,
}

// この型は、model::checkout モジュール側でも同名の型を定義しているが
// それとは異なるモジュールにあるので別の型として扱われる。
// 実際、上記 `Book` 型の checkout フィールドとしてのみ使用する。
#[derive(Debug)]
pub struct Reservation {
    pub reservation_id: ReservationId,
    pub reserved_by: ReservationUser,
    pub reserved_at: DateTime<Utc>,
}