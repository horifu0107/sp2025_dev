use axum::{
    routing::{delete, get, post, put},
    Router,
};
use registry::AppRegistry;

use crate::handler::space::{delete_space, register_space, show_space, show_space_list, update_space};

pub fn build_space_routers() -> Router<AppRegistry> {
    let spaces_routers = Router::new()
        .route("/", post(register_space))
        .route("/", get(show_space_list))
        .route("/:space_id", get(show_space))
        .route("/:space_id", put(update_space))
        .route("/:space_id", delete(delete_space));

    Router::new().nest("/spaces", spaces_routers)
}