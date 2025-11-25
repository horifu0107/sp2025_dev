use kernel::model::{id::{SpaceId,UserId},
    user::SpaceOwner, 
    space::Space};

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

impl From<SpaceRow> for Space {
    fn from(value: SpaceRow) -> Self {
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
        } = value;
        Self {
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
            }
        }
    }
}

// ページネーション用の adapter 内部の型
pub struct PaginatedSpaceRow {
    pub total: i64,
    pub space_id: SpaceId,
}
