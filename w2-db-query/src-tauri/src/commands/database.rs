/// 数据库连接管理 Commands
use crate::models::database::{
    AddDatabaseRequest, ConnectionStatus, DatabaseConnection, DatabaseType,
    TestConnectionRequest, UpdateDatabaseRequest,
};
use crate::services::{cache_service, mysql_service, postgres_service};
use chrono::Utc;
use uuid::Uuid;

/// 获取所有数据库连接
#[tauri::command]
pub async fn list_databases() -> Result<Vec<DatabaseConnection>, String> {
    cache_service::load_connections().map_err(|e| e.to_string())
}

/// 添加新数据库连接
#[tauri::command]
pub async fn add_database(request: AddDatabaseRequest) -> Result<DatabaseConnection, String> {
    // 验证参数
    if request.name.is_empty() {
        return Err("连接名称不能为空".to_string());
    }
    if request.host.is_empty() {
        return Err("主机地址不能为空".to_string());
    }
    if request.database_name.is_empty() {
        return Err("数据库名称不能为空".to_string());
    }
    if request.user.is_empty() {
        return Err("用户名不能为空".to_string());
    }

    // 测试连接（根据数据库类型选择服务）
    let test_result = match request.database_type {
        DatabaseType::PostgreSQL => {
            postgres_service::test_connection(
                &request.host,
                request.port,
                &request.database_name,
                &request.user,
                &request.password,
            )
            .await
        }
        DatabaseType::MySQL => {
            mysql_service::test_connection(
                &request.host,
                request.port,
                &request.database_name,
                &request.user,
                &request.password,
            )
            .await
        }
    };

    test_result.map_err(|e| format!("无法连接到数据库: {}", e))?;

    // 创建连接对象（先克隆需要的字段）
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let host = request.host.clone();
    let database_name = request.database_name.clone();
    let user = request.user.clone();
    let password = request.password.clone();
    let database_type = request.database_type.clone();

    let connection = DatabaseConnection {
        id: id.clone(),
        name: request.name,
        database_type,
        host: host.clone(),
        port: request.port,
        database_name: database_name.clone(),
        user: user.clone(),
        password: password.clone(),
        status: ConnectionStatus::Connected,
        created_at: now,
        updated_at: now,
    };

    // 保存到 SQLite
    cache_service::save_connection(&connection).map_err(|e| format!("保存连接失败: {}", e))?;

    Ok(connection)
}

/// 更新数据库连接
#[tauri::command]
pub async fn update_database(request: UpdateDatabaseRequest) -> Result<DatabaseConnection, String> {
    // 加载现有连接
    let mut connections =
        cache_service::load_connections().map_err(|e| format!("加载连接失败: {}", e))?;

    let connection = connections
        .iter_mut()
        .find(|c| c.id == request.id)
        .ok_or_else(|| "数据库连接不存在".to_string())?;

    // 检查是否需要重新测试连接（在移动值之前）
    let needs_test = request.database_type.is_some()
        || request.host.is_some()
        || request.port.is_some()
        || request.database_name.is_some()
        || request.user.is_some()
        || request.password.is_some();

    // 更新字段
    if let Some(name) = request.name {
        connection.name = name;
    }
    if let Some(database_type) = request.database_type {
        connection.database_type = database_type;
    }
    if let Some(host) = request.host {
        connection.host = host.clone();
    }
    if let Some(port) = request.port {
        connection.port = port;
    }
    if let Some(database_name) = request.database_name {
        connection.database_name = database_name.clone();
    }
    if let Some(user) = request.user {
        connection.user = user.clone();
    }
    if let Some(password) = request.password {
        connection.password = password.clone();
    }

    connection.updated_at = Utc::now();

    if needs_test {
        let test_result = match connection.database_type {
            DatabaseType::PostgreSQL => {
                postgres_service::test_connection(
                    &connection.host,
                    connection.port,
                    &connection.database_name,
                    &connection.user,
                    &connection.password,
                )
                .await
            }
            DatabaseType::MySQL => {
                mysql_service::test_connection(
                    &connection.host,
                    connection.port,
                    &connection.database_name,
                    &connection.user,
                    &connection.password,
                )
                .await
            }
        };

        match test_result {
            Ok(_) => connection.status = ConnectionStatus::Connected,
            Err(_) => connection.status = ConnectionStatus::Failed,
        }
    }

    // 保存更新
    cache_service::save_connection(connection).map_err(|e| format!("更新连接失败: {}", e))?;

    Ok(connection.clone())
}

/// 删除数据库连接
#[tauri::command]
pub async fn delete_database(database_id: String) -> Result<(), String> {
    cache_service::delete_connection(&database_id).map_err(|e| format!("删除连接失败: {}", e))?;
    Ok(())
}

/// 测试数据库连接
#[tauri::command]
pub async fn test_connection(request: TestConnectionRequest) -> Result<bool, String> {
    // 根据 database_type 路由到相应的服务
    let result = match request.database_type {
        DatabaseType::PostgreSQL => {
            postgres_service::test_connection(
                &request.host,
                request.port,
                &request.database_name,
                &request.user,
                &request.password,
            )
            .await
        }
        DatabaseType::MySQL => {
            mysql_service::test_connection(
                &request.host,
                request.port,
                &request.database_name,
                &request.user,
                &request.password,
            )
            .await
        }
    };

    result.map_err(|e| e.to_string())
}
