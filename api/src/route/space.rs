use axum::{
    routing::{delete, get, post, put},
    Router,
};
use registry::AppRegistry;

use crate::handler::{
    space::{delete_space, register_space, show_space, show_space_list, update_space},
    reservation::{reservation_space, reservation_history, return_space,cancel_space, show_reserved_list},
};

pub fn build_space_routers() -> Router<AppRegistry> {
    let spaces_routers = Router::new()
        .route("/", post(register_space))
        .route("/", get(show_space_list))
        .route("/:space_id", get(show_space))
        .route("/:space_id", put(update_space))
        .route("/:space_id", delete(delete_space));

    let reservation_router = Router::new()
        .route("/reservations", get(show_reserved_list))
        .route("/:space_id/reservations", post(reservation_space))
        .route(
            "/:space_id/reservations/:reservation_id/returned",
            put(return_space),
        )
        .route(
            "/:space_id/canceled",
            put(cancel_space),
        )
        .route("/:space_id/reservation-history", get(reservation_history));

    // merge メソッドで router を結合する
    Router::new().nest("/spaces", spaces_routers.merge(reservation_router))
}