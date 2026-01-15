use axum::Router;
use once_cell::sync::Lazy;
use project_alpha_backend::{config::Config, repositories::Repositories, routes::create_routes};
use serde_json::json;
use std::sync::Once;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

static INIT: Once = Once::new();

// 全局测试锁，确保测试串行执行，避免数据竞争
// 每个测试函数在执行前会获取这个锁，执行完成后自动释放
static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

/// 初始化测试环境（只执行一次）
fn init_test_env() {
    INIT.call_once(|| {
        // 设置测试环境变量
        std::env::set_var("RUST_LOG", "error");
        std::env::set_var(
            "DATABASE_URL",
            "postgresql://postgres:123456@localhost:5432/aicom_w1_project_alpha",
        );
        std::env::set_var("HOST", "127.0.0.1");
        std::env::set_var("PORT", "0"); // 0 表示自动分配端口
    });
}

/// 测试服务器结构
struct TestServer {
    base_url: String,
    _handle: JoinHandle<()>,
}

impl TestServer {
    /// 启动测试服务器
    async fn start() -> Self {
        init_test_env();

        // 加载配置
        let config = Config::from_env().expect("Failed to load config");

        // 连接数据库
        let pool = sqlx::PgPool::connect(&config.database.url)
            .await
            .expect("Failed to connect to database");

        // 运行迁移
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // 清理测试数据（按正确顺序，使用 CASCADE 确保外键约束）
        sqlx::query("TRUNCATE TABLE ticket_tags RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await
            .ok();
        sqlx::query("TRUNCATE TABLE tickets RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await
            .ok();
        sqlx::query("TRUNCATE TABLE tags RESTART IDENTITY CASCADE")
            .execute(&pool)
            .await
            .ok();

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

        // 绑定到随机端口
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind to address");
        let addr = listener.local_addr().expect("Failed to get local address");

        let base_url = format!("http://{}", addr);

        // 启动服务器
        let handle = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Server failed");
        });

        // 等待服务器启动
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Self {
            base_url,
            _handle: handle,
        }
    }

    #[allow(dead_code)]
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

/// 测试客户端
struct TestClient {
    client: reqwest::Client,
    base_url: String,
}

impl TestClient {
    fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get(&self, path: &str) -> reqwest::Response {
        self.client
            .get(self.url(path))
            .send()
            .await
            .expect("Request failed")
    }

    async fn post(&self, path: &str, body: serde_json::Value) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .json(&body)
            .send()
            .await
            .expect("Request failed")
    }

    async fn post_empty(&self, path: &str) -> reqwest::Response {
        self.client
            .post(self.url(path))
            .send()
            .await
            .expect("Request failed")
    }

    async fn put(&self, path: &str, body: serde_json::Value) -> reqwest::Response {
        self.client
            .put(self.url(path))
            .json(&body)
            .send()
            .await
            .expect("Request failed")
    }

    async fn patch(&self, path: &str) -> reqwest::Response {
        self.client
            .patch(self.url(path))
            .send()
            .await
            .expect("Request failed")
    }

    async fn delete(&self, path: &str) -> reqwest::Response {
        self.client
            .delete(self.url(path))
            .send()
            .await
            .expect("Request failed")
    }
}

#[tokio::test]
async fn test_tag_crud() {
    // 获取全局测试锁，确保测试串行执行
    let _guard = TEST_MUTEX.lock().await;

    let server = TestServer::start().await;
    let client = TestClient::new(server.base_url.clone());

    // 1. 获取所有标签（应该为空）
    let resp = client.get("/api/tags").await;
    assert_eq!(resp.status(), 200);
    let tags: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tags.len(), 0);

    // 2. 创建第一个标签
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "urgent",
                "color": "#FF0000"
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let tag1: serde_json::Value = resp.json().await.unwrap();
    let tag1_id = tag1["id"].as_str().unwrap();
    assert_eq!(tag1["name"], "urgent");
    assert_eq!(tag1["color"], "#FF0000");

    // 3. 创建第二个标签
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "bug",
                "color": "#FF5733"
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let _tag2: serde_json::Value = resp.json().await.unwrap();

    // 4. 创建第三个标签（不带颜色）
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "feature",
                "color": null
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);

    // 5. 获取所有标签（应该有3个）
    let resp = client.get("/api/tags").await;
    assert_eq!(resp.status(), 200);
    let tags: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tags.len(), 3);

    // 6. 获取单个标签
    let resp = client.get(&format!("/api/tags/{}", tag1_id)).await;
    assert_eq!(resp.status(), 200);
    let tag: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(tag["id"], tag1_id);
    assert_eq!(tag["name"], "urgent");

    // 7. 更新标签（完整更新）
    let resp = client
        .put(
            &format!("/api/tags/{}", tag1_id),
            json!({
                "name": "urgent-updated",
                "color": "#CC0000"
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let tag: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(tag["name"], "urgent-updated");
    assert_eq!(tag["color"], "#CC0000");

    // 8. 更新标签（部分更新 - 只更新名称）
    let resp = client
        .put(
            &format!("/api/tags/{}", tag1_id),
            json!({
                "name": "urgent"
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let tag: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(tag["name"], "urgent");

    // 9. 更新标签（部分更新 - 只更新颜色）
    let resp = client
        .put(
            &format!("/api/tags/{}", tag1_id),
            json!({
                "color": "#FF0000"
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let tag: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(tag["color"], "#FF0000");

    // 10. 尝试创建重复标签（应该失败）
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "urgent",
                "color": "#FF0000"
            }),
        )
        .await;
    assert_eq!(resp.status(), 400);

    // 11. 获取不存在的标签（应该返回404）
    let resp = client
        .get("/api/tags/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(resp.status(), 404);

    // 12. 更新不存在的标签（应该返回404）
    let resp = client
        .put(
            "/api/tags/00000000-0000-0000-0000-000000000000",
            json!({
                "name": "test"
            }),
        )
        .await;
    assert_eq!(resp.status(), 404);

    // 13. 删除标签
    let resp = client.delete(&format!("/api/tags/{}", tag1_id)).await;
    assert_eq!(resp.status(), 204);

    // 14. 验证标签已删除
    let resp = client.get(&format!("/api/tags/{}", tag1_id)).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_ticket_crud() {
    // 获取全局测试锁，确保测试串行执行
    let _guard = TEST_MUTEX.lock().await;

    let server = TestServer::start().await;
    let client = TestClient::new(server.base_url.clone());

    // 先创建一些标签
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "urgent",
                "color": "#FF0000"
            }),
        )
        .await;
    let tag1: serde_json::Value = resp.json().await.unwrap();
    let tag1_id = tag1["id"].as_str().unwrap();

    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "bug",
                "color": "#FF5733"
            }),
        )
        .await;
    let tag2: serde_json::Value = resp.json().await.unwrap();
    let tag2_id = tag2["id"].as_str().unwrap();

    // 1. 获取所有 tickets（应该为空）
    let resp = client.get("/api/tickets").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 0);

    // 2. 创建 ticket（不带标签）
    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Fix login bug",
                "description": "Users cannot login with email",
                "tag_ids": null
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let ticket1: serde_json::Value = resp.json().await.unwrap();
    let ticket1_id = ticket1["id"].as_str().unwrap();
    assert_eq!(ticket1["title"], "Fix login bug");
    assert_eq!(ticket1["completed"], false);

    // 3. 创建 ticket（带标签）
    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Add dark mode",
                "description": "Implement dark mode theme",
                "tag_ids": [tag1_id, tag2_id]
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);
    let ticket2: serde_json::Value = resp.json().await.unwrap();
    let ticket2_id = ticket2["id"].as_str().unwrap();

    // 4. 创建 ticket（不带描述）
    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Update documentation",
                "description": null,
                "tag_ids": []
            }),
        )
        .await;
    assert_eq!(resp.status(), 201);

    // 5. 获取所有 tickets（应该有3个）
    let resp = client.get("/api/tickets").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 3);

    // 6. 获取单个 ticket
    let resp = client.get(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["id"].as_str().unwrap(), ticket1_id);
    assert_eq!(ticket["title"], "Fix login bug");
    assert_eq!(ticket["tags"].as_array().unwrap().len(), 0);

    // 7. 获取带标签的 ticket
    let resp = client.get(&format!("/api/tickets/{}", ticket2_id)).await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["tags"].as_array().unwrap().len(), 2);

    // 8. 更新 ticket（完整更新）
    let resp = client
        .put(
            &format!("/api/tickets/{}", ticket1_id),
            json!({
                "title": "Fix login bug - Updated",
                "description": "Users cannot login with email - Fixed",
                "completed": true
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["title"], "Fix login bug - Updated");
    assert_eq!(ticket["completed"], true);

    // 9. 更新 ticket（部分更新 - 只更新标题）
    let resp = client
        .put(
            &format!("/api/tickets/{}", ticket1_id),
            json!({
                "title": "Fix login bug - Final"
            }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["title"], "Fix login bug - Final");

    // 10. 切换完成状态
    let resp = client
        .patch(&format!("/api/tickets/{}/toggle", ticket1_id))
        .await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["completed"], false);

    // 11. 再次切换完成状态
    let resp = client
        .patch(&format!("/api/tickets/{}/toggle", ticket1_id))
        .await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["completed"], true);

    // 12. 添加标签到 ticket
    let resp = client
        .post_empty(&format!("/api/tickets/{}/tags/{}", ticket1_id, tag1_id))
        .await;
    assert_eq!(resp.status(), 204);

    // 13. 验证标签已添加
    let resp = client.get(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["tags"].as_array().unwrap().len(), 1);

    // 14. 添加另一个标签
    let resp = client
        .post_empty(&format!("/api/tickets/{}/tags/{}", ticket1_id, tag2_id))
        .await;
    assert_eq!(resp.status(), 204);

    // 15. 验证两个标签都已添加
    let resp = client.get(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["tags"].as_array().unwrap().len(), 2);

    // 16. 移除标签
    let resp = client
        .delete(&format!("/api/tickets/{}/tags/{}", ticket1_id, tag1_id))
        .await;
    assert_eq!(resp.status(), 204);

    // 17. 验证标签已移除
    let resp = client.get(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 200);
    let ticket: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(ticket["tags"].as_array().unwrap().len(), 1);

    // 18. 获取不存在的 ticket（应该返回404）
    let resp = client
        .get("/api/tickets/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(resp.status(), 404);

    // 19. 更新不存在的 ticket（应该返回404）
    let resp = client
        .put(
            "/api/tickets/00000000-0000-0000-0000-000000000000",
            json!({
                "title": "Test"
            }),
        )
        .await;
    assert_eq!(resp.status(), 404);

    // 20. 删除 ticket
    let resp = client.delete(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 204);

    // 21. 验证 ticket 已删除
    let resp = client.get(&format!("/api/tickets/{}", ticket1_id)).await;
    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_ticket_filtering() {
    // 获取全局测试锁，确保测试串行执行
    let _guard = TEST_MUTEX.lock().await;

    let server = TestServer::start().await;
    let client = TestClient::new(server.base_url.clone());

    // 创建标签
    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "urgent",
                "color": "#FF0000"
            }),
        )
        .await;
    let tag: serde_json::Value = resp.json().await.unwrap();
    let tag_id = tag["id"].as_str().unwrap();

    // 创建 tickets
    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Fix login bug",
                "description": "Users cannot login",
                "tag_ids": [tag_id]
            }),
        )
        .await;
    let ticket1: serde_json::Value = resp.json().await.unwrap();
    let ticket1_id = ticket1["id"].as_str().unwrap();

    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Add dark mode",
                "description": "Implement dark mode",
                "tag_ids": []
            }),
        )
        .await;
    let ticket2: serde_json::Value = resp.json().await.unwrap();
    let ticket2_id = ticket2["id"].as_str().unwrap();

    // 将 ticket1 标记为完成
    client
        .put(
            &format!("/api/tickets/{}", ticket1_id),
            json!({
                "completed": true
            }),
        )
        .await;

    // 1. 按标签筛选
    let resp = client.get(&format!("/api/tickets?tag={}", tag_id)).await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"].as_str().unwrap(), ticket1_id);

    // 2. 按完成状态筛选（已完成）
    let resp = client.get("/api/tickets?completed=true").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"].as_str().unwrap(), ticket1_id);

    // 3. 按完成状态筛选（未完成）
    let resp = client.get("/api/tickets?completed=false").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"].as_str().unwrap(), ticket2_id);

    // 4. 搜索 tickets
    let resp = client.get("/api/tickets?search=login").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0]["id"].as_str().unwrap(), ticket1_id);

    // 5. 搜索 tickets（不区分大小写）
    let resp = client.get("/api/tickets?search=LOGIN").await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);

    // 6. 组合筛选：标签 + 完成状态 + 搜索
    let resp = client
        .get(&format!(
            "/api/tickets?tag={}&completed=true&search=bug",
            tag_id
        ))
        .await;
    assert_eq!(resp.status(), 200);
    let tickets: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(tickets.len(), 1);
}

#[tokio::test]
async fn test_error_handling() {
    // 获取全局测试锁，确保测试串行执行
    let _guard = TEST_MUTEX.lock().await;

    let server = TestServer::start().await;
    let client = TestClient::new(server.base_url.clone());

    // 1. 添加标签到不存在的 ticket（应该返回404）
    let resp = client
        .post_empty("/api/tickets/00000000-0000-0000-0000-000000000000/tags/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(resp.status(), 404);

    // 2. 从不存在的 ticket 移除标签（应该返回404）
    let resp = client
        .delete("/api/tickets/00000000-0000-0000-0000-000000000000/tags/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(resp.status(), 404);

    // 3. 切换不存在的 ticket 的完成状态（应该返回404）
    let resp = client
        .patch("/api/tickets/00000000-0000-0000-0000-000000000000/toggle")
        .await;
    assert_eq!(resp.status(), 404);

    // 4. 删除不存在的 ticket（应该返回404）
    let resp = client
        .delete("/api/tickets/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(resp.status(), 404);

    // 创建 ticket 和 tag
    let resp = client
        .post(
            "/api/tickets",
            json!({
                "title": "Test ticket",
                "description": "Test"
            }),
        )
        .await;
    let ticket: serde_json::Value = resp.json().await.unwrap();
    let ticket_id = ticket["id"].as_str().unwrap();

    let resp = client
        .post(
            "/api/tags",
            json!({
                "name": "test-tag",
                "color": "#FF0000"
            }),
        )
        .await;
    let tag: serde_json::Value = resp.json().await.unwrap();
    let _tag_id = tag["id"].as_str().unwrap();

    // 5. 从不存在的 ticket 移除标签（应该返回404）
    let resp = client
        .delete(&format!(
            "/api/tickets/{}/tags/00000000-0000-0000-0000-000000000000",
            ticket_id
        ))
        .await;
    assert_eq!(resp.status(), 404);
}
