.PHONY: help \
	install install-frontend install-backend \
	dev dev-frontend dev-backend \
	build build-frontend build-backend build-release \
	fmt fmt-frontend fmt-backend \
	lint lint-frontend lint-backend \
	typecheck \
	test test-backend test-frontend \
	clean clean-frontend clean-backend clean-all

# 项目路径
PROJECT_DIR := .
FRONTEND_DIR := $(PROJECT_DIR)
BACKEND_DIR := $(PROJECT_DIR)/src-tauri

# 默认目标
.DEFAULT_GOAL := help

## help: 显示帮助信息
help:
	@echo "w2-db-query - 数据库查询工具 Makefile 命令"
	@echo ""
	@echo "依赖管理:"
	@echo "  make install              - 安装所有依赖（前端 npm + 后端 cargo）"
	@echo "  make install-frontend     - 仅安装前端依赖（npm install）"
	@echo "  make install-backend      - 仅安装后端依赖（cargo build）"
	@echo ""
	@echo "开发:"
	@echo "  make dev                 - 启动 Tauri 开发模式（前端 + 后端）"
	@echo "  make dev-frontend        - 仅启动前端开发服务器（Vite）"
	@echo ""
	@echo "构建:"
	@echo "  make build               - 构建应用（Debug 模式）"
	@echo "  make build-release       - 构建应用（Release 模式，生产版本）"
	@echo "  make build-frontend      - 仅构建前端（TypeScript + Vite）"
	@echo "  make build-backend       - 仅构建后端（Cargo）"
	@echo ""
	@echo "代码质量:"
	@echo "  make fmt                 - 格式化所有代码（Rust + TypeScript）"
	@echo "  make fmt-frontend        - 格式化前端代码（Prettier）"
	@echo "  make fmt-backend         - 格式化后端代码（cargo fmt）"
	@echo "  make lint                - 检查所有代码（Rust + TypeScript）"
	@echo "  make lint-frontend       - 检查前端代码（ESLint）"
	@echo "  make lint-backend        - 检查后端代码（cargo clippy）"
	@echo "  make typecheck           - TypeScript 类型检查"
	@echo ""
	@echo "测试:"
	@echo "  make test                - 运行所有测试"
	@echo "  make test-backend        - 运行后端测试（cargo test）"
	@echo ""
	@echo "清理:"
	@echo "  make clean               - 清理构建产物（前端 + 后端）"
	@echo "  make clean-frontend      - 清理前端构建产物"
	@echo "  make clean-backend       - 清理后端构建产物"
	@echo "  make clean-all           - 清理所有（包括 node_modules 和 target）"

## install: 安装所有依赖
install: install-frontend install-backend
	@echo "✅ 所有依赖安装完成"

## install-frontend: 安装前端依赖
install-frontend:
	@echo "📦 安装前端依赖..."
	@cd $(FRONTEND_DIR) && npm install --legacy-peer-deps
	@echo "✅ 前端依赖安装完成"

## install-backend: 安装后端依赖（通过构建）
install-backend:
	@echo "📦 安装后端依赖..."
	@cd $(BACKEND_DIR) && cargo build
	@echo "✅ 后端依赖安装完成"

## dev: 启动 Tauri 开发模式
dev:
	@echo "🚀 启动 Tauri 开发模式..."
	@echo "⚠️  使用 Ctrl+C 停止"
	@cd $(FRONTEND_DIR) && npm run tauri dev

## dev-frontend: 仅启动前端开发服务器
dev-frontend:
	@echo "🚀 启动前端开发服务器..."
	@echo "⚠️  使用 Ctrl+C 停止"
	@cd $(FRONTEND_DIR) && npm run dev

## build: 构建应用（Debug 模式）
build:
	@echo "🔨 构建应用（Debug 模式）..."
	@cd $(FRONTEND_DIR) && npm run tauri build --debug
	@echo "✅ 构建完成"

## build-release: 构建应用（Release 模式，生产版本）
build-release:
	@echo "🔨 构建应用（Release 模式，生产版本）..."
	@cd $(FRONTEND_DIR) && npm run tauri build
	@echo "✅ 生产构建完成: $(BACKEND_DIR)/target/release/bundle/"

## build-frontend: 仅构建前端
build-frontend:
	@echo "🔨 构建前端..."
	@cd $(FRONTEND_DIR) && npm run build
	@echo "✅ 前端构建完成: $(FRONTEND_DIR)/dist"

## build-backend: 仅构建后端
build-backend:
	@echo "🔨 构建后端..."
	@cd $(BACKEND_DIR) && cargo build
	@echo "✅ 后端构建完成: $(BACKEND_DIR)/target/debug/w2-db-query"

## fmt: 格式化所有代码
fmt: fmt-backend fmt-frontend
	@echo "✅ 所有代码格式化完成"

## fmt-frontend: 格式化前端代码
fmt-frontend:
	@echo "✨ 格式化前端代码..."
	@cd $(FRONTEND_DIR) && npm run format
	@echo "✅ 前端代码格式化完成"

## fmt-backend: 格式化后端代码
fmt-backend:
	@echo "✨ 格式化后端代码..."
	@cd $(BACKEND_DIR) && cargo fmt
	@echo "✅ 后端代码格式化完成"

## lint: 检查所有代码
lint: lint-backend lint-frontend typecheck
	@echo "✅ 所有代码检查完成"

## lint-frontend: 检查前端代码
lint-frontend:
	@echo "🔍 检查前端代码..."
	@cd $(FRONTEND_DIR) && npm run lint
	@echo "✅ 前端代码检查完成"

## lint-backend: 检查后端代码
lint-backend:
	@echo "🔍 检查后端代码..."
	@cd $(BACKEND_DIR) && cargo clippy -- -D warnings
	@echo "✅ 后端代码检查完成"

## typecheck: TypeScript 类型检查
typecheck:
	@echo "🔍 TypeScript 类型检查..."
	@cd $(FRONTEND_DIR) && npm run typecheck
	@echo "✅ TypeScript 类型检查完成"

## test: 运行所有测试
test: test-backend
	@echo "✅ 所有测试完成"

## test-backend: 运行后端测试
test-backend:
	@echo "🧪 运行后端测试..."
	@cd $(BACKEND_DIR) && cargo test --verbose
	@echo "✅ 后端测试完成"

## test-frontend: 运行前端测试（如果配置了）
test-frontend:
	@echo "🧪 运行前端测试..."
	@echo "⚠️  前端测试尚未配置"
	@echo "✅ 前端测试跳过"

## clean: 清理构建产物
clean: clean-frontend clean-backend
	@echo "✅ 构建产物清理完成"

## clean-frontend: 清理前端构建产物
clean-frontend:
	@echo "🧹 清理前端构建产物..."
	@cd $(FRONTEND_DIR) && rm -rf dist node_modules/.vite .vite
	@echo "✅ 前端构建产物清理完成"

## clean-backend: 清理后端构建产物
clean-backend:
	@echo "🧹 清理后端构建产物..."
	@cd $(BACKEND_DIR) && cargo clean
	@echo "✅ 后端构建产物清理完成"

## clean-all: 清理所有（包括依赖）
clean-all: clean-frontend clean-backend
	@echo "🧹 清理所有文件..."
	@cd $(FRONTEND_DIR) && rm -rf node_modules package-lock.json
	@cd $(BACKEND_DIR) && rm -rf Cargo.lock
	@echo "⚠️  已删除 node_modules 和 Cargo.lock"
	@echo "✅ 清理完成，运行 'make install' 重新安装依赖"
