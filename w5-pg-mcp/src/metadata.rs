use sqlx::postgres::PgPool;
use std::collections::HashSet;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct DatabaseMetadata {
    pub schema_name: String,
    pub tables: Vec<TableInfo>,
    pub views: Vec<ViewInfo>,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub table_name: String,
    pub columns: Vec<ColumnInfo>,
    pub primary_keys: Vec<String>,
    pub indexes: Vec<IndexInfo>,
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub column_default: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IndexInfo {
    pub index_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
}

#[derive(Debug, Clone)]
pub struct ViewInfo {
    pub view_name: String,
    pub definition: String,
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Database query error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Failed to acquire database connection: {0}")]
    ConnectionError(String),
}

#[derive(Debug)]
pub struct MetadataCache {
    pool: PgPool,
    schema: String,
    excluded_tables: HashSet<String>,
    inner: RwLock<DatabaseMetadata>,
}

impl MetadataCache {
    pub fn new(pool: PgPool, schema: String, excluded_tables: HashSet<String>) -> Self {
        Self {
            pool,
            schema: schema.clone(),
            excluded_tables,
            inner: RwLock::new(DatabaseMetadata {
                schema_name: schema,
                tables: Vec::new(),
                views: Vec::new(),
            }),
        }
    }

    pub async fn load(&self) -> Result<(), MetadataError> {
        info!(schema = %self.schema, "Loading database metadata");

        let mut tx = self.pool.begin().await
            .map_err(|e| MetadataError::ConnectionError(e.to_string()))?;

        // Load tables
        let tables: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT table_name
            FROM information_schema.tables
            WHERE table_schema = $1
              AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#,
        )
        .bind(&self.schema)
        .fetch_all(&mut *tx)
        .await?;

        // Load views
        let views: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT table_name, view_definition
            FROM information_schema.views
            WHERE table_schema = $1
            ORDER BY table_name
            "#,
        )
        .bind(&self.schema)
        .fetch_all(&mut *tx)
        .await?;

        let mut table_infos = Vec::new();

        for (table_name,) in &tables {
            if self.excluded_tables.contains(&table_name.to_lowercase()) {
                debug!(table = %table_name, "Skipping excluded table");
                continue;
            }

            // Load columns with comments
            let columns: Vec<(String, String, bool, Option<String>, Option<String>)> = sqlx::query_as(
                r#"
                SELECT
                    c.column_name,
                    CONCAT(c.data_type,
                        CASE
                            WHEN c.character_maximum_length IS NOT NULL
                                THEN CONCAT('(', c.character_maximum_length, ')')
                            WHEN c.numeric_precision IS NOT NULL AND c.numeric_scale IS NOT NULL
                                THEN CONCAT('(', c.numeric_precision, ',', c.numeric_scale, ')')
                            WHEN c.numeric_precision IS NOT NULL
                                THEN CONCAT('(', c.numeric_precision, ')')
                            ELSE ''
                        END
                    ) as data_type,
                    c.is_nullable = 'YES',
                    c.column_default,
                    pgd.description
                FROM information_schema.columns c
                LEFT JOIN pg_catalog.pg_statio_all_tables st
                    ON st.schemaname = $1 AND st.relname = c.table_name
                LEFT JOIN pg_catalog.pg_description pgd
                    ON pgd.objoid = st.relid AND pgd.objsubid = c.ordinal_position
                WHERE c.table_schema = $1 AND c.table_name = $2
                ORDER BY c.ordinal_position
                "#,
            )
            .bind(&self.schema)
            .bind(table_name)
            .fetch_all(&mut *tx)
            .await?;

            // Load primary keys
            let primary_keys: Vec<(String,)> = sqlx::query_as(
                r#"
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY'
                    AND tc.table_schema = $1
                    AND tc.table_name = $2
                ORDER BY kcu.ordinal_position
                "#,
            )
            .bind(&self.schema)
            .bind(table_name)
            .fetch_all(&mut *tx)
            .await?;

            // Load indexes
            let indexes: Vec<(String, bool, String)> = sqlx::query_as(
                r#"
                SELECT
                    i.relname as index_name,
                    ix.indisunique as is_unique,
                    array_to_string(ARRAY(
                        SELECT pg_catalog.pg_get_indexdef(ix.indexrelid, k + 1, true)
                        FROM generate_subscripts(ix.indkey, 1) as k
                        ORDER BY k
                    ), ', ') as columns
                FROM pg_catalog.pg_index ix
                JOIN pg_catalog.pg_class t ON t.oid = ix.indrelid
                JOIN pg_catalog.pg_class i ON i.oid = ix.indexrelid
                JOIN pg_catalog.pg_namespace n ON n.oid = t.relnamespace
                WHERE n.nspname = $1
                    AND t.relname = $2
                ORDER BY i.relname
                "#,
            )
            .bind(&self.schema)
            .bind(table_name)
            .fetch_all(&mut *tx)
            .await?;

            let column_infos: Vec<ColumnInfo> = columns
                .into_iter()
                .map(|(name, data_type, nullable, default, comment)| ColumnInfo {
                    column_name: name,
                    data_type,
                    is_nullable: nullable,
                    column_default: default,
                    comment,
                })
                .collect();

            let pk_names: Vec<String> = primary_keys.into_iter().map(|(name,)| name).collect();

            let index_infos: Vec<IndexInfo> = indexes
                .into_iter()
                .map(|(name, is_unique, columns)| IndexInfo {
                    index_name: name,
                    columns: columns.split(", ").map(|s| s.to_string()).collect(),
                    is_unique,
                })
                .collect();

            table_infos.push(TableInfo {
                table_name: table_name.clone(),
                columns: column_infos,
                primary_keys: pk_names,
                indexes: index_infos,
            });
        }

        let view_infos: Vec<ViewInfo> = views
            .into_iter()
            .filter(|(name, _)| !self.excluded_tables.contains(&name.to_lowercase()))
            .map(|(name, definition)| ViewInfo {
                view_name: name,
                definition,
            })
            .collect();

        tx.commit().await?;

        let table_count = table_infos.len();
        let view_count = view_infos.len();

        let mut metadata = self.inner.write().await;
        metadata.tables = table_infos;
        metadata.views = view_infos;

        info!(table_count = table_count, view_count = view_count, "Metadata loaded");

        Ok(())
    }

    pub fn start_refresh_loop(self: std::sync::Arc<Self>, interval_secs: u64) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                debug!("Refreshing metadata cache");
                if let Err(e) = self.load().await {
                    tracing::error!(error = %e, "Failed to refresh metadata");
                }
            }
        })
    }

    pub async fn get_relevant_context(&self, question: &str, prompt_budget: usize) -> String {
        let metadata = self.inner.read().await;
        let question_lower = question.to_lowercase();
        let keywords: Vec<&str> = question_lower.split_whitespace().collect();

        let mut context = String::new();
        context.push_str(&format!("Database schema: {}\n\n", metadata.schema_name));

        // Score tables by relevance
        let mut scored_tables: Vec<(i32, &TableInfo)> = metadata
            .tables
            .iter()
            .map(|table| {
                let mut score = 0;
                let table_name_lower = table.table_name.to_lowercase();

                // Check if table name matches keywords
                for keyword in &keywords {
                    if table_name_lower.contains(keyword) {
                        score += 10;
                    }
                }

                // Check column names
                for column in &table.columns {
                    let column_name_lower = column.column_name.to_lowercase();
                    for keyword in &keywords {
                        if column_name_lower.contains(keyword) {
                            score += 5;
                        }
                    }
                }

                (score, table)
            })
            .collect();

        // Sort by score descending
        scored_tables.sort_by(|a, b| b.0.cmp(&a.0));

        // Format tables, stopping at budget
        for (_, table) in scored_tables {
            let table_str = Self::format_table(table);
            if context.len() + table_str.len() > prompt_budget {
                break;
            }
            context.push_str(&table_str);
            context.push('\n');
        }

        // Add view names (without definitions)
        if !metadata.views.is_empty() && context.len() < prompt_budget {
            context.push_str("\nViews:\n");
            for view in &metadata.views {
                let view_line = format!("- {}\n", view.view_name);
                if context.len() + view_line.len() > prompt_budget {
                    break;
                }
                context.push_str(&view_line);
            }
        }

        context
    }

    fn format_table(table: &TableInfo) -> String {
        let mut result = format!("Table: {}\n", table.table_name);
        result.push_str("| Column | Type | Nullable | Default |\n");
        result.push_str("|--------|------|----------|----------|\n");

        for column in &table.columns {
            let nullable = if column.is_nullable { "YES" } else { "NO" };
            let default = column.column_default.as_deref().unwrap_or("");
            result.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                column.column_name, column.data_type, nullable, default
            ));
        }

        if !table.primary_keys.is_empty() {
            result.push_str(&format!("\nPrimary keys: {}\n", table.primary_keys.join(", ")));
        }

        if !table.indexes.is_empty() {
            result.push_str("\nIndexes:\n");
            for idx in &table.indexes {
                let unique = if idx.is_unique { "UNIQUE " } else { "" };
                result.push_str(&format!(
                    "  {}{}({})\n",
                    unique,
                    idx.index_name,
                    idx.columns.join(", ")
                ));
            }
        }

        result
    }

    pub async fn table_count(&self) -> usize {
        self.inner.read().await.tables.len()
    }

    pub async fn view_count(&self) -> usize {
        self.inner.read().await.views.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_table() {
        let table = TableInfo {
            table_name: "users".to_string(),
            columns: vec![
                ColumnInfo {
                    column_name: "id".to_string(),
                    data_type: "integer".to_string(),
                    is_nullable: false,
                    column_default: Some("nextval('users_id_seq')".to_string()),
                    comment: None,
                },
                ColumnInfo {
                    column_name: "name".to_string(),
                    data_type: "varchar(255)".to_string(),
                    is_nullable: true,
                    column_default: None,
                    comment: None,
                },
            ],
            primary_keys: vec!["id".to_string()],
            indexes: vec![IndexInfo {
                index_name: "users_pkey".to_string(),
                columns: vec!["id".to_string()],
                is_unique: true,
            }],
        };

        let formatted = MetadataCache::format_table(&table);

        assert!(formatted.contains("Table: users"));
        assert!(formatted.contains("| id | integer | NO |"));
        assert!(formatted.contains("| name | varchar(255) | YES |"));
        assert!(formatted.contains("Primary keys: id"));
        assert!(formatted.contains("UNIQUE users_pkey(id)"));
    }

    #[test]
    fn test_metadata_default() {
        let metadata = DatabaseMetadata {
            schema_name: "public".to_string(),
            tables: vec![],
            views: vec![],
        };

        assert_eq!(metadata.schema_name, "public");
        assert!(metadata.tables.is_empty());
        assert!(metadata.views.is_empty());
    }
}
