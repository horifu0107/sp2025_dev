use async_trait::async_trait;
use shared::error::AppResult;

use crate::model::{
    id::{SpaceId,UserId},
    space::{event::{CreateSpace, DeleteSpace, UpdateSpace},
        Space, SpaceListOptions,},
    list::PaginatedList,
};

#[async_trait]
pub trait SpaceRepository: Send + Sync {
    async fn create(&self, event: CreateSpace,user_id: UserId) -> AppResult<()>;
    async fn find_all(&self,options: SpaceListOptions) -> AppResult<PaginatedList<Space>>;
    async fn find_by_id(&self, space_id: SpaceId) -> AppResult<Option<Space>>;
    async fn find_all_space_for_all_cancel(&self) -> AppResult<Vec<Space>>;
    async fn update(&self, event: UpdateSpace) -> AppResult<()>;
    async fn update_is_active(&self, event: UpdateSpace) -> AppResult<()>;
    async fn delete(&self, event: DeleteSpace) -> AppResult<()>;
}
