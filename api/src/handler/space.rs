use crate::{
    extractor::AuthorizedUser,
    model::space::{
        SpaceListQuery, SpaceResponse, CreateSpaceRequest, PaginatedSpaceResponse, UpdateSpaceRequest,
        UpdateSpaceRequestWithIds,
    },
};use axum::{
    extract::{Path, Query,State},
    http::StatusCode,
    Json,
};
use garde::Validate;
use kernel::model::{space::event::DeleteSpace, id::SpaceId};

use registry::AppRegistry;
use shared::error::{AppError, AppResult};
use thiserror::Error;

// #[derive(Error, Debug)]
// pub enum AppError {
//     #[error("{0}")]
//     InternalError(#[from] anyhow::Error),
// }
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         (StatusCode::INTERNAL_SERVER_ERROR, "").into_response()
//     }
// }

pub async fn register_space(
    user: AuthorizedUser,
    State(registry): State<AppRegistry>,
    Json(req): Json<CreateSpaceRequest>,
) -> Result<StatusCode, AppError> {
    req.validate(&())?;

    registry
        .space_repository()
        .create(req.into(), user.id())
        .await
        .map(|_| StatusCode::CREATED)
}

pub async fn show_space_list(
    _user: AuthorizedUser,
    Query(query): Query<SpaceListQuery>,
    State(registry): State<AppRegistry>,
) ->  AppResult<Json<PaginatedSpaceResponse>> {
    query.validate(&())?;

    registry
        .space_repository()
        .find_all(query.into())
        .await
        .map(PaginatedSpaceResponse::from)
        .map(Json)
}

pub async fn show_space(
    _user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
) -> AppResult<Json<SpaceResponse>> {
    registry
        .space_repository()
        .find_by_id(space_id)
        .await
        .and_then(|bc| match bc {
            Some(bc) => Ok(Json(bc.into())),
            None => Err(AppError::EntityNotFound("not found".into())),
        })
}

pub async fn update_space(
    user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
    Json(req): Json<UpdateSpaceRequest>,
) -> AppResult<StatusCode> {
    req.validate(&())?;

    let update_space = UpdateSpaceRequestWithIds::new(space_id, user.id(), req);
    registry
        .space_repository()
        .update(update_space.into())
        .await
        .map(|_| StatusCode::OK)
}

pub async fn delete_space(
    user: AuthorizedUser,
    Path(space_id): Path<SpaceId>,
    State(registry): State<AppRegistry>,
) -> AppResult<StatusCode> {
    let delete_space = DeleteSpace {
        space_id,
        requested_user: user.id(),
    };
    registry
        .space_repository()
        .delete(delete_space)
        .await
        .map(|_| StatusCode::OK)
}