use crate::model::id::{SpaceId, UserId};


pub struct CreateSpace {
    pub space_name: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}

#[derive(Debug)]
pub struct UpdateSpace {
    pub space_id: SpaceId,
    pub space_name: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
    pub requested_user: UserId,
}

#[derive(Debug)]
pub struct DeleteSpace {
    pub space_id: SpaceId,
    pub requested_user: UserId,
}