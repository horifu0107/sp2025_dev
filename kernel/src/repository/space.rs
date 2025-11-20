use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    id::SpaceId,
    space::{event::CreateSpace, Space},
};

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    async fn create(&self, event: CreateSpace) -> AppResult<()>;
    async fn find_all(&self) -> AppResult<Vec<Space>>;
    async fn find_by_id(&self, space_id: SpaceId) -> AppResult<Option<Space>>;
}
