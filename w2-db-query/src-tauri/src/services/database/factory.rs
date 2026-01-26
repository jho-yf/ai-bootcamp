// 数据库服务工厂实现
//
// 使用工厂模式创建和管理数据库服务实例

use super::r#trait::{DatabaseService};
use crate::models::database::DatabaseType;
use crate::utils::error::AppError;
use std::collections::HashMap;
use std::sync::Arc;

// 导入数据库服务实现
use super::postgres_impl::PostgresService;
use super::mysql_impl::MySqlService;

/// 数据库服务工厂
pub struct DatabaseServiceFactory {
    services: HashMap<DatabaseType, Arc<dyn DatabaseService>>,
}

impl DatabaseServiceFactory {
    /// 创建新工厂并注册内置服务
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

    /// 注册新数据库服务
    pub fn register_service(
        &mut self,
        db_type: DatabaseType,
        service: Arc<dyn DatabaseService>,
    ) {
        self.services.insert(db_type, service);
    }

    /// 获取特定数据库类型的服务
    pub fn get_service(&self, db_type: &DatabaseType) -> Result<Arc<dyn DatabaseService>, AppError> {
        self.services
            .get(db_type)
            .cloned()
            .ok_or_else(|| AppError::DatabaseConnection(format!("不支持的数据库类型: {:?}", db_type)))
    }

    /// 获取所有支持的数据库类型
    pub fn supported_types(&self) -> Vec<DatabaseType> {
        self.services.keys().cloned().collect()
    }

    /// 检查是否支持某个数据库类型
    pub fn is_supported(&self, db_type: &DatabaseType) -> bool {
        self.services.contains_key(db_type)
    }
}

impl Default for DatabaseServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局工厂实例
static mut GLOBAL_FACTORY: Option<DatabaseServiceFactory> = None;

/// 初始化全局工厂（从 main.rs 调用）
pub fn init_global_factory() {
    unsafe {
        GLOBAL_FACTORY = Some(DatabaseServiceFactory::new());
    }
}

/// 获取全局工厂实例
pub fn get_global_factory() -> &'static DatabaseServiceFactory {
    unsafe {
        GLOBAL_FACTORY
            .as_ref()
            .expect("Factory not initialized. Call init_global_factory() first.")
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
    fn test_supported_types() {
        let factory = DatabaseServiceFactory::new();
        let types = factory.supported_types();

        assert_eq!(types.len(), 2);
        assert!(types.contains(&DatabaseType::PostgreSQL));
        assert!(types.contains(&DatabaseType::MySQL));
    }
}
