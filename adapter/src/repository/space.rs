use async_trait::async_trait;
use derive_new::new;
use kernel::model::{
    id::{SpaceId, UserId},
    {space::event::DeleteSpace, list::PaginatedList},
};
use kernel::{
    model::space::{
        event::{CreateSpace, UpdateSpace},
        Space, SpaceListOptions,
    },
    repository::space::SpaceRepository,
};
use crate::database::ConnectionPool;
use crate::database::model::space::{SpaceRow, PaginatedSpaceRow};

use shared::error::{AppError, AppResult};

#[derive(new)]
pub struct SpaceRepositoryImpl {
    db: ConnectionPool,
}

#[async_trait]
impl SpaceRepository for SpaceRepositoryImpl {
    async fn create(&self, event: CreateSpace,user_id: UserId) -> AppResult<()> {
        sqlx::query!(
            r#"
                INSERT INTO spaces (space_name,  is_active, description,capacity,equipment,address,user_id)
                VALUES($1, $2, $3, $4, $5, $6 ,$7)
            "#,
            event.space_name,
            event.is_active,
            event.description,
            event.capacity,
            event.equipment,
            event.address,
            user_id as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        Ok(())
    }

    async fn find_all(&self, options: SpaceListOptions) -> AppResult<PaginatedList<Space>> {
        let SpaceListOptions { limit, offset } = options;
         let rows: Vec<PaginatedSpaceRow> = sqlx::query_as!(
            PaginatedSpaceRow,
            r#"
                SELECT
                COUNT(*) OVER() AS "total!",
                s.space_id AS space_id
                FROM spaces AS s
                ORDER BY s.created_at DESC
                LIMIT $1
                OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let total = rows.first().map(|r| r.total).unwrap_or_default(); // レコードが 1 つもないときは total も 0 にする
        let space_ids = rows.into_iter().map(|r| r.space_id).collect::<Vec<SpaceId>>();

        let rows: Vec<SpaceRow> = sqlx::query_as!(
            SpaceRow,
            r#"
                SELECT
                    s.space_id AS space_id,
                    s.space_name AS space_name,
                    s.is_active AS is_active,
                    s.description AS description,
                    s.capacity AS capacity,
                    s.equipment AS equipment,
                    s.address AS address,
                    u.user_id AS owned_by,
                    u.user_name AS owner_name
                FROM spaces AS s
                INNER JOIN users AS u USING(user_id)
                WHERE s.space_id IN (SELECT * FROM UNNEST($1::uuid[]))
                ORDER BY s.created_at DESC
            "#,
            &space_ids as _
        )
        .fetch_all(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        let items = rows.into_iter().map(Space::from).collect();
        Ok(PaginatedList {
                    total,
                    limit,
                    offset,
                    items,
                })

    }

    async fn find_by_id(&self, space_id: SpaceId) -> AppResult<Option<Space>> {
        let row: Option<SpaceRow> = sqlx::query_as!(
            SpaceRow,
            r#"
                SELECT
                    s.space_id AS space_id,
                    s.space_name AS space_name,
                    s.is_active AS is_active,
                    s.description AS description,
                    s.capacity AS capacity,
                    s.equipment AS equipment,
                    s.address AS address, 
                    u.user_id AS owned_by,
                    u.user_name AS owner_name
                FROM spaces AS s
                INNER JOIN users AS u USING(user_id)
                WHERE s.space_id = $1
            "#,
            space_id as _
        )
        .fetch_optional(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        Ok(row.map(Space::from))
    }
    // update は SQL の UPDATE 文に当てはめているだけであるが、
    // 内容を変更できるのは所有者のみとするため、
    // SQL クエリの WHERE 条件を book_id と user_id の複合条件としている。
    async fn update(&self, event: UpdateSpace) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                UPDATE spaces
                SET
                    space_name = $1,
                    is_active = $2,
                    description = $3,
                    capacity = $4,
                    equipment = $5,
                    address = $6
                WHERE space_id = $7
                AND user_id = $8
            "#,
            event.space_name,
            event.is_active,
            event.description,
            event.capacity,
            event.equipment,
            event.address,
            event.space_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;
        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified space not found".into()));
        }

        Ok(())
    }
    // update と同様に、delete も所有者のみが行えるよう、
    // SQL クエリの WHERE の条件には book_id と user_id の複合条件にしている。
    async fn delete(&self, event: DeleteSpace) -> AppResult<()> {
        let res = sqlx::query!(
            r#"
                DELETE FROM spaces
                WHERE space_id = $1
                AND user_id = $2
            "#,
            event.space_id as _,
            event.requested_user as _
        )
        .execute(self.db.inner_ref())
        .await
        .map_err(AppError::SpecificOperationError)?;

        if res.rows_affected() < 1 {
            return Err(AppError::EntityNotFound("specified space not found".into()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::user::UserRepositoryImpl;
    use kernel::{model::user::event::CreateUser, repository::user::UserRepository};


    #[sqlx::test]
    #[ignore]
    async fn test_register_space(pool: sqlx::PgPool) -> anyhow::Result<()> {
        // 蔵書のデータを追加・取得するためにはユーザー情報がないといけないため
        // テストコードのほうでもロールおよびユーザー情報を追加するコードを足した。
        // テストコードで、このようなデータベースにあらかじめデータを追加しておくために
        // fixture という機能が便利であるが、次章で解説するためここでは愚直な実装としておく。
        sqlx::query!(r#"INSERT INTO roles(name) VALUES ('Admin'), ('User');"#)
            .execute(&pool)
            .await?;
        let user_repo = UserRepositoryImpl::new(ConnectionPool::new(pool.clone()));
        let repo = SpaceRepositoryImpl::new(ConnectionPool::new(pool));

        let space = CreateSpace {
            space_name: "Test SpaceName".into(),
            is_active: true.into(),
            description: "Test Description".into(),
            capacity: 5.into(),
            equipment: "Test Equipment".into(),
            address: "Test Address".into(),
        };

        repo.create(space,user.id).await?;
        // find_all を実行するためには BookListOptions 型の値が必要なので作る。
        let options = SpaceListOptions {
            limit: 20,
            offset: 0,
        };

        let res = repo.find_all(options).await?;
        assert_eq!(res.items.len(), 1);

        let space_id = res.items[0].id;
        let res = repo.find_by_id(space_id).await?;
        assert!(res.is_some());

        let Space {
            id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            owner,
        } = res.unwrap();
        assert_eq!(id, space_id);
        assert_eq!(space_name, "Test SpaceName");
        assert_eq!(is_active, true);
        assert_eq!(description, "Test Description");
        assert_eq!(capacity, 5);
        assert_eq!(equipment, "Test Equipment");
        assert_eq!(address, "Test Address");
        assert_eq!(owner.name, "Test User");


        Ok(())
    }
}
