use crate::models::{CreateTicketRequest, Tag, Ticket, TicketWithTags, UpdateTicketRequest};
use crate::utils::error::{AppError, Result};
use chrono::Utc;
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

    /// Find all tickets with optional filtering by tag, search term, and completed status.
    /// 
    /// # Performance Note
    /// This method performs N+1 queries (one for tickets, then one per ticket for tags).
    /// For large datasets, consider optimizing with a single JOIN query and grouping in application code.
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

        // Add search condition (uses ILIKE for case-insensitive search)
        // The GIN index on title will help optimize this query
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
        // Note: This results in N+1 queries. For better performance with large datasets,
        // consider fetching all tags in a single query and grouping by ticket_id.
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

        let now = Utc::now();

        // Insert ticket
        let ticket = sqlx::query_as::<_, Ticket>(
            "INSERT INTO tickets (title, description, completed, created_at, updated_at)
             VALUES ($1, $2, false, $3, $3)
             RETURNING id, title, description, completed, created_at, updated_at"
        )
        .bind(&request.title)
        .bind(&request.description)
        .bind(now)
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

    /// Update a ticket with partial data.
    /// 
    /// Only provided fields will be updated. The `updated_at` field is always updated.
    /// Returns an error if the ticket does not exist.
    pub async fn update(&self, id: Uuid, request: UpdateTicketRequest) -> Result<Ticket> {
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
            // No updates, fetch and return the existing ticket
            return self
                .find_by_id(id)
                .await?
                .map(|twt| twt.ticket)
                .ok_or_else(|| AppError::NotFound(format!("Ticket with id {} not found", id)));
        }

        // Always update updated_at
        updates.push(format!("updated_at = ${}", bind_count));
        bind_count += 1;

        let query = format!(
            "UPDATE tickets SET {} WHERE id = ${} 
             RETURNING id, title, description, completed, created_at, updated_at",
            updates.join(", "),
            bind_count
        );

        let now = Utc::now();
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

        sql_query = sql_query.bind(now);
        sql_query = sql_query.bind(id);

        let ticket = sql_query.fetch_optional(&self.pool).await?;
        ticket.ok_or_else(|| AppError::NotFound(format!("Ticket with id {} not found", id)))
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
        let now = Utc::now();
        let ticket = sqlx::query_as::<_, Ticket>(
            "UPDATE tickets 
             SET completed = NOT completed, updated_at = $1
             WHERE id = $2
             RETURNING id, title, description, completed, created_at, updated_at"
        )
        .bind(now)
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

    /// Helper method to get tags for a ticket.
    /// 
    /// This method is used internally to fetch tags for tickets.
    /// The query uses an INNER JOIN which is efficient due to the index on ticket_tags.
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
