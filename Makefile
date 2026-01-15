.PHONY: help \
	w1-install w1-dev w1-backend w1-frontend w1-backend-build w1-frontend-build \
	w1-backend-migrate w1-backend-test w1-clean

# 项目路径
W1_PROJECT_DIR := w1-project-alpha
W1_BACKEND_DIR := $(W1_PROJECT_DIR)/backend
W1_FRONTEND_DIR := $(W1_PROJECT_DIR)/frontend

# 默认目标
.DEFAULT_GOAL := help

## help: 显示帮助信息
help:
	@echo "AI Bootcamp - Makefile 命令"
	@echo ""
	@echo "项目 w1-project-alpha:"
	@echo "  make w1-install          - 安装所有依赖（前端 npm install）"
	@echo "  make w1-dev              - 同时启动后端和前端（开发模式）"
	@echo "  make w1-backend          - 启动后端服务器（开发模式）"
	@echo "  make w1-frontend         - 启动前端开发服务器"
	@echo "  make w1-backend-build    - 构建后端（Release 模式）"
	@echo "  make w1-frontend-build   - 构建前端（生产模式）"
	@echo "  make w1-backend-migrate  - 运行数据库迁移"
	@echo "  make w1-backend-test     - 运行后端测试"
	@echo "  make w1-clean            - 清理构建产物"

## w1-install: 安装 w1-project-alpha 的所有依赖
w1-install:
	@echo "📦 安装 w1-project-alpha 前端依赖..."
	@cd $(W1_FRONTEND_DIR) && npm install
	@echo "✅ 依赖安装完成"

## w1-dev: 同时启动 w1-project-alpha 后端和前端（开发模式）
w1-dev:
	@echo "🚀 启动 w1-project-alpha 开发环境（后端 + 前端）..."
	@echo "⚠️  注意：这将启动两个进程，使用 Ctrl+C 停止"
	@echo ""
	@trap 'kill 0' EXIT; \
	cd $(W1_BACKEND_DIR) && cargo run & \
	cd $(W1_FRONTEND_DIR) && npm run dev & \
	wait

## w1-backend: 启动 w1-project-alpha 后端服务器（开发模式）
w1-backend:
	@echo "🚀 启动 w1-project-alpha 后端服务器..."
	@cd $(W1_BACKEND_DIR) && cargo run

## w1-frontend: 启动 w1-project-alpha 前端开发服务器
w1-frontend:
	@echo "🚀 启动 w1-project-alpha 前端开发服务器..."
	@cd $(W1_FRONTEND_DIR) && npm run dev

## w1-backend-build: 构建 w1-project-alpha 后端（Release 模式）
w1-backend-build:
	@echo "🔨 构建 w1-project-alpha 后端（Release 模式）..."
	@cd $(W1_BACKEND_DIR) && cargo build --release
	@echo "✅ 后端构建完成: $(W1_BACKEND_DIR)/target/release/project-alpha-backend"

## w1-frontend-build: 构建 w1-project-alpha 前端（生产模式）
w1-frontend-build:
	@echo "🔨 构建 w1-project-alpha 前端（生产模式）..."
	@cd $(W1_FRONTEND_DIR) && npm run build
	@echo "✅ 前端构建完成: $(W1_FRONTEND_DIR)/dist"

## w1-backend-migrate: 运行 w1-project-alpha 数据库迁移
w1-backend-migrate:
	@echo "🗄️  运行 w1-project-alpha 数据库迁移..."
	@cd $(W1_BACKEND_DIR) && sqlx migrate run --source migrations
	@echo "✅ 数据库迁移完成"

## w1-backend-test: 运行 w1-project-alpha 后端测试
w1-backend-test:
	@echo "🧪 运行 w1-project-alpha 后端测试..."
	@cd $(W1_BACKEND_DIR) && cargo test --all-features --verbose

## w1-clean: 清理 w1-project-alpha 构建产物
w1-clean:
	@echo "🧹 清理 w1-project-alpha 构建产物..."
	@cd $(W1_BACKEND_DIR) && cargo clean
	@cd $(W1_FRONTEND_DIR) && rm -rf dist node_modules/.vite
	@echo "✅ 清理完成"
