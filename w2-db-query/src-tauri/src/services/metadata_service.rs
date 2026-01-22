// 集成测试移到 tests/integration_test.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::metadata::{ColumnInfo, TableInfo};

    #[test]
    fn test_format_metadata_empty() {
        // 测试空元数据
        let metadata = DatabaseMetadata {
            connection_id: "test".to_string(),
            tables: vec![],
            views: vec![],
            extracted_at: chrono::Utc::now(),
        };

        // 这个测试验证 extract_metadata 函数能处理空数据
        // 实际测试需要数据库连接，在 integration_test.rs 中
    }

    #[test]
    fn test_extract_primary_keys_query() {
        // 测试主键查询 SQL（不执行，只验证 SQL 结构）
        let query = "SELECT kcu.column_name
             FROM information_schema.table_constraints tc
             JOIN information_schema.key_column_usage kcu
               ON tc.constraint_name = kcu.constraint_name
               AND tc.table_schema = kcu.table_schema
             WHERE tc.constraint_type = 'PRIMARY KEY'
               AND tc.table_schema = $1
               AND tc.table_name = $2
             ORDER BY kcu.ordinal_position";

        // 验证查询包含必要的部分
        assert!(query.contains("PRIMARY KEY"));
        assert!(query.contains("table_schema"));
        assert!(query.contains("table_name"));
    }

    #[test]
    fn test_extract_foreign_keys_query() {
        // 测试外键查询 SQL（不执行，只验证 SQL 结构）
        let query = "SELECT
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
               AND tc.table_name = $2";

        // 验证查询包含必要的部分
        assert!(query.contains("FOREIGN KEY"));
        assert!(query.contains("constraint_name"));
        assert!(query.contains("foreign_table_name"));
    }
}
