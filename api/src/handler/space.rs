use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use registry::AppRegistry;
use thiserror::Error;
use uuid::Uuid;

use crate::model::space::{SpaceResponse, CreateSpaceRequest};

#[derive(Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    InternalError(#[from] anyhow::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
    }
}

pub async fn register_space(
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateSpaceRequest>,
) -> Result<StatusCode, AppError> {
    registry
        .space_repository()
        .create(req.into())
        .await
        .map(|_| StatusCode::CREATED)
        .map_err(AppError::from)
}

pub async fn show_space_list(
    State(registry): State<AppRegistry>,
) -> Result<Json<Vec<SpaceResponse>>, AppError> {
    registry
        .space_repository()
        .find_all()
        .await
        .map(|v| v.into_iter().map(SpaceResponse::from).collect::<Vec<_>>())
        .map(Json)
        .map_err(AppError::from)
}

pub async fn show_space(
    Path(space_id): Path<Uuid>,
    State(registry): State<AppRegistry>,
) -> Result<Json<SpaceResponse>, AppError> {
    registry
        .space_repository()
        .find_by_id(space_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(anyhow::anyhow!("The specific space was not found")),
        })
        .map_err(AppError::from)
}