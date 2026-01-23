/// 元数据管理 Commands
use crate::models::database::DatabaseType;
use crate::models::metadata::DatabaseMetadata;
use crate::services::{cache_service, mysql_service, postgres_service};

/// 获取数据库元数据（优先返回缓存）
#[tauri::command]
pub async fn get_database_metadata(database_id: String) -> Result<DatabaseMetadata, String> {
    // 尝试从缓存加载
    if let Some(metadata_json) = cache_service::load_metadata(&database_id)
        .map_err(|e| format!("加载元数据缓存失败: {}", e))?
    {
        let metadata: DatabaseMetadata =
            serde_json::from_str(&metadata_json).map_err(|e| format!("解析元数据失败: {}", e))?;
        return Ok(metadata);
    }

    // 缓存不存在，从数据库提取
    refresh_metadata(database_id).await
}

/// 刷新数据库元数据（强制重新提取）
#[tauri::command]
pub async fn refresh_metadata(database_id: String) -> Result<DatabaseMetadata, String> {
    // 加载连接配置
    let connections =
        cache_service::load_connections().map_err(|e| format!("加载连接失败: {}", e))?;

    let connection = connections
        .iter()
        .find(|c| c.id == database_id)
        .ok_or_else(|| "数据库连接不存在".to_string())?;

    // 根据数据库类型提取元数据
    let metadata = match connection.database_type {
        DatabaseType::PostgreSQL => {
            // PostgreSQL 元数据提取
            let client = postgres_service::connect(
                &connection.host,
                connection.port,
                &connection.database_name,
                &connection.user,
                &connection.password,
            )
            .await
            .map_err(|e| format!("连接失败: {}", e))?;

            postgres_service::extract_metadata(&client, &database_id)
                .await
                .map_err(|e| format!("提取元数据失败: {}", e))?
        }
        DatabaseType::MySQL => {
            // MySQL 元数据提取
            let pool = mysql_service::connect(
                &connection.host,
                connection.port,
                &connection.database_name,
                &connection.user,
                &connection.password,
            )
            .await
            .map_err(|e| format!("连接失败: {}", e))?;

            mysql_service::extract_metadata(&pool, &database_id)
                .await
                .map_err(|e| format!("提取元数据失败: {}", e))?
        }
    };

    // 保存到缓存
    let metadata_json =
        serde_json::to_string(&metadata).map_err(|e| format!("序列化元数据失败: {}", e))?;

    cache_service::save_metadata(&database_id, &metadata_json)
        .map_err(|e| format!("保存元数据失败: {}", e))?;

    Ok(metadata)
}
