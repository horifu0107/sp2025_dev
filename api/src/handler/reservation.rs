use crate::{
    extractor::AuthorizedUser,
    model::{reservation::{
        CreateReservationRequest, 
        UpdateReservationRequest,
        ReservationsResponse
    },
},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use kernel::model::{
    reservation::event::{CreateReservation, UpdateReturned},
    id::{SpaceId, ReservationId},
};
use registry::AppRegistry;
use shared::error::AppResult;

pub async fn reservation_space(
    user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateReservationRequest>,
) -> AppResult<StatusCode> {
    // reminder_at を予約開始の1時間前へ
    let reminder_at = req.reservation_start_time - chrono::Duration::hours(1);

    let create_reservation_history =CreateReservation::new(
        space_id, 
        user.id(),
        chrono::Utc::now(),
        req.reservation_start_time,  
        req.reservation_end_time,
        reminder_at,
        false,
    );

    registry
        .reservation_repository()
        .create(create_reservation_history)
        .await
        .map(|_| StatusCode::CREATED)
}

pub async fn return_space(
    user: AuthorizedUser,
    Path((space_id, reservation_id)): Path<(SpaceId, ReservationId)>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {

    // ① 予約情報を DB から取得
    let reservation = registry
        .reservation_repository()
        .find_by_id(reservation_id)
        .await?;   // Reservation を返す想定
    
    let update_returned = UpdateReturned::new(
        reservation_id, 
        space_id, 
        user.id(), 
        false,
        chrono::Utc::now(),
        reservation.reservation_start_time,
        reservation.reservation_end_time,
        reservation.reminder_at,
    );

    registry
        .reservation_repository()
        .update_returned(update_returned)
        .await
        .map(|_| StatusCode::OK)
}

pub async fn show_reserved_list(
    _user: AuthorizedUser,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ReservationsResponse>> {
    registry
        .reservation_repository()
        .find_unreturned_all()
        .await
        .map(ReservationsResponse::from)
        .map(Json)
}

pub async fn reservation_history(
    _user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<ReservationsResponse>> {
    registry
        .reservation_repository()
        .find_history_by_space_id(space_id)
        .await
        .map(ReservationsResponse::from)
        .map(Json)
}