use crate::models::{CreateTagRequest, Tag, UpdateTagRequest};
use crate::utils::error::{AppError, Result};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct TagRepository {
    pool: PgPool,
}

impl TagRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_all(&self) -> Result<Vec<Tag>> {
        let tags = sqlx::query_as::<_, Tag>(
            "SELECT id, name, color, created_at 
             FROM tags 
             ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Tag>> {
        let tag = sqlx::query_as::<_, Tag>(
            "SELECT id, name, color, created_at 
             FROM tags 
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tag)
    }

    pub async fn find_by_name(&self, name: &str) -> Result<Option<Tag>> {
        let tag = sqlx::query_as::<_, Tag>(
            "SELECT id, name, color, created_at 
             FROM tags 
             WHERE name = $1"
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(tag)
    }

    pub async fn create(&self, request: CreateTagRequest) -> Result<Tag> {
        // Check if tag with same name already exists
        if let Some(_existing) = self.find_by_name(&request.name).await? {
            return Err(AppError::Validation(
                format!("Tag with name '{}' already exists", request.name)
            ));
        }

        let now = Utc::now();
        let tag = sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (name, color, created_at)
             VALUES ($1, $2, $3)
             RETURNING id, name, color, created_at"
        )
        .bind(&request.name)
        .bind(&request.color)
        .bind(now)
        .fetch_one(&self.pool)
        .await?;

        Ok(tag)
    }

    /// Update a tag with partial data.
    /// 
    /// Only provided fields will be updated. Returns an error if the tag does not exist
    /// or if the new name conflicts with an existing tag.
    pub async fn update(&self, id: Uuid, request: UpdateTagRequest) -> Result<Tag> {
        // If updating name, check for duplicates
        if let Some(ref new_name) = request.name {
            if let Some(existing_tag) = self.find_by_name(new_name).await? {
                if existing_tag.id != id {
                    return Err(AppError::Validation(
                        format!("Tag with name '{}' already exists", new_name)
                    ));
                }
            }
        }

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut bind_count = 1;

        if request.name.is_some() {
            updates.push(format!("name = ${}", bind_count));
            bind_count += 1;
        }
        if request.color.is_some() {
            updates.push(format!("color = ${}", bind_count));
            bind_count += 1;
        }

        if updates.is_empty() {
            // No updates, fetch and return the existing tag
            return self
                .find_by_id(id)
                .await?
                .ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", id)));
        }

        let query = format!(
            "UPDATE tags SET {} WHERE id = ${} 
             RETURNING id, name, color, created_at",
            updates.join(", "),
            bind_count
        );

        let mut sql_query = sqlx::query_as::<_, Tag>(&query);

        if let Some(name) = request.name {
            sql_query = sql_query.bind(name);
        }
        if let Some(color) = request.color {
            sql_query = sql_query.bind(color);
        }

        sql_query = sql_query.bind(id);

        let tag = sql_query.fetch_optional(&self.pool).await?;
        tag.ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", id)))
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query("DELETE FROM tags WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound(format!("Tag with id {} not found", id)));
        }

        Ok(())
    }
}
