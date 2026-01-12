use crate::models::{CreateTicketRequest, Tag, Ticket, TicketWithTags, UpdateTicketRequest};
use crate::utils::error::{AppError, Result};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TicketRepository {
    pool: PgPool,
}

impl TicketRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(
        &self,
        tag_id: Option<Uuid>,
        search: Option<String>,
        completed: Option<bool>,
    ) -> Result<Vec<TicketWithTags>> {
        // Base query for tickets
        let mut query = String::from(
            "SELECT DISTINCT t.id, t.title, t.description, t.completed, t.created_at, t.updated_at
             FROM tickets t"
        );

        let mut conditions = Vec::new();
        let mut bind_count = 0;

        // Join with ticket_tags if filtering by tag
        if tag_id.is_some() {
            query.push_str(" INNER JOIN ticket_tags tt ON t.id = tt.ticket_id");
            bind_count += 1;
            conditions.push(format!("tt.tag_id = ${}", bind_count));
        }

        // Add search condition
        if search.is_some() {
            bind_count += 1;
            conditions.push(format!("t.title ILIKE ${}", bind_count));
        }

        // Add completed filter
        if completed.is_some() {
            bind_count += 1;
            conditions.push(format!("t.completed = ${}", bind_count));
        }

        // Add WHERE clause if there are conditions
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY t.created_at DESC");

        // Build and execute query
        let mut sql_query = sqlx::query_as::<_, Ticket>(&query);

        if let Some(tag) = tag_id {
            sql_query = sql_query.bind(tag);
        }
        if let Some(search_term) = search {
            sql_query = sql_query.bind(format!("%{}%", search_term));
        }
        if let Some(is_completed) = completed {
            sql_query = sql_query.bind(is_completed);
        }

        let tickets = sql_query.fetch_all(&self.pool).await?;

        // Fetch tags for each ticket
        let mut tickets_with_tags = Vec::new();
        for ticket in tickets {
            let tags = self.get_ticket_tags(ticket.id).await?;
            tickets_with_tags.push(TicketWithTags { ticket, tags });
        }

        Ok(tickets_with_tags)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TicketWithTags>> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "SELECT id, title, description, completed, created_at, updated_at 
             FROM tickets 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match ticket {
            Some(ticket) => {
                let tags = self.get_ticket_tags(ticket.id).await?;
                Ok(Some(TicketWithTags { ticket, tags }))
            }
            None => Ok(None),
        }
    }

    pub async fn create(&self, request: CreateTicketRequest) -> Result<Ticket> {
        let mut tx = self.pool.begin().await?;

        // Insert ticket
        let ticket = sqlx::query_as::<_, Ticket>(
            "INSERT INTO tickets (title, description, completed)
             VALUES ($1, $2, false)
             RETURNING id, title, description, completed, created_at, updated_at"
        )
        .bind(&request.title)
        .bind(&request.description)
        .fetch_one(&mut *tx)
        .await?;

        // Add tags if provided
        if let Some(tag_ids) = request.tag_ids {
            for tag_id in tag_ids {
                sqlx::query(
                    "INSERT INTO ticket_tags (ticket_id, tag_id)
                     VALUES ($1, $2)
                     ON CONFLICT DO NOTHING"
                )
                .bind(ticket.id)
                .bind(tag_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(ticket)
    }

    pub async fn update(&self, id: Uuid, request: UpdateTicketRequest) -> Result<Ticket> {
        // Check if ticket exists
        let existing = self.find_by_id(id).await?;
        if existing.is_none() {
            return Err(AppError::NotFound(format!("Ticket with id {} not found", id)));
        }

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if request.title.is_some() {
            updates.push(format!("title = ${}", bind_count));
            bind_count += 1;
        }
        if request.description.is_some() {
            updates.push(format!("description = ${}", bind_count));
            bind_count += 1;
        }
        if request.completed.is_some() {
            updates.push(format!("completed = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            // No updates, just return the existing ticket
            return Ok(existing.unwrap().ticket);
        }

        let query = format!(
            "UPDATE tickets SET {} WHERE id = ${} 
             RETURNING id, title, description, completed, created_at, updated_at",
            updates.join(", "),
            bind_count
        );

        let mut sql_query = sqlx::query_as::<_, Ticket>(&query);

        if let Some(title) = request.title {
            sql_query = sql_query.bind(title);
        }
        if let Some(description) = request.description {
            sql_query = sql_query.bind(description);
        }
        if let Some(completed) = request.completed {
            sql_query = sql_query.bind(completed);
        }

        sql_query = sql_query.bind(id);

        let ticket = sql_query.fetch_one(&self.pool).await?;
        Ok(ticket)
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tickets WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Ticket with id {} not found", id)));
        }

        Ok(())
    }

    pub async fn toggle_completed(&self, id: Uuid) -> Result<Ticket> {
        let ticket = sqlx::query_as::<_, Ticket>(
            "UPDATE tickets 
             SET completed = NOT completed 
             WHERE id = $1
             RETURNING id, title, description, completed, created_at, updated_at"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        ticket.ok_or_else(|| AppError::NotFound(format!("Ticket with id {} not found", id)))
    }

    pub async fn add_tag(&self, ticket_id: Uuid, tag_id: Uuid) -> Result<()> {
        // Check if ticket exists
        let ticket = self.find_by_id(ticket_id).await?;
        if ticket.is_none() {
            return Err(AppError::NotFound(format!("Ticket with id {} not found", ticket_id)));
        }

        sqlx::query(
            "INSERT INTO ticket_tags (ticket_id, tag_id)
             VALUES ($1, $2)
             ON CONFLICT DO NOTHING"
        )
        .bind(ticket_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn remove_tag(&self, ticket_id: Uuid, tag_id: Uuid) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM ticket_tags 
             WHERE ticket_id = $1 AND tag_id = $2"
        )
        .bind(ticket_id)
        .bind(tag_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(
                format!("Tag {} not found on ticket {}", tag_id, ticket_id)
            ));
        }

        Ok(())
    }

    // Helper method to get tags for a ticket
    async fn get_ticket_tags(&self, ticket_id: Uuid) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT t.id, t.name, t.color, t.created_at
             FROM tags t
             INNER JOIN ticket_tags tt ON t.id = tt.tag_id
             WHERE tt.ticket_id = $1
             ORDER BY t.name"
        )
        .bind(ticket_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}
