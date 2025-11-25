pub mod event;
use super::{id::SpaceId, user::SpaceOwner};

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
}

// ページネーションの範囲を指定するための設定値を格納する型
#[derive(Debug)]
pub struct SpaceListOptions {
    pub limit: i64,
    pub offset: i64,
}