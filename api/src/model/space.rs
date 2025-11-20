use kernel::model::{
    id::SpaceId,
    space::{event::CreateSpace, Space},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceRequest {
    pub space_name: String,
    pub owner: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}

impl From<CreateSpaceRequest> for CreateSpace {
    fn from(value: CreateSpaceRequest) -> Self {
        let CreateSpaceRequest {
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address,
        } = value;
        CreateSpace {
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceResponse {
    pub id: SpaceId,
    pub space_name: String,
    pub owner: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
}

impl From<Space> for SpaceResponse {
    fn from(value: Space) -> Self {
        let Space {
            id,
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address,
        } = value;
        Self {
            id,
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address,
        }
    }
}
