# Project Alpha - Ticket 管理工具实现计划

## 1. 项目概述

本实现计划基于 `./specs/0001-spec.md` 中的需求和设计文档，详细描述了 Project Alpha 的具体实现步骤、时间安排和技术细节。

## 2. 项目结构

### 2.1 整体项目结构
```
project-alpha/
├── backend/                # Rust 后端项目
│   ├── src/
│   ├── Cargo.toml
│   ├── .env.example
│   └── sqlx-data.json
├── frontend/               # React 前端项目
│   ├── src/
│   ├── public/
│   ├── package.json
│   ├── vite.config.ts
│   └── tailwind.config.js
├── specs/                  # 需求和设计文档
│   ├── 0001-spec.md
│   └── 0002-implementation-plan.md
├── docker-compose.yml      # 开发环境容器编排
└── README.md
```

## 3. 后端实现计划

### 3.1 第一阶段：基础架构搭建（预计 2-3 天）

#### 3.1.1 项目初始化
1. 创建 Rust 项目
   ```bash
   cargo new backend --name project-alpha-backend
   cd backend
   ```

2. 配置 Cargo.toml 依赖
   ```toml
   [dependencies]
   # Web框架
   axum = "0.7"
   tower = "0.4"
   tower-http = { version = "0.5", features = ["cors", "trace"] }
   
   # 异步运行时
   tokio = { version = "1.35", features = ["full"] }
   
   # 数据库
   sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
   
   # 序列化
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   
   # 错误处理
   thiserror = "1.0"
   anyhow = "1.0"
   
   # 日志
   tracing = "0.1"
   tracing-subscriber = { version = "0.3", features = ["env-filter"] }
   
   # UUID
   uuid = { version = "1.6", features = ["v4", "serde"] }
   
   # 时间处理
   chrono = { version = "0.4", features = ["serde"] }
   
   # 环境配置
   dotenvy = "0.15"
   
   # 验证
   validator = { version = "0.16", features = ["derive"] }
   ```

3. 创建项目目录结构
   ```bash
   mkdir -p src/{config,models,handlers,services,repositories,utils,migrations}
   ```

#### 3.1.2 配置系统
1. 创建配置结构体 `src/config/mod.rs`
   ```rust
   use serde::Deserialize;
   
   #[derive(Debug, Deserialize)]
   pub struct Config {
       pub database: DatabaseConfig,
       pub server: ServerConfig,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct DatabaseConfig {
       pub url: String,
       pub max_connections: u32,
   }
   
   #[derive(Debug, Deserialize)]
   pub struct ServerConfig {
       pub host: String,
       pub port: u16,
   }
   
   impl Config {
       pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
           dotenvy::dotenv().ok();
           
           let config = Config {
               database: DatabaseConfig {
                   url: std::env::var("DATABASE_URL")?,
                   max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                       .unwrap_or_else(|_| "5".to_string())
                       .parse()?,
               },
               server: ServerConfig {
                   host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
                   port: std::env::var("PORT")
                       .unwrap_or_else(|_| "3000".to_string())
                       .parse()?,
               },
           };
           
           Ok(config)
       }
   }
   ```

2. 创建环境变量示例文件 `.env.example`
   ```env
   # 数据库配置
   DATABASE_URL=postgresql://username:password@localhost/project_alpha
   DATABASE_MAX_CONNECTIONS=5
   
   # 服务器配置
   HOST=127.0.0.1
   PORT=3000
   
   # 日志级别
   RUST_LOG=debug
   ```

#### 3.1.3 数据库模型
1. 创建 Ticket 模型 `src/models/ticket.rs`
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   use sqlx::FromRow;
   use uuid::Uuid;
   
   #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
   pub struct Ticket {
       pub id: Uuid,
       pub title: String,
       pub description: Option<String>,
       pub completed: bool,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct CreateTicketRequest {
       pub title: String,
       pub description: Option<String>,
       pub tag_ids: Option<Vec<Uuid>>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct UpdateTicketRequest {
       pub title: Option<String>,
       pub description: Option<String>,
       pub completed: Option<bool>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct TicketWithTags {
       #[serde(flatten)]
       pub ticket: Ticket,
       pub tags: Vec<Tag>,
   }
   ```

2. 创建 Tag 模型 `src/models/tag.rs`
   ```rust
   use chrono::{DateTime, Utc};
   use serde::{Deserialize, Serialize};
   use sqlx::FromRow;
   use uuid::Uuid;
   
   #[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
   pub struct Tag {
       pub id: Uuid,
       pub name: String,
       pub color: Option<String>,
       pub created_at: DateTime<Utc>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct CreateTagRequest {
       pub name: String,
       pub color: Option<String>,
   }
   
   #[derive(Debug, Serialize, Deserialize)]
   pub struct UpdateTagRequest {
       pub name: Option<String>,
       pub color: Option<String>,
   }
   ```

3. 创建模型模块 `src/models/mod.rs`
   ```rust
   pub mod ticket;
   pub mod tag;
   
   pub use ticket::*;
   pub use tag::*;
   ```

#### 3.1.4 数据库迁移
1. 创建初始化迁移 `migrations/20231201000001_initial_schema.sql`
   ```sql
   -- 创建 tickets 表
   CREATE TABLE tickets (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       title VARCHAR(255) NOT NULL,
       description TEXT,
       completed BOOLEAN NOT NULL DEFAULT FALSE,
       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );
   
   -- 创建 tags 表
   CREATE TABLE tags (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       name VARCHAR(50) NOT NULL UNIQUE,
       color VARCHAR(7),
       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );
   
   -- 创建 ticket_tags 关联表
   CREATE TABLE ticket_tags (
       ticket_id UUID NOT NULL REFERENCES tickets(id) ON DELETE CASCADE,
       tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
       PRIMARY KEY (ticket_id, tag_id)
   );
   
   -- 创建索引
   CREATE INDEX idx_tickets_title_gin ON tickets USING gin(title gin_trgm_ops);
   CREATE INDEX idx_ticket_tags_ticket_id ON ticket_tags(ticket_id);
   CREATE INDEX idx_ticket_tags_tag_id ON ticket_tags(tag_id);
   
   -- 创建更新时间触发器
   CREATE OR REPLACE FUNCTION update_updated_at_column()
   RETURNS TRIGGER AS $$
   BEGIN
       NEW.updated_at = NOW();
       RETURN NEW;
   END;
   $$ language 'plpgsql';
   
   CREATE TRIGGER update_tickets_updated_at 
       BEFORE UPDATE ON tickets 
       FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
   ```

#### 3.1.5 错误处理
1. 创建错误类型 `src/utils/error.rs`
   ```rust
   use axum::{
       http::StatusCode,
       response::{IntoResponse, Response},
       Json,
   };
   use serde_json::json;
   use thiserror::Error;
   
   #[derive(Error, Debug)]
   pub enum AppError {
       #[error("Database error: {0}")]
       Database(#[from] sqlx::Error),
       
       #[error("Not found: {0}")]
       NotFound(String),
       
       #[error("Validation error: {0}")]
       Validation(String),
       
       #[error("Internal server error: {0}")]
       Internal(String),
   }
   
   impl IntoResponse for AppError {
       fn into_response(self) -> Response {
           let (status, error_message) = match self {
               AppError::Database(err) => {
                   tracing::error!("Database error: {:?}", err);
                   (StatusCode::INTERNAL_SERVER_ERROR, "数据库错误")
               }
               AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.as_str()),
               AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
               AppError::Internal(msg) => {
                   tracing::error!("Internal error: {}", msg);
                   (StatusCode::INTERNAL_SERVER_ERROR, "内部服务器错误")
               }
           };
           
           let body = Json(json!({
               "error": error_message,
               "status": status.as_u16()
           }));
           
           (status, body).into_response()
       }
   }
   
   pub type Result<T> = std::result::Result<T, AppError>;
   ```

### 3.2 第二阶段：核心功能实现（预计 3-4 天）

#### 3.2.1 数据访问层
1. 创建 Ticket Repository `src/repositories/ticket_repository.rs`
   ```rust
   use super::AppError;
   use crate::models::{Ticket, CreateTicketRequest, UpdateTicketRequest, TicketWithTags, Tag};
   use sqlx::PgPool;
   use uuid::Uuid;
   
   pub struct TicketRepository {
       pool: PgPool,
   }
   
   impl TicketRepository {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
       
       pub async fn find_all(
           &self,
           tag_id: Option<Uuid>,
           search: Option<String>,
           completed: Option<bool>,
       ) -> Result<Vec<TicketWithTags>> {
           // 实现查询逻辑
       }
       
       pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TicketWithTags>> {
           // 实现根据ID查询逻辑
       }
       
       pub async fn create(&self, request: CreateTicketRequest) -> Result<Ticket> {
           // 实现创建逻辑
       }
       
       pub async fn update(&self, id: Uuid, request: UpdateTicketRequest) -> Result<Ticket> {
           // 实现更新逻辑
       }
       
       pub async fn delete(&self, id: Uuid) -> Result<()> {
           // 实现删除逻辑
       }
       
       pub async fn toggle_completed(&self, id: Uuid) -> Result<Ticket> {
           // 实现切换完成状态逻辑
       }
       
       pub async fn add_tag(&self, ticket_id: Uuid, tag_id: Uuid) -> Result<()> {
           // 实现添加标签逻辑
       }
       
       pub async fn remove_tag(&self, ticket_id: Uuid, tag_id: Uuid) -> Result<()> {
           // 实现移除标签逻辑
       }
   }
   ```

2. 创建 Tag Repository `src/repositories/tag_repository.rs`
   ```rust
   use super::AppError;
   use crate::models::{Tag, CreateTagRequest, UpdateTagRequest};
   use sqlx::PgPool;
   use uuid::Uuid;
   
   pub struct TagRepository {
       pool: PgPool,
   }
   
   impl TagRepository {
       pub fn new(pool: PgPool) -> Self {
           Self { pool }
       }
       
       pub async fn find_all(&self) -> Result<Vec<Tag>> {
           // 实现查询所有标签逻辑
       }
       
       pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Tag>> {
           // 实现根据ID查询逻辑
       }
       
       pub async fn find_by_name(&self, name: &str) -> Result<Option<Tag>> {
           // 实现根据名称查询逻辑
       }
       
       pub async fn create(&self, request: CreateTagRequest) -> Result<Tag> {
           // 实现创建标签逻辑
       }
       
       pub async fn update(&self, id: Uuid, request: UpdateTagRequest) -> Result<Tag> {
           // 实现更新标签逻辑
       }
       
       pub async fn delete(&self, id: Uuid) -> Result<()> {
           // 实现删除标签逻辑
       }
   }
   ```

3. 创建 Repository 模块 `src/repositories/mod.rs`
   ```rust
   use crate::utils::error::Result;
   use sqlx::PgPool;
   
   pub mod ticket_repository;
   pub mod tag_repository;
   
   pub use ticket_repository::TicketRepository;
   pub use tag_repository::TagRepository;
   
   pub struct Repositories {
       pub ticket: TicketRepository,
       pub tag: TagRepository,
   }
   
   impl Repositories {
       pub fn new(pool: PgPool) -> Self {
           Self {
               ticket: TicketRepository::new(pool.clone()),
               tag: TagRepository::new(pool),
           }
       }
   }
   ```

#### 3.2.2 API 处理器
1. 创建 Ticket Handler `src/handlers/ticket_handler.rs`
   ```rust
   use axum::{
       extract::{Path, Query, State},
       http::StatusCode,
       response::Json,
   };
   use serde::Deserialize;
   use uuid::Uuid;
   
   use crate::{
       models::{CreateTicketRequest, UpdateTicketRequest, TicketWithTags},
       repositories::Repositories,
       utils::error::{AppError, Result},
   };
   
   #[derive(Debug, Deserialize)]
   pub struct TicketQuery {
       pub tag: Option<Uuid>,
       pub search: Option<String>,
       pub completed: Option<bool>,
   }
   
   pub async fn get_tickets(
       Query(query): Query<TicketQuery>,
       State(repositories): State<Repositories>,
   ) -> Result<Json<Vec<TicketWithTags>>> {
       let tickets = repositories.ticket.find_all(query.tag, query.search, query.completed).await?;
       Ok(Json(tickets))
   }
   
   pub async fn get_ticket(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
   ) -> Result<Json<TicketWithTags>> {
       let ticket = repositories.ticket.find_by_id(id).await?
           .ok_or_else(|| AppError::NotFound(format!("Ticket with id {} not found", id)))?;
       Ok(Json(ticket))
   }
   
   pub async fn create_ticket(
       State(repositories): State<Repositories>,
       Json(request): Json<CreateTicketRequest>,
   ) -> Result<(StatusCode, Json<Ticket>)> {
       let ticket = repositories.ticket.create(request).await?;
       Ok((StatusCode::CREATED, Json(ticket)))
   }
   
   pub async fn update_ticket(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
       Json(request): Json<UpdateTicketRequest>,
   ) -> Result<Json<Ticket>> {
       let ticket = repositories.ticket.update(id, request).await?;
       Ok(Json(ticket))
   }
   
   pub async fn delete_ticket(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
   ) -> Result<StatusCode> {
       repositories.ticket.delete(id).await?;
       Ok(StatusCode::NO_CONTENT)
   }
   
   pub async fn toggle_ticket_completed(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
   ) -> Result<Json<Ticket>> {
       let ticket = repositories.ticket.toggle_completed(id).await?;
       Ok(Json(ticket))
   }
   
   pub async fn add_tag_to_ticket(
       Path((ticket_id, tag_id)): Path<(Uuid, Uuid)>,
       State(repositories): State<Repositories>,
   ) -> Result<StatusCode> {
       repositories.ticket.add_tag(ticket_id, tag_id).await?;
       Ok(StatusCode::NO_CONTENT)
   }
   
   pub async fn remove_tag_from_ticket(
       Path((ticket_id, tag_id)): Path<(Uuid, Uuid)>,
       State(repositories): State<Repositories>,
   ) -> Result<StatusCode> {
       repositories.ticket.remove_tag(ticket_id, tag_id).await?;
       Ok(StatusCode::NO_CONTENT)
   }
   ```

2. 创建 Tag Handler `src/handlers/tag_handler.rs`
   ```rust
   use axum::{
       extract::{Path, State},
       http::StatusCode,
       response::Json,
   };
   use uuid::Uuid;
   
   use crate::{
       models::{CreateTagRequest, UpdateTagRequest, Tag},
       repositories::Repositories,
       utils::error::{AppError, Result},
   };
   
   pub async fn get_tags(
       State(repositories): State<Repositories>,
   ) -> Result<Json<Vec<Tag>>> {
       let tags = repositories.tag.find_all().await?;
       Ok(Json(tags))
   }
   
   pub async fn get_tag(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
   ) -> Result<Json<Tag>> {
       let tag = repositories.tag.find_by_id(id).await?
           .ok_or_else(|| AppError::NotFound(format!("Tag with id {} not found", id)))?;
       Ok(Json(tag))
   }
   
   pub async fn create_tag(
       State(repositories): State<Repositories>,
       Json(request): Json<CreateTagRequest>,
   ) -> Result<(StatusCode, Json<Tag>)> {
       let tag = repositories.tag.create(request).await?;
       Ok((StatusCode::CREATED, Json(tag)))
   }
   
   pub async fn update_tag(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
       Json(request): Json<UpdateTagRequest>,
   ) -> Result<Json<Tag>> {
       let tag = repositories.tag.update(id, request).await?;
       Ok(Json(tag))
   }
   
   pub async fn delete_tag(
       Path(id): Path<Uuid>,
       State(repositories): State<Repositories>,
   ) -> Result<StatusCode> {
       repositories.tag.delete(id).await?;
       Ok(StatusCode::NO_CONTENT)
   }
   ```

3. 创建 Handler 模块 `src/handlers/mod.rs`
   ```rust
   pub mod ticket_handler;
   pub mod tag_handler;
   
   pub use ticket_handler::*;
   pub use tag_handler::*;
   ```

#### 3.2.3 路由设置
1. 创建路由模块 `src/routes/mod.rs`
   ```rust
   use axum::{
       routing::{get, post, put, delete, patch},
       Router,
   };
   
   use crate::handlers::*;
   use crate::repositories::Repositories;
   
   pub fn create_routes(repositories: Repositories) -> Router {
       Router::new()
           // Ticket 路由
           .route("/api/tickets", get(get_tickets).post(create_ticket))
           .route("/api/tickets/:id", get(get_ticket).put(update_ticket).delete(delete_ticket))
           .route("/api/tickets/:id/toggle", patch(toggle_ticket_completed))
           .route("/api/tickets/:id/tags/:tag_id", post(add_tag_to_ticket).delete(remove_tag_from_ticket))
           
           // Tag 路由
           .route("/api/tags", get(get_tags).post(create_tag))
           .route("/api/tags/:id", get(get_tag).put(update_tag).delete(delete_tag))
           
           .with_state(repositories)
   }
   ```

#### 3.2.4 主程序
1. 更新 `src/main.rs`
   ```rust
   use axum::Router;
   use dotenvy::dotenv;
   use std::net::SocketAddr;
   use tower_http::cors::{Any, CorsLayer};
   use tower_http::trace::TraceLayer;
   use tracing::info;
   use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
   
   mod config;
   mod handlers;
   mod models;
   mod repositories;
   mod routes;
   mod utils;
   
   use config::Config;
   use repositories::Repositories;
   use routes::create_routes;
   
   #[tokio::main]
   async fn main() -> Result<(), Box<dyn std::error::Error>> {
       // 加载环境变量
       dotenv().ok();
       
       // 初始化日志
       tracing_subscriber::registry()
           .with(tracing_subscriber::EnvFilter::new(
               std::env::var("RUST_LOG").unwrap_or_else(|_| "project_alpha_backend=debug".into()),
           ))
           .with(tracing_subscriber::fmt::layer())
           .init();
       
       // 加载配置
       let config = Config::from_env()?;
       info!("Loaded configuration: {:?}", config);
       
       // 连接数据库
       let pool = sqlx::PgPool::connect(&config.database.url).await?;
       info!("Connected to database");
       
       // 运行迁移
       sqlx::migrate!("./migrations").run(&pool).await?;
       info!("Database migrations completed");
       
       // 创建仓库
       let repositories = Repositories::new(pool);
       
       // 创建应用
       let app = Router::new()
           .merge(create_routes(repositories))
           .layer(
               CorsLayer::new()
                   .allow_origin(Any)
                   .allow_methods(Any)
                   .allow_headers(Any),
           )
           .layer(TraceLayer::new_for_http());
       
       // 启动服务器
       let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));
       info!("Starting server on {}", addr);
       
       let listener = tokio::net::TcpListener::bind(addr).await?;
       axum::serve(listener, app).await?;
       
       Ok(())
   }
   ```

## 4. 前端实现计划

### 4.1 第一阶段：基础架构搭建（预计 2-3 天）

#### 4.1.1 项目初始化
1. 创建 React + TypeScript 项目
   ```bash
   cd project-alpha
   npm create vite@latest frontend -- --template react-ts
   cd frontend
   ```

2. 安装依赖
   ```bash
   # UI 框架和样式
   npm install react react-dom
   npm install -D tailwindcss postcss autoprefixer
   npm install -D @types/react @types/react-dom
   
   # UI 组件库
   npm install class-variance-authority clsx tailwind-merge
   npm install lucide-react @radix-ui/react-dialog @radix-ui/react-dropdown-menu @radix-ui/react-select @radix-ui/react-toast
   
   # 状态管理
   npm install zustand
   
   # 路由
   npm install react-router-dom
   npm install -D @types/react-router-dom
   
   # HTTP 客户端
   npm install axios
   
   # 表单处理
   npm install react-hook-form @hookform/resolvers zod
   
   # 工具库
   npm install date-fns
   ```

3. 配置 Tailwind CSS
   ```bash
   npx tailwindcss init -p
   ```

4. 更新 `tailwind.config.js`
   ```js
   /** @type {import('tailwindcss').Config} */
   export default {
     content: [
       "./index.html",
       "./src/**/*.{js,ts,jsx,tsx}",
     ],
     theme: {
       extend: {
         colors: {
           border: "hsl(var(--border))",
           input: "hsl(var(--input))",
           ring: "hsl(var(--ring))",
           background: "hsl(var(--background))",
           foreground: "hsl(var(--foreground))",
           primary: {
             DEFAULT: "hsl(var(--primary))",
             foreground: "hsl(var(--primary-foreground))",
           },
           secondary: {
             DEFAULT: "hsl(var(--secondary))",
             foreground: "hsl(var(--secondary-foreground))",
           },
           destructive: {
             DEFAULT: "hsl(var(--destructive))",
             foreground: "hsl(var(--destructive-foreground))",
           },
           muted: {
             DEFAULT: "hsl(var(--muted))",
             foreground: "hsl(var(--muted-foreground))",
           },
           accent: {
             DEFAULT: "hsl(var(--accent))",
             foreground: "hsl(var(--accent-foreground))",
           },
           popover: {
             DEFAULT: "hsl(var(--popover))",
             foreground: "hsl(var(--popover-foreground))",
           },
           card: {
             DEFAULT: "hsl(var(--card))",
             foreground: "hsl(var(--card-foreground))",
           },
         },
         borderRadius: {
           lg: "var(--radius)",
           md: "calc(var(--radius) - 2px)",
           sm: "calc(var(--radius) - 4px)",
         },
       },
     },
     plugins: [],
   }
   ```

5. 更新 `src/index.css`
   ```css
   @tailwind base;
   @tailwind components;
   @tailwind utilities;
   
   @layer base {
     :root {
       --background: 0 0% 100%;
       --foreground: 222.2 84% 4.9%;
       --card: 0 0% 100%;
       --card-foreground: 222.2 84% 4.9%;
       --popover: 0 0% 100%;
       --popover-foreground: 222.2 84% 4.9%;
       --primary: 222.2 47.4% 11.2%;
       --primary-foreground: 210 40% 98%;
       --secondary: 210 40% 96%;
       --secondary-foreground: 222.2 84% 4.9%;
       --muted: 210 40% 96%;
       --muted-foreground: 215.4 16.3% 46.9%;
       --accent: 210 40% 96%;
       --accent-foreground: 222.2 84% 4.9%;
       --destructive: 0 84.2% 60.2%;
       --destructive-foreground: 210 40% 98%;
       --border: 214.3 31.8% 91.4%;
       --input: 214.3 31.8% 91.4%;
       --ring: 222.2 84% 4.9%;
       --radius: 0.5rem;
     }
     
     .dark {
       --background: 222.2 84% 4.9%;
       --foreground: 210 40% 98%;
       --card: 222.2 84% 4.9%;
       --card-foreground: 210 40% 98%;
       --popover: 222.2 84% 4.9%;
       --popover-foreground: 210 40% 98%;
       --primary: 210 40% 98%;
       --primary-foreground: 222.2 47.4% 11.2%;
       --secondary: 217.2 32.6% 17.5%;
       --secondary-foreground: 210 40% 98%;
       --muted: 217.2 32.6% 17.5%;
       --muted-foreground: 215 20.2% 65.1%;
       --accent: 217.2 32.6% 17.5%;
       --accent-foreground: 210 40% 98%;
       --destructive: 0 62.8% 30.6%;
       --destructive-foreground: 210 40% 98%;
       --border: 217.2 32.6% 17.5%;
       --input: 217.2 32.6% 17.5%;
       --ring: 212.7 26.8% 83.9%;
     }
   }
   
   @layer base {
     * {
       @apply border-border;
     }
     body {
       @apply bg-background text-foreground;
     }
   }
   ```

#### 4.1.2 项目结构
```bash
mkdir -p src/{components,pages,hooks,services,stores,types,utils,lib}
```

#### 4.1.3 类型定义
1. 创建 `src/types/index.ts`
   ```typescript
   export interface Ticket {
     id: string;
     title: string;
     description?: string;
     completed: boolean;
     createdAt: string;
     updatedAt: string;
     tags?: Tag[];
   }
   
   export interface Tag {
     id: string;
     name: string;
     color?: string;
     createdAt: string;
   }
   
   export interface CreateTicketRequest {
     title: string;
     description?: string;
     tagIds?: string[];
   }
   
   export interface UpdateTicketRequest {
     title?: string;
     description?: string;
     completed?: boolean;
   }
   
   export interface CreateTagRequest {
     name: string;
     color?: string;
   }
   
   export interface UpdateTagRequest {
     name?: string;
     color?: string;
   }
   
   export interface TicketQuery {
     tag?: string;
     search?: string;
     completed?: boolean;
   }
   ```

#### 4.1.4 API 客户端
1. 创建 `src/services/api.ts`
   ```typescript
   import axios from 'axios';
   import { Ticket, Tag, CreateTicketRequest, UpdateTicketRequest, CreateTagRequest, UpdateTagRequest, TicketQuery } from '../types';
   
   const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000/api';
   
   const apiClient = axios.create({
     baseURL: API_BASE_URL,
     headers: {
       'Content-Type': 'application/json',
     },
   });
   
   // Ticket API
   export const ticketApi = {
     getAll: async (query?: TicketQuery): Promise<Ticket[]> => {
       const response = await apiClient.get('/tickets', { params: query });
       return response.data;
     },
     
     getById: async (id: string): Promise<Ticket> => {
       const response = await apiClient.get(`/tickets/${id}`);
       return response.data;
     },
     
     create: async (data: CreateTicketRequest): Promise<Ticket> => {
       const response = await apiClient.post('/tickets', data);
       return response.data;
     },
     
     update: async (id: string, data: UpdateTicketRequest): Promise<Ticket> => {
       const response = await apiClient.put(`/tickets/${id}`, data);
       return response.data;
     },
     
     delete: async (id: string): Promise<void> => {
       await apiClient.delete(`/tickets/${id}`);
     },
     
     toggleCompleted: async (id: string): Promise<Ticket> => {
       const response = await apiClient.patch(`/tickets/${id}/toggle`);
       return response.data;
     },
     
     addTag: async (ticketId: string, tagId: string): Promise<void> => {
       await apiClient.post(`/tickets/${ticketId}/tags/${tagId}`);
     },
     
     removeTag: async (ticketId: string, tagId: string): Promise<void> => {
       await apiClient.delete(`/tickets/${ticketId}/tags/${tagId}`);
     },
   };
   
   // Tag API
   export const tagApi = {
     getAll: async (): Promise<Tag[]> => {
       const response = await apiClient.get('/tags');
       return response.data;
     },
     
     getById: async (id: string): Promise<Tag> => {
       const response = await apiClient.get(`/tags/${id}`);
       return response.data;
     },
     
     create: async (data: CreateTagRequest): Promise<Tag> => {
       const response = await apiClient.post('/tags', data);
       return response.data;
     },
     
     update: async (id: string, data: UpdateTagRequest): Promise<Tag> => {
       const response = await apiClient.put(`/tags/${id}`, data);
       return response.data;
     },
     
     delete: async (id: string): Promise<void> => {
       await apiClient.delete(`/tags/${id}`);
     },
   };
   ```

#### 4.1.5 状态管理
1. 创建 `src/stores/ticketStore.ts`
   ```typescript
   import { create } from 'zustand';
   import { devtools } from 'zustand/middleware';
   import { Ticket, CreateTicketRequest, UpdateTicketRequest, TicketQuery } from '../types';
   import { ticketApi } from '../services/api';
   
   interface TicketState {
     tickets: Ticket[];
     loading: boolean;
     error: string | null;
     
     // Actions
     fetchTickets: (query?: TicketQuery) => Promise<void>;
     fetchTicket: (id: string) => Promise<void>;
     createTicket: (data: CreateTicketRequest) => Promise<void>;
     updateTicket: (id: string, data: UpdateTicketRequest) => Promise<void>;
     deleteTicket: (id: string) => Promise<void>;
     toggleTicketCompleted: (id: string) => Promise<void>;
     addTagToTicket: (ticketId: string, tagId: string) => Promise<void>;
     removeTagFromTicket: (ticketId: string, tagId: string) => Promise<void>;
     clearError: () => void;
   }
   
   export const useTicketStore = create<TicketState>()(
     devtools(
       (set, get) => ({
         tickets: [],
         loading: false,
         error: null,
         
         fetchTickets: async (query) => {
           set({ loading: true, error: null });
           try {
             const tickets = await ticketApi.getAll(query);
             set({ tickets, loading: false });
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to fetch tickets',
               loading: false 
             });
           }
         },
         
         fetchTicket: async (id) => {
           set({ loading: true, error: null });
           try {
             const ticket = await ticketApi.getById(id);
             set(state => ({
               tickets: state.tickets.some(t => t.id === ticket.id)
                 ? state.tickets.map(t => t.id === ticket.id ? ticket : t)
                 : [...state.tickets, ticket],
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to fetch ticket',
               loading: false 
             });
           }
         },
         
         createTicket: async (data) => {
           set({ loading: true, error: null });
           try {
             const ticket = await ticketApi.create(data);
             set(state => ({
               tickets: [...state.tickets, ticket],
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to create ticket',
               loading: false 
             });
           }
         },
         
         updateTicket: async (id, data) => {
           set({ loading: true, error: null });
           try {
             const updatedTicket = await ticketApi.update(id, data);
             set(state => ({
               tickets: state.tickets.map(t => t.id === id ? updatedTicket : t),
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to update ticket',
               loading: false 
             });
           }
         },
         
         deleteTicket: async (id) => {
           set({ loading: true, error: null });
           try {
             await ticketApi.delete(id);
             set(state => ({
               tickets: state.tickets.filter(t => t.id !== id),
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to delete ticket',
               loading: false 
             });
           }
         },
         
         toggleTicketCompleted: async (id) => {
           set({ loading: true, error: null });
           try {
             const updatedTicket = await ticketApi.toggleCompleted(id);
             set(state => ({
               tickets: state.tickets.map(t => t.id === id ? updatedTicket : t),
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to toggle ticket completion',
               loading: false 
             });
           }
         },
         
         addTagToTicket: async (ticketId, tagId) => {
           set({ loading: true, error: null });
           try {
             await ticketApi.addTag(ticketId, tagId);
             // 重新获取该 ticket 以获取最新数据
             await get().fetchTicket(ticketId);
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to add tag to ticket',
               loading: false 
             });
           }
         },
         
         removeTagFromTicket: async (ticketId, tagId) => {
           set({ loading: true, error: null });
           try {
             await ticketApi.removeTag(ticketId, tagId);
             // 重新获取该 ticket 以获取最新数据
             await get().fetchTicket(ticketId);
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to remove tag from ticket',
               loading: false 
             });
           }
         },
         
         clearError: () => set({ error: null }),
       }),
       {
         name: 'ticket-store',
       }
     )
   );
   ```

2. 创建 `src/stores/tagStore.ts`
   ```typescript
   import { create } from 'zustand';
   import { devtools } from 'zustand/middleware';
   import { Tag, CreateTagRequest, UpdateTagRequest } from '../types';
   import { tagApi } from '../services/api';
   
   interface TagState {
     tags: Tag[];
     loading: boolean;
     error: string | null;
     
     // Actions
     fetchTags: () => Promise<void>;
     fetchTag: (id: string) => Promise<void>;
     createTag: (data: CreateTagRequest) => Promise<void>;
     updateTag: (id: string, data: UpdateTagRequest) => Promise<void>;
     deleteTag: (id: string) => Promise<void>;
     clearError: () => void;
   }
   
   export const useTagStore = create<TagState>()(
     devtools(
       (set, get) => ({
         tags: [],
         loading: false,
         error: null,
         
         fetchTags: async () => {
           set({ loading: true, error: null });
           try {
             const tags = await tagApi.getAll();
             set({ tags, loading: false });
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to fetch tags',
               loading: false 
             });
           }
         },
         
         fetchTag: async (id) => {
           set({ loading: true, error: null });
           try {
             const tag = await tagApi.getById(id);
             set(state => ({
               tags: state.tags.some(t => t.id === tag.id)
                 ? state.tags.map(t => t.id === tag.id ? tag : t)
                 : [...state.tags, tag],
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to fetch tag',
               loading: false 
             });
           }
         },
         
         createTag: async (data) => {
           set({ loading: true, error: null });
           try {
             const tag = await tagApi.create(data);
             set(state => ({
               tags: [...state.tags, tag],
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to create tag',
               loading: false 
             });
           }
         },
         
         updateTag: async (id, data) => {
           set({ loading: true, error: null });
           try {
             const updatedTag = await tagApi.update(id, data);
             set(state => ({
               tags: state.tags.map(t => t.id === id ? updatedTag : t),
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to update tag',
               loading: false 
             });
           }
         },
         
         deleteTag: async (id) => {
           set({ loading: true, error: null });
           try {
             await tagApi.delete(id);
             set(state => ({
               tags: state.tags.filter(t => t.id !== id),
               loading: false
             }));
           } catch (error) {
             set({ 
               error: error instanceof Error ? error.message : 'Failed to delete tag',
               loading: false 
             });
           }
         },
         
         clearError: () => set({ error: null }),
       }),
       {
         name: 'tag-store',
       }
     )
   );
   ```

3. 创建 `src/stores/uiStore.ts`
   ```typescript
   import { create } from 'zustand';
   import { devtools, persist } from 'zustand/middleware';
   import { TicketQuery } from '../types';
   
   interface UIState {
     // 筛选和搜索状态
     searchQuery: string;
     selectedTagId: string | null;
     showCompleted: boolean | null;
     
     // UI 状态
     sidebarOpen: boolean;
     ticketFormOpen: boolean;
     tagManagerOpen: boolean;
     editingTicketId: string | null;
     
     // Actions
     setSearchQuery: (query: string) => void;
     setSelectedTagId: (tagId: string | null) => void;
     setShowCompleted: (show: boolean | null) => void;
     toggleSidebar: () => void;
     setSidebarOpen: (open: boolean) => void;
     openTicketForm: (ticketId?: string) => void;
     closeTicketForm: () => void;
     toggleTagManager: () => void;
     setTagManagerOpen: (open: boolean) => void;
     
     // 获取当前查询参数
     getTicketQuery: () => TicketQuery;
     resetFilters: () => void;
   }
   
   export const useUIStore = create<UIState>()(
     devtools(
       persist(
         (set, get) => ({
           // 筛选和搜索状态
           searchQuery: '',
           selectedTagId: null,
           showCompleted: null,
           
           // UI 状态
           sidebarOpen: true,
           ticketFormOpen: false,
           tagManagerOpen: false,
           editingTicketId: null,
           
           // Actions
           setSearchQuery: (query) => set({ searchQuery: query }),
           setSelectedTagId: (tagId) => set({ selectedTagId: tagId }),
           setShowCompleted: (show) => set({ showCompleted: show }),
           toggleSidebar: () => set((state) => ({ sidebarOpen: !state.sidebarOpen })),
           setSidebarOpen: (open) => set({ sidebarOpen: open }),
           openTicketForm: (ticketId) => set({ 
             ticketFormOpen: true, 
             editingTicketId: ticketId || null 
           }),
           closeTicketForm: () => set({ 
             ticketFormOpen: false, 
             editingTicketId: null 
           }),
           toggleTagManager: () => set((state) => ({ tagManagerOpen: !state.tagManagerOpen })),
           setTagManagerOpen: (open) => set({ tagManagerOpen: open }),
           
           // 获取当前查询参数
           getTicketQuery: () => {
             const { searchQuery, selectedTagId, showCompleted } = get();
             const query: TicketQuery = {};
             
             if (searchQuery.trim()) {
               query.search = searchQuery.trim();
             }
             
             if (selectedTagId) {
               query.tag = selectedTagId;
             }
             
             if (showCompleted !== null) {
               query.completed = showCompleted;
             }
             
             return query;
           },
           
           resetFilters: () => set({
             searchQuery: '',
             selectedTagId: null,
             showCompleted: null,
           }),
         }),
         {
           name: 'ui-store',
           partialize: (state) => ({
             searchQuery: state.searchQuery,
             selectedTagId: state.selectedTagId,
             showCompleted: state.showCompleted,
             sidebarOpen: state.sidebarOpen,
           }),
         }
       )
     )
   );
   ```

4. 创建 `src/stores/index.ts`
   ```typescript
   export { useTicketStore } from './ticketStore';
   export { useTagStore } from './tagStore';
   export { useUIStore } from './uiStore';
   ```

### 4.2 第二阶段：组件开发（预计 4-5 天）

#### 4.2.1 基础 UI 组件
1. 创建 `src/lib/utils.ts`
   ```typescript
   import { type ClassValue, clsx } from "clsx"
   import { twMerge } from "tailwind-merge"
   
   export function cn(...inputs: ClassValue[]) {
     return twMerge(clsx(inputs))
   }
   ```

2. 创建 `src/components/ui/button.tsx`
   ```typescript
   import * as React from "react"
   import { Slot } from "@radix-ui/react-slot"
   import { cva, type VariantProps } from "class-variance-authority"
   
   import { cn } from "@/lib/utils"
   
   const buttonVariants = cva(
     "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium ring-offset-background transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none disabled:opacity-50",
     {
       variants: {
         variant: {
           default: "bg-primary text-primary-foreground hover:bg-primary/90",
           destructive:
             "bg-destructive text-destructive-foreground hover:bg-destructive/90",
           outline:
             "border border-input bg-background hover:bg-accent hover:text-accent-foreground",
           secondary:
             "bg-secondary text-secondary-foreground hover:bg-secondary/80",
           ghost: "hover:bg-accent hover:text-accent-foreground",
           link: "text-primary underline-offset-4 hover:underline",
         },
         size: {
           default: "h-10 px-4 py-2",
           sm: "h-9 rounded-md px-3",
           lg: "h-11 rounded-md px-8",
           icon: "h-10 w-10",
         },
       },
       defaultVariants: {
         variant: "default",
         size: "default",
       },
     }
   )
   
   export interface ButtonProps
     extends React.ButtonHTMLAttributes<HTMLButtonElement>,
       VariantProps<typeof buttonVariants> {
     asChild?: boolean
   }
   
   const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
     ({ className, variant, size, asChild = false, ...props }, ref) => {
       const Comp = asChild ? Slot : "button"
       return (
         <Comp
           className={cn(buttonVariants({ variant, size, className }))}
           ref={ref}
           {...props}
         />
       )
     }
   )
   Button.displayName = "Button"
   
   export { Button, buttonVariants }
   ```

3. 创建其他基础 UI 组件（input, card, label, select, dialog 等）

#### 4.2.2 业务组件
1. 创建 `src/components/TicketCard.tsx`
   ```typescript
   import React from 'react';
   import { format } from 'date-fns';
   import { Check, Edit, Trash2, Tag } from 'lucide-react';
   import { Ticket } from '../types';
   import { Button } from './ui/button';
   import { Badge } from './ui/badge';
   import { Card, CardContent, CardFooter, CardHeader } from './ui/card';
   import { useTicketStore } from '../stores';
   
   interface TicketCardProps {
     ticket: Ticket;
     onEdit: (ticket: Ticket) => void;
   }
   
   export const TicketCard: React.FC<TicketCardProps> = ({ ticket, onEdit }) => {
     const { toggleTicketCompleted, deleteTicket } = useTicketStore();
     
     const handleToggleCompleted = async () => {
       await toggleTicketCompleted(ticket.id);
     };
     
     const handleDelete = async () => {
       if (window.confirm('确定要删除这个 ticket 吗？')) {
         await deleteTicket(ticket.id);
       }
     };
     
     return (
       <Card className={`w-full ${ticket.completed ? 'opacity-75' : ''}`}>
         <CardHeader className="pb-2">
           <div className="flex justify-between items-start">
             <h3 className={`text-lg font-semibold ${ticket.completed ? 'line-through' : ''}`}>
               {ticket.title}
             </h3>
             <div className="flex space-x-1">
               <Button
                 variant="ghost"
                 size="icon"
                 onClick={handleToggleCompleted}
                 className={ticket.completed ? 'text-green-600' : ''}
               >
                 <Check className="h-4 w-4" />
               </Button>
               <Button variant="ghost" size="icon" onClick={() => onEdit(ticket)}>
                 <Edit className="h-4 w-4" />
               </Button>
               <Button variant="ghost" size="icon" onClick={handleDelete}>
                 <Trash2 className="h-4 w-4" />
               </Button>
             </div>
           </div>
         </CardHeader>
         
         <CardContent className="pb-2">
           {ticket.description && (
             <p className="text-sm text-muted-foreground mb-2">
               {ticket.description}
             </p>
           )}
           
           {ticket.tags && ticket.tags.length > 0 && (
             <div className="flex flex-wrap gap-1 mb-2">
               {ticket.tags.map(tag => (
                 <Badge key={tag.id} variant="secondary" className="text-xs">
                   {tag.name}
                 </Badge>
               ))}
             </div>
           )}
         </CardContent>
         
         <CardFooter className="pt-0">
           <p className="text-xs text-muted-foreground">
             创建于 {format(new Date(ticket.createdAt), 'yyyy-MM-dd HH:mm')}
           </p>
         </CardFooter>
       </Card>
     );
   };
   ```

2. 创建 `src/components/TicketList.tsx`
   ```typescript
   import React, { useEffect } from 'react';
   import { Ticket } from '../types';
   import { TicketCard } from './TicketCard';
   import { useTicketStore, useUIStore } from '../stores';
   import { Button } from './ui/button';
   import { Plus } from 'lucide-react';
   
   export const TicketList: React.FC = () => {
     const { tickets, loading, error, fetchTickets } = useTicketStore();
     const { getTicketQuery, openTicketForm } = useUIStore();
     
     useEffect(() => {
       fetchTickets(getTicketQuery());
     }, [fetchTickets, getTicketQuery]);
     
     const handleEditTicket = (ticket: Ticket) => {
       openTicketForm(ticket.id);
     };
     
     const handleCreateTicket = () => {
       openTicketForm();
     };
     
     if (loading) {
       return (
         <div className="flex justify-center items-center h-64">
           <div className="text-lg">加载中...</div>
         </div>
       );
     }
     
     if (error) {
       return (
         <div className="flex flex-col items-center justify-center h-64">
           <div className="text-lg text-destructive mb-4">错误: {error}</div>
           <Button onClick={() => fetchTickets(getTicketQuery())}>
             重试
           </Button>
         </div>
       );
     }
     
     return (
       <div className="space-y-4">
         <div className="flex justify-between items-center">
           <h2 className="text-2xl font-bold">Tickets</h2>
           <Button onClick={handleCreateTicket}>
             <Plus className="mr-2 h-4 w-4" />
             新建 Ticket
           </Button>
         </div>
         
         {tickets.length === 0 ? (
           <div className="text-center py-12">
             <p className="text-lg text-muted-foreground mb-4">没有找到 tickets</p>
             <Button onClick={handleCreateTicket}>
               <Plus className="mr-2 h-4 w-4" />
               创建第一个 Ticket
             </Button>
           </div>
         ) : (
           <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
             {tickets.map(ticket => (
               <TicketCard
                 key={ticket.id}
                 ticket={ticket}
                 onEdit={handleEditTicket}
               />
             ))}
           </div>
         )}
       </div>
     );
   };
   ```

3. 创建 `src/components/TicketForm.tsx`
   ```typescript
   import React, { useEffect } from 'react';
   import { useForm } from 'react-hook-form';
   import { zodResolver } from '@hookform/resolvers/zod';
   import * as z from 'zod';
   import { Ticket, CreateTicketRequest, UpdateTicketRequest } from '../types';
   import { useTicketStore, useUIStore } from '../stores';
   import { Button } from './ui/button';
   import { Input } from './ui/input';
   import { Textarea } from './ui/textarea';
   import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from './ui/form';
   import { Dialog, DialogContent, DialogHeader, DialogTitle } from './ui/dialog';
   import { TagSelector } from './TagSelector';
   
   const ticketSchema = z.object({
     title: z.string().min(1, '标题不能为空'),
     description: z.string().optional(),
     tagIds: z.array(z.string()).optional(),
   });
   
   type TicketFormData = z.infer<typeof ticketSchema>;
   
   export const TicketForm: React.FC = () => {
     const { createTicket, updateTicket, tickets } = useTicketStore();
     const { ticketFormOpen, editingTicketId, closeTicketForm } = useUIStore();
     
     const isEditing = !!editingTicketId;
     const editingTicket = isEditing 
       ? tickets.find(t => t.id === editingTicketId)
       : null;
     
     const form = useForm<TicketFormData>({
       resolver: zodResolver(ticketSchema),
       defaultValues: {
         title: '',
         description: '',
         tagIds: [],
       },
     });
     
     useEffect(() => {
       if (isEditing && editingTicket) {
         form.reset({
           title: editingTicket.title,
           description: editingTicket.description || '',
           tagIds: editingTicket.tags?.map(tag => tag.id) || [],
         });
       } else {
         form.reset({
           title: '',
           description: '',
           tagIds: [],
         });
       }
     }, [isEditing, editingTicket, form]);
     
     const onSubmit = async (data: TicketFormData) => {
       try {
         if (isEditing && editingTicketId) {
           const updateData: UpdateTicketRequest = {
             title: data.title,
             description: data.description,
           };
           await updateTicket(editingTicketId, updateData);
         } else {
           const createData: CreateTicketRequest = {
             title: data.title,
             description: data.description,
             tagIds: data.tagIds,
           };
           await createTicket(createData);
         }
         closeTicketForm();
       } catch (error) {
         console.error('Failed to save ticket:', error);
       }
     };
     
     return (
       <Dialog open={ticketFormOpen} onOpenChange={closeTicketForm}>
         <DialogContent className="sm:max-w-[425px]">
           <DialogHeader>
             <DialogTitle>{isEditing ? '编辑 Ticket' : '创建 Ticket'}</DialogTitle>
           </DialogHeader>
           
           <Form {...form}>
             <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
               <FormField
                 control={form.control}
                 name="title"
                 render={({ field }) => (
                   <FormItem>
                     <FormLabel>标题</FormLabel>
                     <FormControl>
                       <Input placeholder="输入 ticket 标题" {...field} />
                     </FormControl>
                     <FormMessage />
                   </FormItem>
                 )}
               />
               
               <FormField
                 control={form.control}
                 name="description"
                 render={({ field }) => (
                   <FormItem>
                     <FormLabel>描述</FormLabel>
                     <FormControl>
                       <Textarea 
                         placeholder="输入 ticket 描述" 
                         className="resize-none" 
                         {...field} 
                       />
                     </FormControl>
                     <FormMessage />
                   </FormItem>
                 )}
               />
               
               {!isEditing && (
                 <FormField
                   control={form.control}
                   name="tagIds"
                   render={({ field }) => (
                     <FormItem>
                       <FormLabel>标签</FormLabel>
                       <FormControl>
                         <TagSelector
                           value={field.value || []}
                           onChange={field.onChange}
                         />
                       </FormControl>
                       <FormMessage />
                     </FormItem>
                   )}
                 />
               )}
               
               <div className="flex justify-end space-x-2">
                 <Button type="button" variant="outline" onClick={closeTicketForm}>
                   取消
                 </Button>
                 <Button type="submit">
                   {isEditing ? '更新' : '创建'}
                 </Button>
               </div>
             </form>
           </Form>
         </DialogContent>
       </Dialog>
     );
   };
   ```

4. 创建其他业务组件（TagSelector, TagManager, SearchBar, FilterPanel 等）

#### 4.2.3 页面组件
1. 创建 `src/pages/TicketsPage.tsx`
   ```typescript
   import React from 'react';
   import { TicketList } from '../components/TicketList';
   import { TicketForm } from '../components/TicketForm';
   import { Sidebar } from '../components/Sidebar';
   
   export const TicketsPage: React.FC = () => {
     return (
       <div className="flex h-screen bg-background">
         <Sidebar />
         <main className="flex-1 overflow-auto p-6">
           <TicketList />
           <TicketForm />
         </main>
       </div>
     );
   };
   ```

2. 创建 `src/pages/TagsPage.tsx`
   ```typescript
   import React from 'react';
   import { TagManager } from '../components/TagManager';
   
   export const TagsPage: React.FC = () => {
     return (
       <div className="container mx-auto py-6">
         <TagManager />
       </div>
     );
   };
   ```

#### 4.2.4 路由设置
1. 更新 `src/App.tsx`
   ```typescript
   import React from 'react';
   import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
   import { TicketsPage } from './pages/TicketsPage';
   import { TagsPage } from './pages/TagsPage';
   import { Layout } from './components/Layout';
   
   function App() {
     return (
       <Router>
         <Layout>
           <Routes>
             <Route path="/" element={<TicketsPage />} />
             <Route path="/tags" element={<TagsPage />} />
           </Routes>
         </Layout>
       </Router>
     );
   }
   
   export default App;
   ```

2. 创建 `src/components/Layout.tsx`
   ```typescript
   import React from 'react';
   import { Link, useLocation } from 'react-router-dom';
   import { cn } from '../lib/utils';
   
   interface LayoutProps {
     children: React.ReactNode;
   }
   
   export const Layout: React.FC<LayoutProps> = ({ children }) => {
     const location = useLocation();
     
     const isActive = (path: string) => {
       return location.pathname === path;
     };
     
     return (
       <div className="min-h-screen bg-background">
         <header className="border-b">
           <div className="container mx-auto px-4 py-4">
             <nav className="flex space-x-6">
               <Link
                 to="/"
                 className={cn(
                   "text-sm font-medium transition-colors hover:text-primary",
                   isActive("/") ? "text-foreground" : "text-muted-foreground"
                 )}
               >
                 Tickets
               </Link>
               <Link
                 to="/tags"
                 className={cn(
                   "text-sm font-medium transition-colors hover:text-primary",
                   isActive("/tags") ? "text-foreground" : "text-muted-foreground"
                 )}
               >
                 标签管理
               </Link>
             </nav>
           </div>
         </header>
         
         <main>{children}</main>
       </div>
     );
   };
   ```

## 5. 数据库实现计划

### 5.1 数据库设置（预计 1 天）

1. 安装 PostgreSQL 15+
2. 创建数据库
   ```sql
   CREATE DATABASE project_alpha;
   ```

3. 创建数据库用户（可选）
   ```sql
   CREATE USER project_alpha_user WITH PASSWORD 'your_password';
   GRANT ALL PRIVILEGES ON DATABASE project_alpha TO project_alpha_user;
   ```

4. 启用 pg_trgm 扩展（用于模糊搜索）
   ```sql
   \c project_alpha;
   CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
   CREATE EXTENSION IF NOT EXISTS "pg_trgm";
   ```

### 5.2 数据库迁移（已在后端计划中包含）

## 6. 部署和测试计划

### 6.1 开发环境设置（预计 1 天）

1. 创建 Docker Compose 配置 `docker-compose.yml`
   ```yaml
   version: '3.8'
   
   services:
     postgres:
       image: postgres:15
       container_name: project_alpha_db
       environment:
         POSTGRES_DB: project_alpha
         POSTGRES_USER: project_alpha_user
         POSTGRES_PASSWORD: password
       ports:
         - "5432:5432"
       volumes:
         - postgres_data:/var/lib/postgresql/data
   
     backend:
       build:
         context: ./backend
         dockerfile: Dockerfile.dev
       container_name: project_alpha_backend
       environment:
         DATABASE_URL: postgresql://project_alpha_user:password@postgres:5432/project_alpha
         HOST: 0.0.0.0
         PORT: 3000
         RUST_LOG: debug
       ports:
         - "3000:3000"
       volumes:
         - ./backend:/app
       depends_on:
         - postgres
       command: cargo run
   
     frontend:
       build:
         context: ./frontend
         dockerfile: Dockerfile.dev
       container_name: project_alpha_frontend
       environment:
         VITE_API_BASE_URL: http://localhost:3000/api
       ports:
         - "5173:5173"
       volumes:
         - ./frontend:/app
       depends_on:
         - backend
       command: npm run dev
   
   volumes:
     postgres_data:
   ```

2. 创建后端开发 Dockerfile `backend/Dockerfile.dev`
   ```dockerfile
   FROM rust:1.75
   
   WORKDIR /app
   
   # 安装系统依赖
   RUN apt-get update && apt-get install -y \
       pkg-config \
       libssl-dev \
       && rm -rf /var/lib/apt/lists/*
   
   # 安装 SQLx CLI
   RUN cargo install sqlx-cli --no-default-features --features native-tls,postgres
   
   # 复制 Cargo 文件
   COPY Cargo.toml Cargo.lock ./
   
   # 创建虚拟 main.rs 以缓存依赖
   RUN mkdir src && echo "fn main() {}" > src/main.rs
   RUN cargo build --release
   RUN rm -rf src
   
   # 复制源代码
   COPY . .
   
   # 构建应用
   RUN cargo build
   ```

3. 创建前端开发 Dockerfile `frontend/Dockerfile.dev`
   ```dockerfile
   FROM node:18-alpine
   
   WORKDIR /app
   
   # 复制 package 文件
   COPY package.json package-lock.json ./
   
   # 安装依赖
   RUN npm ci
   
   # 复制源代码
   COPY . .
   
   # 暴露端口
   EXPOSE 5173
   
   # 启动开发服务器
   CMD ["npm", "run", "dev", "--", "--host", "0.0.0.0"]
   ```

### 6.2 测试计划（预计 2-3 天）

#### 6.2.1 后端测试
1. 单元测试
   - 模型测试
   - 仓库测试
   - 服务测试

2. 集成测试
   - API 端点测试

#### 6.2.2 前端测试
1. 单元测试
   - 组件测试
   - 状态管理测试

2. 集成测试
   - 页面测试
   - 用户交互测试

#### 6.2.3 E2E 测试
1. 使用 Playwright 或 Cypress 进行端到端测试

### 6.3 生产环境部署（预计 1-2 天）

1. 创建生产环境 Dockerfile
2. 设置 CI/CD 流程
3. 配置生产环境数据库
4. 部署到云服务提供商

## 7. 时间安排

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| 1 | 后端基础架构搭建 | 2-3 天 |
| 2 | 后端核心功能实现 | 3-4 天 |
| 3 | 前端基础架构搭建 | 2-3 天 |
| 4 | 前端组件开发 | 4-5 天 |
| 5 | 数据库设置 | 1 天 |
| 6 | 开发环境设置 | 1 天 |
| 7 | 测试 | 2-3 天 |
| 8 | 生产环境部署 | 1-2 天 |
| **总计** | | **16-21 天** |

## 8. 风险评估与缓解措施

### 8.1 技术风险
1. **Rust 学习曲线**
   - 风险: 团队对 Rust 不熟悉可能导致开发进度延迟
   - 缓解: 提前进行 Rust 技术培训，使用简单的代码结构

2. **前后端集成复杂性**
   - 风险: API 接口不匹配可能导致集成困难
   - 缓解: 早期定义清晰的 API 规范，使用 OpenAPI/Swagger 文档

3. **数据库性能问题**
   - 风险: 随着数据量增加，查询性能可能下降
   - 缓解: 合理设计索引，考虑分页和缓存策略

### 8.2 项目风险
1. **需求变更**
   - 风险: 开发过程中需求可能发生变化
   - 缓解: 采用敏捷开发方法，保持代码灵活性

2. **时间压力**
   - 风险: 项目时间可能不足以完成所有功能
   - 缓解: 优先实现核心功能，非关键功能可后续迭代

## 9. 后续优化计划

1. **性能优化**
   - 实现前端代码分割和懒加载
   - 添加后端缓存层
   - 优化数据库查询

2. **功能扩展**
   - 添加用户认证系统
   - 实现更高级的搜索功能
   - 添加数据导出功能

3. **用户体验优化**
   - 添加键盘快捷键支持
   - 实现拖拽排序功能
   - 添加更多动画和过渡效果

## 10. 总结

本实现计划详细描述了 Project Alpha 的开发过程，包括后端、前端、数据库和部署的各个方面。按照这个计划，团队可以在大约 3-4 周内完成一个功能完整的 Ticket 管理工具。计划中包含了详细的技术实现细节、代码示例和时间安排，可以作为开发团队的指导文档。