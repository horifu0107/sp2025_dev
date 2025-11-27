use kernel::model::{
    space::{
        event::{CreateSpace, UpdateSpace},
        Space, SpaceListOptions,
    },
    id::{SpaceId, UserId},
    list::PaginatedList,
};
use super::user::SpaceOwner;
use derive_new::new;
use garde::Validate;
use serde::{Deserialize, Serialize};

use super::user::ReservationUser;
use chrono::{DateTime, Utc};
use kernel::model::space::Reservation;
use kernel::model::id::ReservationId;


#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpaceRequest {
    #[garde(length(min = 1))]
    pub space_name: String,
    #[garde(skip)]
    pub is_active: bool,
    #[garde(skip)]
    pub description: String,
    #[garde(range(min=1))]
    pub capacity: i32,
    #[garde(length(min = 1))]
    pub equipment: String,
    #[garde(length(min = 1))]
    pub address: String,
}

impl From<CreateSpaceRequest> for CreateSpace {
    fn from(value: CreateSpaceRequest) -> Self {
        let CreateSpaceRequest {
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
        } = value;
        CreateSpace {
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
        }
    }
}

// 蔵書データの更新用の型を追加する
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSpaceRequest {
    #[garde(length(min = 1))]
    pub space_name: String,
    #[garde(skip)] 
    pub is_active: bool,
    #[garde(skip)]
    pub description: String,
    #[garde(range(min=1))]
    pub capacity: i32,
    #[garde(length(min = 1))]
    pub equipment: String,
    #[garde(skip)]
    pub address: String,
}

// パスパラメータからの SpaceId、
// リクエスト時に AuthorizedUser から取り出す UserId、
// UpdateSpaceRequest の 3 つの値のセットを UpdateSpace 型に変換するための一時的な型
#[derive(new)]
pub struct UpdateSpaceRequestWithIds(SpaceId, UserId, UpdateSpaceRequest);
impl From<UpdateSpaceRequestWithIds> for UpdateSpace {
    fn from(value: UpdateSpaceRequestWithIds) -> Self {
        let UpdateSpaceRequestWithIds(
            space_id,
            user_id,
            UpdateSpaceRequest {
                space_name,
                is_active,
                description,
                capacity,
                equipment,
                address,
            },
        ) = value;
        UpdateSpace {
            space_id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            requested_user: user_id,
        }
    }
}

// クエリで limit と offset を受け取るための型
// handler 側のメソッドで、クエリのデータを取得できる。
#[derive(Debug, Deserialize, Validate)]
pub struct SpaceListQuery {
    #[garde(range(min = 0))]
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[garde(range(min = 0))]
    #[serde(default)] // default は 0
    pub offset: i64,
}

const DEFAULT_LIMIT: i64 = 20;
const fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

impl From<SpaceListQuery> for SpaceListOptions {
    fn from(value: SpaceListQuery) -> Self {
        let SpaceListQuery { limit, offset } = value;
        Self { limit, offset }
    }
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceResponse {
    pub space_id: SpaceId,
    pub space_name: String,
    pub is_active: bool,
    pub description: String,
    pub capacity: i32,
    pub equipment: String,
    pub address: String,
    pub owner: SpaceOwner,
    pub reservation: Option<SpaceReservationResponse>,
}

impl From<Space> for SpaceResponse {
    fn from(value: Space) -> Self {
        let Space {
            space_id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            owner,
            reservation,
        } = value;
        Self {
            space_id,
            space_name,
            is_active,
            description,
            capacity,
            equipment,
            address,
            owner:owner.into(),
            reservation: reservation.map(SpaceReservationResponse::from),
        }
    }
}

// api レイヤーでのページネーション表現用の型
// 型の内部で持つフィールドは `PaginatedList<Space>` と同じであるが、
// serde::Serialize を実装しているので JSON に変換してクライアントに返せる
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedSpaceResponse {
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
    pub items: Vec<SpaceResponse>,
}

impl From<PaginatedList<Space>> for PaginatedSpaceResponse {
    fn from(value: PaginatedList<Space>) -> Self {
        let PaginatedList {
            total,
            limit,
            offset,
            items,
        } = value;
        Self {
            total,
            limit,
            offset,
            items: items.into_iter().map(SpaceResponse::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceReservationResponse {
    pub reservatio_id: ReservationId,
    pub reserved_by: ReservationUser,
    pub reserved_at: DateTime<Utc>,
}

impl From<Reservation> for SpaceReservationResponse {
    fn from(value: Reservation) -> Self {
        let Reservation {
            reservation_id,
            reserved_by,
            reserved_at,
        } = value;
        Self {
            reservatio_id: reservation_id,
            reserved_by: reserved_by.into(),
            reserved_at,
        }
    }
}