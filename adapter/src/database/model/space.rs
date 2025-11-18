use kernel::model::space::Space;
use uuid::Uuid;

pub struct SpaceRow {
    pub space_id:Uuid,
    pub space_name: String,
    pub owner: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address:String,
}

impl From<SpaceRow> for Space {
    fn from(value: SpaceRow) -> Self {
        let SpaceRow {
            space_id,
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address
        } = value;
        Self {
            id: space_id,
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address
        }
    }
}