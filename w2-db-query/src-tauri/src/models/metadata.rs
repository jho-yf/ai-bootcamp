use chrono::{DateTime, Utc};
/// 数据库元数据模型
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseMetadata {
    /// 对应的数据库连接 ID
    pub connection_id: String,

    /// 所有表
    pub tables: Vec<TableInfo>,

    /// 所有视图
    pub views: Vec<ViewInfo>,

    /// 元数据提取时间
    pub extracted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInfo {
    /// Schema 名称（通常是 "public"）
    pub schema: String,

    /// 表名
    pub name: String,

    /// 表类型（"table" 或 "view"）
    pub table_type: String,

    /// 所有列
    pub columns: Vec<ColumnInfo>,

    /// 主键列名列表
    pub primary_keys: Vec<String>,

    /// 外键约束
    pub foreign_keys: Vec<ForeignKeyInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ViewInfo {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    /// 视图定义 SQL（可选）
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInfo {
    /// 列名
    pub name: String,

    /// 数据类型（如 "integer", "varchar", "timestamp"）
    pub data_type: String,

    /// 是否可空
    pub nullable: bool,

    /// 默认值（可选）
    pub default_value: Option<String>,

    /// 是否为主键
    pub is_primary_key: bool,

    /// 列位置（用于排序）
    pub ordinal_position: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignKeyInfo {
    /// 外键名称
    pub constraint_name: String,

    /// 本表列名
    pub column_name: String,

    /// 引用的表名
    pub referenced_table: String,

    /// 引用的列名
    pub referenced_column: String,
}
