use crate::database::model::space::SpaceRow;
use anyhow::Result;
use async_trait::async_trait;
use derive_new::new;
use kernel::model::space::{event::CreateSpace, Space};
use kernel::repository::space::SpaceRepository;
use uuid::Uuid;

use crate::database::ConnectionPool;

#[derive(new)]
pub struct SpaceRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl SpaceRepository for SpaceRepositoryImpl {
    async fn create(&self, event: CreateSpace) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO spaces (space_name, owner, is_active, description,capacity,equipment,address)
                VALUES($1, $2, $3, $4, $5, $6, $7)
            "#,
            event.space_name,
            event.owner,
            event.is_active,
            event.description,
            event.capacity,
            event.equipment,
            event.address
        )
        .execute(self.db.inner_ref())
        .await?;

        Ok(())
    }

    async fn find_all(&self) -> Result<Vec<Space>> {
        let rows: Vec<SpaceRow> = sqlx::query_as!(
            SpaceRow,
            r#"
                SELECT
                    space_id,
                    space_name,
                    owner,
                    is_active,
                    description,
                    capacity,
                    equipment,
                    address
                FROM spaces
                ORDER BY created_at DESC
            "#
        )
        .fetch_all(self.db.inner_ref())
        .await?;

        Ok(rows.into_iter().map(Space::from).collect())
    }

    async fn find_by_id(&self, space_id: Uuid) -> Result<Option<Space>> {
        let row: Option<SpaceRow> = sqlx::query_as!(
            SpaceRow,
            r#"
                SELECT
                    space_id,
                    space_name,
                    owner,
                    is_active,
                    description,
                    capacity,
                    equipment,
                    address
                FROM spaces
                WHERE space_id = $1
            "#,
            space_id
        )
        .fetch_optional(self.db.inner_ref())
        .await?;

        Ok(row.map(Space::from))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_register_space(pool: sqlx::PgPool) -> anyhow::Result<()> {
        let repo = SpaceRepositoryImpl::new(ConnectionPool::new(pool));

        let space = CreateSpace {
            space_name: "Test SpaceName".into(),
            owner: "Test Owner".into(),
            is_active: true.into(),
            description: "Test Description".into(),
            capacity:5.into(),
            equipment:"Test Equipment".into(),
            address:"Test Address".into()
        };

        repo.create(space).await?;

        let res = repo.find_all().await?;
        assert_eq!(res.len(), 1);

        let space_id = res[0].id;
        let res = repo.find_by_id(space_id).await?;
        assert!(res.is_some());

        let Space {
            id,
            space_name,
            owner,
            is_active,
            description,
            capacity,
            equipment,
            address
        } = res.unwrap();
        assert_eq!(id, space_id);
        assert_eq!(space_name, "Test SpaceName");
        assert_eq!(owner, "Test Owner");
        assert_eq!(is_active, true);
        assert_eq!(description, "Test Description");
        assert_eq!(capacity, 5);
        assert_eq!(equipment, "Test Equipment");
        assert_eq!(address, "Test Address");
        Ok(())
    }
}