/// 元数据提取服务：从 PostgreSQL 提取数据库结构信息
use crate::models::metadata::{ColumnInfo, DatabaseMetadata, ForeignKeyInfo, TableInfo, ViewInfo};
use crate::utils::error::AppError;
use chrono::Utc;
use tokio_postgres::Client;

/// 从 PostgreSQL 提取完整的数据库元数据
pub async fn extract_metadata(
    client: &Client,
    connection_id: &str,
) -> Result<DatabaseMetadata, AppError> {
    let tables = extract_tables(client).await?;
    let views = extract_views(client).await?;

    Ok(DatabaseMetadata {
        connection_id: connection_id.to_string(),
        tables,
        views,
        extracted_at: Utc::now(),
    })
}

/// 提取所有表信息
async fn extract_tables(client: &Client) -> Result<Vec<TableInfo>, AppError> {
    // 查询所有表
    let table_rows = client
        .query(
            "SELECT table_schema, table_name, table_type
             FROM information_schema.tables
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
             AND table_type = 'BASE TABLE'
             ORDER BY table_schema, table_name",
            &[],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询表失败: {}", e)))?;

    let mut tables = Vec::new();

    for row in table_rows {
        let schema: String = row.get(0);
        let table_name: String = row.get(1);
        let table_type: String = row.get(2);

        // 提取列信息
        let columns = extract_columns(client, &schema, &table_name).await?;

        // 提取主键
        let primary_keys = extract_primary_keys(client, &schema, &table_name).await?;

        // 提取外键
        let foreign_keys = extract_foreign_keys(client, &schema, &table_name).await?;

        tables.push(TableInfo {
            schema,
            name: table_name,
            table_type,
            columns,
            primary_keys,
            foreign_keys,
        });
    }

    Ok(tables)
}

/// 提取所有视图信息
async fn extract_views(client: &Client) -> Result<Vec<ViewInfo>, AppError> {
    let view_rows = client
        .query(
            "SELECT table_schema, table_name
             FROM information_schema.tables
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
             AND table_type = 'VIEW'
             ORDER BY table_schema, table_name",
            &[],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询视图失败: {}", e)))?;

    let mut views = Vec::new();

    for row in view_rows {
        let schema: String = row.get(0);
        let view_name: String = row.get(1);

        // 提取视图列信息
        let columns = extract_columns(client, &schema, &view_name).await?;

        // 可选：提取视图定义
        let definition = extract_view_definition(client, &schema, &view_name).await?;

        views.push(ViewInfo {
            schema,
            name: view_name,
            columns,
            definition,
        });
    }

    Ok(views)
}

/// 提取表的列信息
async fn extract_columns(
    client: &Client,
    schema: &str,
    table_name: &str,
) -> Result<Vec<ColumnInfo>, AppError> {
    let rows = client
        .query(
            "SELECT column_name, data_type, is_nullable, column_default, ordinal_position
             FROM information_schema.columns
             WHERE table_schema = $1 AND table_name = $2
             ORDER BY ordinal_position",
            &[&schema, &table_name],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询列失败: {}", e)))?;

    // 获取主键列
    let primary_key_cols = extract_primary_keys(client, schema, table_name).await?;
    let primary_key_set: std::collections::HashSet<String> = primary_key_cols.into_iter().collect();

    let mut columns = Vec::new();
    for row in rows {
        let column_name: String = row.get(0);
        let data_type: String = row.get(1);
        let is_nullable: String = row.get(2);
        let column_default: Option<String> = row.get(3);
        let ordinal_position: i32 = row.get(4);

        columns.push(ColumnInfo {
            name: column_name.clone(),
            data_type,
            nullable: is_nullable == "YES",
            default_value: column_default,
            is_primary_key: primary_key_set.contains(&column_name),
            ordinal_position,
        });
    }

    Ok(columns)
}

/// 提取主键列名
async fn extract_primary_keys(
    client: &Client,
    schema: &str,
    table_name: &str,
) -> Result<Vec<String>, AppError> {
    let rows = client
        .query(
            "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
               ON tc.constraint_name = kcu.constraint_name
               AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
               AND tc.table_schema = $1
               AND tc.table_name = $2
             ORDER BY kcu.ordinal_position",
            &[&schema, &table_name],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询主键失败: {}", e)))?;

    Ok(rows.iter().map(|row| row.get(0)).collect())
}

/// 提取外键约束
async fn extract_foreign_keys(
    client: &Client,
    schema: &str,
    table_name: &str,
) -> Result<Vec<ForeignKeyInfo>, AppError> {
    let rows = client
        .query(
            "SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
             FROM information_schema.table_constraints AS tc
             JOIN information_schema.key_column_usage AS kcu
               ON tc.constraint_name = kcu.constraint_name
               AND tc.table_schema = kcu.table_schema
             JOIN information_schema.constraint_column_usage AS ccu
               ON ccu.constraint_name = tc.constraint_name
               AND ccu.table_schema = tc.table_schema
             WHERE tc.constraint_type = 'FOREIGN KEY'
               AND tc.table_schema = $1
               AND tc.table_name = $2",
            &[&schema, &table_name],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询外键失败: {}", e)))?;

    let mut foreign_keys = Vec::new();
    for row in rows {
        foreign_keys.push(ForeignKeyInfo {
            constraint_name: row.get(0),
            column_name: row.get(1),
            referenced_table: row.get(2),
            referenced_column: row.get(3),
        });
    }

    Ok(foreign_keys)
}

/// 提取视图定义 SQL
async fn extract_view_definition(
    client: &Client,
    schema: &str,
    view_name: &str,
) -> Result<Option<String>, AppError> {
    let rows = client
        .query(
            "SELECT definition
             FROM pg_views
             WHERE schemaname = $1 AND viewname = $2",
            &[&schema, &view_name],
        )
        .await
        .map_err(|e| AppError::MetadataExtraction(format!("查询视图定义失败: {}", e)))?;

    if let Some(row) = rows.first() {
        Ok(Some(row.get(0)))
    } else {
        Ok(None)
    }
}

// 集成测试移到 tests/integration_test.rs
