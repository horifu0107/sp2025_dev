pub mod event;
use uuid::Uuid;


#[derive(Debug)]
pub struct Space {
    pub id: Uuid,
    pub space_name: String,
    pub owner: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address:String,
}