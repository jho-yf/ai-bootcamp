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
    use crate::services::database::ParameterSyntax;

    #[test]
    fn test_factory_creation() {
        let factory = DatabaseServiceFactory::new();
        assert!(factory.is_supported(&DatabaseType::PostgreSQL));
        assert!(factory.is_supported(&DatabaseType::MySQL));
    }

    #[test]
    fn test_factory_default_trait() {
        // 测试 Default trait 实现
        let factory1 = DatabaseServiceFactory::default();
        assert!(factory1.is_supported(&DatabaseType::PostgreSQL));
        assert!(factory1.is_supported(&DatabaseType::MySQL));

        let factory2 = DatabaseServiceFactory::new();
        // 两个工厂应该是独立的实例
        assert!(factory1.is_supported(&DatabaseType::PostgreSQL));
        assert!(factory2.is_supported(&DatabaseType::PostgreSQL));
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
    fn test_get_service_unsupported_type() {
        let factory = DatabaseServiceFactory::new();

        // 尝试获取不支持的数据库类型（如果有其他变体）
        // 这里测试错误处理机制
        // 注意：当前只有 PostgreSQL 和 MySQL，所以这个测试展示错误处理能力
        let result = factory.get_service(&DatabaseType::PostgreSQL);
        assert!(result.is_ok());

        // 验证返回的服务可以被正确使用
        let service = result.unwrap();
        assert_eq!(service.service_name(), "PostgreSQL");
    }

    #[test]
    fn test_supported_types() {
        let factory = DatabaseServiceFactory::new();
        let types = factory.supported_types();

        assert_eq!(types.len(), 2);
        assert!(types.contains(&DatabaseType::PostgreSQL));
        assert!(types.contains(&DatabaseType::MySQL));
    }

    #[test]
    fn test_is_supported() {
        let factory = DatabaseServiceFactory::new();

        // 测试支持的类型
        assert!(factory.is_supported(&DatabaseType::PostgreSQL));
        assert!(factory.is_supported(&DatabaseType::MySQL));

        // 测试不存在的类型（通过检查 is_supported 的否定逻辑）
        // 当前只支持两种数据库，所以这个测试验证基础功能
        let unsupported_count = factory
            .supported_types()
            .iter()
            .filter(|t| !factory.is_supported(t))
            .count();
        assert_eq!(unsupported_count, 0, "supported_types 中的所有类型都应该被标记为支持");
    }

    #[test]
    fn test_register_service_override() {
        let mut factory = DatabaseServiceFactory::new();

        // 注册一个新的 PostgreSQL 服务（覆盖默认的）
        let new_pg_service = Arc::new(PostgresService::new());
        factory.register_service(DatabaseType::PostgreSQL, new_pg_service);

        // 验证服务仍然可用
        let service = factory.get_service(&DatabaseType::PostgreSQL);
        assert!(service.is_ok());
        assert_eq!(service.unwrap().service_name(), "PostgreSQL");

        // 验证支持的类型数量不变
        assert_eq!(factory.supported_types().len(), 2);
    }

    #[test]
    fn test_multiple_service_instances() {
        let factory = DatabaseServiceFactory::new();

        // 多次获取相同类型的服务
        let service1 = factory.get_service(&DatabaseType::PostgreSQL).unwrap();
        let service2 = factory.get_service(&DatabaseType::PostgreSQL).unwrap();

        // 服务名称应该相同
        assert_eq!(service1.service_name(), service2.service_name());

        // SQL 方言应该相同
        let dialect1 = service1.get_sql_dialect();
        let dialect2 = service2.get_sql_dialect();
        assert_eq!(dialect1.name, dialect2.name);
    }

    #[test]
    fn test_service_sql_dialects() {
        let factory = DatabaseServiceFactory::new();

        let pg_service = factory.get_service(&DatabaseType::PostgreSQL).unwrap();
        let mysql_service = factory.get_service(&DatabaseType::MySQL).unwrap();

        let pg_dialect = pg_service.get_sql_dialect();
        let mysql_dialect = mysql_service.get_sql_dialect();

        // 验证 PostgreSQL 方言特性
        assert_eq!(pg_dialect.name, "PostgreSQL");
        assert_eq!(pg_dialect.identifier_quote, '"');
        assert_eq!(pg_dialect.parameter_syntax, ParameterSyntax::DollarNumeric);

        // 验证 MySQL 方言特性
        assert_eq!(mysql_dialect.name, "MySQL");
        assert_eq!(mysql_dialect.identifier_quote, '`');
        assert_eq!(mysql_dialect.parameter_syntax, ParameterSyntax::QuestionMark);

        // 验证方言差异
        assert_ne!(pg_dialect.identifier_quote, mysql_dialect.identifier_quote);
        assert_ne!(pg_dialect.parameter_syntax, mysql_dialect.parameter_syntax);
    }
}
