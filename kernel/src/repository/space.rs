use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::model::space::{event::CreateSpace, Space};

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    async fn create(&self, event: CreateSpace) -> Result<()>;
    async fn find_all(&self) -> Result<Vec<Space>>;
    async fn find_by_id(&self, space_id: Uuid) -> Result<Option<Space>>;
}