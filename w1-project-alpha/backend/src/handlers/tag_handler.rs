use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    models::{CreateTagRequest, Tag, UpdateTagRequest},
    repositories::Repositories,
    utils::error::Result,
};

pub async fn get_tags(
    State(repositories): State<Repositories>,
) -> Result<Json<Vec<Tag>>> {
    let tags = repositories.tag.find_all().await?;
    Ok(Json(tags))
}

pub async fn get_tag(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
) -> Result<Json<Tag>> {
    let tag = repositories
        .tag
        .find_by_id(id)
        .await?
        .ok_or_else(|| crate::utils::error::AppError::NotFound(format!("Tag with id {} not found", id)))?;
    Ok(Json(tag))
}

pub async fn create_tag(
    State(repositories): State<Repositories>,
    Json(request): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>)> {
    let tag = repositories.tag.create(request).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub async fn update_tag(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
    Json(request): Json<UpdateTagRequest>,
) -> Result<Json<Tag>> {
    let tag = repositories.tag.update(id, request).await?;
    Ok(Json(tag))
}

pub async fn delete_tag(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
) -> Result<StatusCode> {
    repositories.tag.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
