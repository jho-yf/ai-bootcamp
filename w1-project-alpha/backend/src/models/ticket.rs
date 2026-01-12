use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Ticket {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTicketRequest {
    pub title: String,
    pub description: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTicketRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketWithTags {
    #[serde(flatten)]
    pub ticket: Ticket,
    pub tags: Vec<super::tag::Tag>,
}
