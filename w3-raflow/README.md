# RaFlow - 语音输入工具

RaFlow 是一款基于 ElevenLabs Scribe v2 Realtime API 构建的跨平台语音输入工具，提供低延迟、高准确率的实时语音转文字功能。

## Phase 1 完成（基础框架搭建）

Phase 1 已完成所有 9 个任务：

### 项目初始化
- ✅ 任务 1.1: 创建 Tauri 2 项目脚手架
- ✅ 任务 1.2: 配置 Rust 依赖和模块结构
- ✅ 任务 1.3: 初始化前端项目

### 核心模块搭建
- ✅ 任务 1.4: 实现错误类型系统
- ✅ 任务 1.5: 实现应用状态管理
- ✅ 任务 1.6: 实现应用主结构

### 配置系统
- ✅ 任务 1.7: 实现配置数据模型
- ✅ 任务 1.8: 实现配置存储服务
- ✅ 任务 1.9: 实现 Tauri 配置命令

## 项目结构

```
w3-raflow/
├── src/                    # React 前端
│   ├── api/               # Tauri API 封装
│   ├── stores/            # Zustand 状态管理
│   ├── App.tsx            # 主应用组件
│   └── main.tsx           # 入口文件
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── core/          # 核心模块（错误、状态、应用）
│   │   ├── config/        # 配置管理
│   │   ├── audio/         # 音频模块（待实现）
│   │   ├── network/       # 网络模块（待实现）
│   │   ├── input/         # 输入模块（待实现）
│   │   ├── tray/          # 托盘模块（待实现）
│   │   └── commands/      # Tauri 命令
│   ├── Cargo.toml
│   └── tauri.conf.json
├── package.json
└── Cargo.toml
```

## 开发环境

- Rust 1.82+
- Node.js 18+ or 20+
- npm 10+

## 开发命令

### 安装依赖
```bash
npm install
```

### 开发模式
```bash
npm run tauri:dev
```

### 构建生产版本
```bash
npm run tauri:build
```

## 下一步（Phase 2）

Phase 2 将实现核心功能：

- 音频捕获模块
- WebSocket 通信模块
- 文本插入模块
- 全局热键模块

## 技术栈

### 后端
- Rust 2024
- Tauri 2.1
- Tokio 1.49
- cpal 0.17（音频）
- tokio-tungstenite 0.28（WebSocket）

### 前端
- React 18
- TypeScript 5.7
- Vite 6.0
- TailwindCSS 3.4
- Zustand 5.0

## 许可证

MIT
