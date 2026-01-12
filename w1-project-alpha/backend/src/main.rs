use axum::Router;
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
mod services;
mod utils;

use config::Config;
use repositories::Repositories;
use routes::create_routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenvy::dotenv().ok();
    
    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "project_alpha_backend=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    // 加载配置
    let config = Config::from_env()?;
    info!("Loaded configuration: {:?}", config);
    
    // 连接数据库
    // Note: SQLx 0.7 doesn't support max_connections in connect(), 
    // but we keep the config field for future use
    let pool = sqlx::PgPool::connect(&config.database.url).await?;
    info!("Connected to database (configured max connections: {})", config.database.max_connections);
    
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
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .map_err(|e| format!("Invalid server address {}:{} - {}", config.server.host, config.server.port, e))?;
    info!("Starting server on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
