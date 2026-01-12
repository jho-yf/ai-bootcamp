use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    models::{CreateTicketRequest, Ticket, TicketWithTags, UpdateTicketRequest},
    repositories::Repositories,
    utils::error::Result,
};

#[derive(Debug, Deserialize)]
pub struct TicketQuery {
    pub tag: Option<Uuid>,
    pub search: Option<String>,
    pub completed: Option<bool>,
}

pub async fn get_tickets(
    Query(query): Query<TicketQuery>,
    State(repositories): State<Repositories>,
) -> Result<Json<Vec<TicketWithTags>>> {
    let tickets = repositories
        .ticket
        .find_all(query.tag, query.search, query.completed)
        .await?;
    Ok(Json(tickets))
}

pub async fn get_ticket(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
) -> Result<Json<TicketWithTags>> {
    let ticket = repositories
        .ticket
        .find_by_id(id)
        .await?
        .ok_or_else(|| crate::utils::error::AppError::NotFound(format!("Ticket with id {} not found", id)))?;
    Ok(Json(ticket))
}

pub async fn create_ticket(
    State(repositories): State<Repositories>,
    Json(request): Json<CreateTicketRequest>,
) -> Result<(StatusCode, Json<Ticket>)> {
    let ticket = repositories.ticket.create(request).await?;
    Ok((StatusCode::CREATED, Json(ticket)))
}

pub async fn update_ticket(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
    Json(request): Json<UpdateTicketRequest>,
) -> Result<Json<Ticket>> {
    let ticket = repositories.ticket.update(id, request).await?;
    Ok(Json(ticket))
}

pub async fn delete_ticket(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
) -> Result<StatusCode> {
    repositories.ticket.delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_ticket_completed(
    Path(id): Path<Uuid>,
    State(repositories): State<Repositories>,
) -> Result<Json<Ticket>> {
    let ticket = repositories.ticket.toggle_completed(id).await?;
    Ok(Json(ticket))
}

pub async fn add_tag_to_ticket(
    Path((ticket_id, tag_id)): Path<(Uuid, Uuid)>,
    State(repositories): State<Repositories>,
) -> Result<StatusCode> {
    repositories.ticket.add_tag(ticket_id, tag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_tag_from_ticket(
    Path((ticket_id, tag_id)): Path<(Uuid, Uuid)>,
    State(repositories): State<Repositories>,
) -> Result<StatusCode> {
    repositories.ticket.remove_tag(ticket_id, tag_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
