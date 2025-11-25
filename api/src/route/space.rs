use axum::{
    routing::{get, post},
    Router,
};
use registry::AppRegistry;

use crate::handler::space::{register_space, show_space, show_space_list};

pub fn build_space_routers() -> Router<AppRegistry> {
    let spaces_routers = Router::new()
        .route("/", post(register_space))
        .route("/", get(show_space_list))
        .route("/:space_id", get(show_space));

    Router::new().nest("/spaces", spaces_routers)
}
