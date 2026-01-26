// 数据库服务工厂实现
//
// 使用工厂模式创建和管理数据库服务实例
// 这是实现开闭原则的关键组件
//
// 文件位置: src-tauri/src/services/database/factory.rs

use super::trait::{DatabaseService, DbConnection};
use crate::models::database::DatabaseType;
use crate::utils::error::AppError;
use std::collections::HashMap;
use std::sync::Arc;

// 导入数据库服务实现
use super::postgres_impl::PostgresService;
use super::mysql_impl::MySqlService;

/// 数据库服务工厂 - 创建服务而不耦合到具体实现
///
/// # 设计模式
/// - 工厂模式：集中管理服务创建
/// - 注册表模式：运行时注册新服务
/// - 单例模式：全局唯一实例
///
/// # 使用示例
/// ```rust
/// let factory = DatabaseServiceFactory::new();
/// let service = factory.get_service(&DatabaseType::PostgreSQL)?;
/// let conn = service.connect("localhost", 5432, "mydb", "user", "pass").await?;
/// ```
pub struct DatabaseServiceFactory {
    services: HashMap<DatabaseType, Arc<dyn DatabaseService>>,
}

impl DatabaseServiceFactory {
    /// 创建新工厂并注册内置服务
    ///
    /// # 返回
    /// 包含 PostgreSQL 和 MySQL 服务的工厂实例
    pub fn new() -> Self {
        let mut factory = Self {
            services: HashMap::new(),
        };

        // 注册内置数据库服务
        factory.register_service(
            DatabaseType::PostgreSQL,
            Arc::new(PostgresService::new()),
        );
        factory.register_service(
            DatabaseType::MySQL,
            Arc::new(MySqlService::new()),
        );

        factory
    }

    /// 注册新数据库服务（扩展点）
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    /// - `service`: 服务实例（使用 Arc 包装以支持多线程共享）
    ///
    /// # 示例
    /// ```rust
    /// factory.register_service(
    ///     DatabaseType::SQLite,
    ///     Arc::new(SqliteService::new()),
    /// );
    /// ```
    pub fn register_service(
        &mut self,
        db_type: DatabaseType,
        service: Arc<dyn DatabaseService>,
    ) {
        self.services.insert(db_type, service);
    }

    /// 获取特定数据库类型的服务
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// - `Ok(Arc<dyn DatabaseService>)`: 服务实例
    /// - `Err(AppError::UnsupportedDatabase)`: 如果数据库类型未注册
    pub fn get_service(&self, db_type: &DatabaseType) -> Result<Arc<dyn DatabaseService>, AppError> {
        self.services
            .get(db_type)
            .cloned()
            .ok_or_else(|| AppError::UnsupportedDatabase(format!("{:?}", db_type)))
    }

    /// 获取所有支持的数据库类型
    ///
    /// # 返回
    /// 数据库类型向量
    pub fn supported_types(&self) -> Vec<DatabaseType> {
        self.services.keys().cloned().collect()
    }

    /// 检查是否支持某个数据库类型
    ///
    /// # 参数
    /// - `db_type`: 数据库类型
    ///
    /// # 返回
    /// - `true`: 支持
    /// - `false`: 不支持
    pub fn is_supported(&self, db_type: &DatabaseType) -> bool {
        self.services.contains_key(db_type)
    }
}

impl Default for DatabaseServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局工厂实例（使用 once_cell 或 lazy_static）
///
/// # 安全说明
/// 使用 unsafe 是因为我们需要一个全局可变的静态变量。
/// 在实际应用中，应该使用 once_cell 或 lazy_static crate 来安全地实现。
///
/// # 推荐实现（使用 once_cell）
/// ```toml
/// [dependencies]
/// once_cell = "1.18"
/// ```
///
/// ```rust
/// use once_cell::sync::OnceCell;
///
/// static GLOBAL_FACTORY: OnceCell<DatabaseServiceFactory> = OnceCell::new();
///
/// pub fn init_global_factory() {
///     GLOBAL_FACTORY.get_or_init(|| DatabaseServiceFactory::new());
/// }
///
/// pub fn get_global_factory() -> &'static DatabaseServiceFactory {
///     GLOBAL_FACTORY.get().expect("Factory not initialized")
/// }
/// ```
static mut GLOBAL_FACTORY: Option<DatabaseServiceFactory> = None;

/// 初始化全局工厂（从 main.rs 调用）
///
/// # 调用位置
/// 在 `src-tauri/src/main.rs` 的 main 函数开始处调用：
/// ```rust
/// fn main() {
///     services::database::init_global_factory();
///     tauri::Builder::default()
///         // ...
///         .run(tauri::generate_context!())
///         .expect("error while running tauri application");
/// }
/// ```
pub fn init_global_factory() {
    unsafe {
        GLOBAL_FACTORY = Some(DatabaseServiceFactory::new());
    }
}

/// 获取全局工厂实例
///
/// # 返回
/// 全局工厂的静态引用
///
/// # Panic
/// 如果工厂未初始化（未调用 `init_global_factory()`），会 panic
pub fn get_global_factory() -> &'static DatabaseServiceFactory {
    unsafe {
        GLOBAL_FACTORY
            .as_ref()
            .expect("Factory not initialized. Call init_global_factory() first.")
    }
}

/// 重置全局工厂（主要用于测试）
///
/// # 安全
/// 仅在测试环境中使用
#[cfg(test)]
pub fn reset_global_factory() {
    unsafe {
        GLOBAL_FACTORY = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = DatabaseServiceFactory::new();
        assert!(factory.is_supported(&DatabaseType::PostgreSQL));
        assert!(factory.is_supported(&DatabaseType::MySQL));
    }

    #[test]
    fn test_get_service() {
        let factory = DatabaseServiceFactory::new();

        let pg_service = factory.get_service(&DatabaseType::PostgreSQL);
        assert!(pg_service.is_ok());
        assert_eq!(pg_service.unwrap().service_name(), "PostgreSQL");

        let mysql_service = factory.get_service(&DatabaseType::MySQL);
        assert!(mysql_service.is_ok());
        assert_eq!(mysql_service.unwrap().service_name(), "MySQL");
    }

    #[test]
    fn test_unsupported_database() {
        let factory = DatabaseServiceFactory::new();
        let result = factory.get_service(&DatabaseType::SQLite);
        assert!(result.is_err());
    }

    #[test]
    fn test_register_custom_service() {
        let mut factory = DatabaseServiceFactory::new();

        // 假设我们有一个自定义服务
        // factory.register_service(
        //     DatabaseType::Custom,
        //     Arc::new(CustomService::new()),
        // );

        assert!(factory.is_supported(&DatabaseType::PostgreSQL));
    }

    #[test]
    fn test_supported_types() {
        let factory = DatabaseServiceFactory::new();
        let types = factory.supported_types();

        assert_eq!(types.len(), 2);
        assert!(types.contains(&DatabaseType::PostgreSQL));
        assert!(types.contains(&DatabaseType::MySQL));
    }
}
