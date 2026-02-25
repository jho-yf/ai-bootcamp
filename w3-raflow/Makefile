# RaFlow Makefile
# 便捷开发命令

.PHONY: help dev build clean install test check fmt lint

# 默认目标
.DEFAULT_GOAL := help

# 颜色输出
BLUE  := \033[0;34m
GREEN := \033[0;32m
YELLOW:= \033[0;33m
WHITE := \033[0;37m
RESET := \033[0m

## help: 显示帮助信息
help:
	@echo "$(BLUE)RaFlow 开发命令$(RESET)"
	@echo ""
	@grep -E '^## ' Makefile | sed 's/## /  /' | sed 's/:/$(GREEN):$(RESET)/'

## dev: 启动开发服务器
dev:
	@echo "$(GREEN)启动开发服务器...$(RESET)"
	npm run tauri:dev

## build: 构建生产版本
build:
	@echo "$(GREEN)构建生产版本...$(RESET)"
	npm run tauri:build

## build-debug: 构建调试版本 (不压缩)
build-debug:
	@echo "$(GREEN)构建调试版本...$(RESET)"
	cargo tauri build --debug

## install: 安装依赖
install:
	@echo "$(GREEN)安装 npm 依赖...$(RESET)"
	npm install

## install-rust: 更新 Rust 依赖
install-rust:
	@echo "$(GREEN)更新 Rust 依赖...$(RESET)"
	cargo fetch

## clean: 清理构建产物
clean:
	@echo "$(YELLOW)清理构建产物...$(RESET)"
	rm -rf dist target
	cargo clean

## test: 运行 Rust 测试
test:
	@echo "$(GREEN)运行 Rust 测试...$(RESET)"
	cargo test

## test-watch: 监听模式运行测试
test-watch:
	@echo "$(GREEN)监听模式运行测试...$(RESET)"
	cargo watch -x test

## check: 快速检查代码 (不构建)
check:
	@echo "$(GREEN)检查代码...$(RESET)"
	cargo check

## clippy: 运行 Clippy 代码检查
clippy:
	@echo "$(GREEN)运行 Clippy...$(RESET)"
	cargo clippy -- -D warnings

## fmt: 格式化 Rust 代码
fmt:
	@echo "$(GREEN)格式化 Rust 代码...$(RESET)"
	cargo fmt

## fmt-check: 检查 Rust 代码格式
fmt-check:
	@echo "$(GREEN)检查代码格式...$(RESET)"
	cargo fmt -- --check

## lint: 运行 TypeScript 类型检查
lint:
	@echo "$(GREEN)TypeScript 类型检查...$(RESET)"
	npx tsc --noEmit

## update: 更新所有依赖
update:
	@echo "$(GREEN)更新依赖...$(RESET)"
	npm update
	cargo update

## release: 构建发布版本
release:
	@echo "$(GREEN)构建发布版本...$(RESET)"
	cargo tauri build

## run: 仅运行已编译的调试版本
run:
	@echo "$(GREEN)运行调试版本...$(RESET)"
	cargo run

## docs: 生成 Rust 文档
docs:
	@echo "$(GREEN)生成文档...$(RESET)"
	cargo doc --open

## tree: 显示依赖树
tree:
	@echo "$(GREEN)显示 Rust 依赖树...$(RESET)"
	cargo tree

## size: 分析构建产物大小
size:
	@echo "$(GREEN)分析构建大小...$(RESET)"
	cargo striptease

## watch: 监听文件变化并自动重新构建
watch:
	@echo "$(GREEN)监听文件变化...$(RESET)"
	cargo tauri dev
