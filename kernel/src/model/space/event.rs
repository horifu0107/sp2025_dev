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
    pub space_name: Option<String>,
    pub is_active: Option<bool>,
    pub description: Option<String>,
    pub capacity: Option<i32>,
    pub equipment: Option<String>,
    pub address: Option<String>,
    pub requested_user: UserId,
}

#[derive(Debug)]
pub struct DeleteSpace {
    pub space_id: SpaceId,
    pub requested_user: UserId,
}