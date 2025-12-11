use crate::model::{
    reservation::{
        event::{CreateReservation, UpdateReturned},
        Reservation,
    },
    id::{ SpaceId, UserId,ReservationId},
};
use async_trait::async_trait;
use shared::error::AppResult;

#[async_trait]
pub trait ReservationRepository: Send + Sync {
    // 予約操作を行う
    async fn create(&self, event: CreateReservation) -> AppResult<ReservationId>;
    // 予約終了操作を行う
    async fn update_returned(&self, event: UpdateReturned) -> AppResult<()>;
    // すべての現在の予約情報を取得する
    async fn find_unreturned_all(&self) -> AppResult<Vec<Reservation>>;
    // reservation_idからReservation型のデータを渡す
    async fn find_by_id(&self,reservation_id:ReservationId) -> AppResult<Reservation>;
    // ユーザー ID に紐づく現在の予約情報を取得する
    async fn find_unreturned_by_user_id(&self, user_id: UserId) -> AppResult<Vec<Reservation>>;
    //スペース　ID　に紐づく予約中の予約一覧を取得する
    async fn find_reservations_by_space_id(&self, space_id: SpaceId) -> AppResult<Vec<Reservation>>;
    //リザーブテーション　ID　に紐づくreminder_is_alreadyを更新する
    async fn update_reminder_is_already(&self, reservatio_id:ReservationId,reminder_is_already:bool) -> AppResult<()>;
    // 予約履歴を取得する
    async fn find_history_by_space_id(&self, space_id:  SpaceId) -> AppResult<Vec<Reservation>>;
}