pub mod event;
use super::id::SpaceId;

#[derive(Debug)]
pub struct Space {
    pub id: SpaceId,
    pub space_name: String,
    pub owner: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}
