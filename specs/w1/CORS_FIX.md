# CORS 跨域问题修复说明

## 问题描述

访问前端页面时出现跨域异常（CORS error）。

## 解决方案

已实施两个解决方案来修复跨域问题：

### 方案 1: 改进后端 CORS 配置（已实施）

**文件**: `backend/src/main.rs`

**更改**:
- 添加了 `allow_credentials(true)` - 允许携带凭证
- 添加了 `expose_headers(Any)` - 暴露所有响应头

```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
    .allow_credentials(true)
    .expose_headers(Any)
```

### 方案 2: 配置 Vite 开发代理（已实施，推荐）

**文件**: `frontend/vite.config.ts`

**更改**:
- 配置了开发服务器代理，将所有 `/api` 请求代理到 `http://localhost:3000`

```typescript
server: {
  proxy: {
    '/api': {
      target: 'http://localhost:3000',
      changeOrigin: true,
      secure: false,
    },
  },
}
```

**文件**: `frontend/src/services/api.ts`

**更改**:
- 开发环境下使用空字符串作为 API 基础 URL（使用相对路径）
- 生产环境使用完整 URL

```typescript
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 
  (import.meta.env.DEV ? "" : "http://localhost:3000");
```

## 使用方法

### 开发环境（推荐使用代理）

1. **启动后端服务器**:
   ```bash
   cd backend
   cargo run
   ```
   后端将在 `http://localhost:3000` 运行

2. **启动前端开发服务器**:
   ```bash
   cd frontend
   npm run dev
   ```
   前端将在 `http://localhost:5173` 运行（Vite 默认端口）

3. **访问前端**:
   打开浏览器访问 `http://localhost:5173`
   
   **注意**: 前端会通过 Vite 代理访问后端，不会出现跨域问题。

### 生产环境

1. **设置环境变量**:
   创建 `frontend/.env.production`:
   ```env
   VITE_API_BASE_URL=http://your-backend-url:3000
   ```

2. **构建前端**:
   ```bash
   cd frontend
   npm run build
   ```

3. **部署**:
   将 `dist` 目录部署到静态文件服务器

## 验证

### 检查后端 CORS 配置

启动后端后，可以通过以下方式验证 CORS 配置：

```bash
curl -H "Origin: http://localhost:5173" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: Content-Type" \
     -X OPTIONS \
     http://localhost:3000/api/tickets \
     -v
```

应该看到响应头包含：
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: *`
- `Access-Control-Allow-Headers: *`
- `Access-Control-Allow-Credentials: true`

### 检查前端代理

启动前端开发服务器后，访问 `http://localhost:5173`，打开浏览器开发者工具：

1. **Network 标签**: 查看 API 请求
2. **Console 标签**: 查看是否有 CORS 错误

如果使用代理，请求 URL 应该是相对路径 `/api/...`，而不是 `http://localhost:3000/api/...`。

## 故障排除

### 问题 1: 仍然出现 CORS 错误

**可能原因**:
- 后端服务器未启动
- 端口不匹配
- 浏览器缓存

**解决方法**:
1. 确认后端服务器正在运行在 `http://localhost:3000`
2. 检查前端 API 配置是否正确
3. 清除浏览器缓存并硬刷新（Ctrl+Shift+R）

### 问题 2: 代理不工作

**可能原因**:
- Vite 配置未生效
- 需要重启开发服务器

**解决方法**:
1. 停止前端开发服务器（Ctrl+C）
2. 重新启动：`npm run dev`
3. 检查 `vite.config.ts` 配置是否正确

### 问题 3: 生产环境跨域问题

**解决方法**:
1. 确保设置了正确的 `VITE_API_BASE_URL` 环境变量
2. 确保后端 CORS 配置允许生产环境的前端域名
3. 或者在生产环境也使用代理（Nginx 等）

## 注意事项

1. **开发环境**: 推荐使用 Vite 代理，避免跨域问题
2. **生产环境**: 需要配置正确的 CORS 策略，只允许信任的域名
3. **安全性**: 当前配置允许所有来源（`Any`），生产环境应该限制为特定域名

## 生产环境 CORS 配置建议

如果需要限制特定域名，可以修改后端配置：

```rust
use tower_http::cors::{CorsLayer, AllowOrigin};

CorsLayer::new()
    .allow_origin(AllowOrigin::exact("https://your-frontend-domain.com".parse().unwrap()))
    .allow_methods(Any)
    .allow_headers(Any)
    .allow_credentials(true)
```

或者从环境变量读取：

```rust
let allowed_origin = std::env::var("ALLOWED_ORIGIN")
    .unwrap_or_else(|_| "*".to_string());
    
CorsLayer::new()
    .allow_origin(if allowed_origin == "*" {
        Any.into()
    } else {
        AllowOrigin::exact(allowed_origin.parse().unwrap())
    })
    .allow_methods(Any)
    .allow_headers(Any)
    .allow_credentials(true)
```
