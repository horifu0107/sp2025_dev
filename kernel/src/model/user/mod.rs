// kernel/src/model/user/mod.rs
use crate::model::{id::UserId, role::Role};
pub mod event;

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    pub user_id: UserId,
    pub user_name: String,
    pub email: String,
    pub role: Role,
}

#[derive(Debug)]
pub struct SpaceOwner {
    pub owner_id: UserId,
    pub owner_name: String,
}

#[derive(Debug)]
pub struct ReservationUser {
    pub user_id: UserId,
    pub user_name: String,
}