.PHONY: help install dev build fmt lint test clean
.DEFAULT_GOAL := help

# 路径变量
F := .
B := src-tauri

## help: 显示帮助信息
help:
	@echo "w2-db-query Makefile 命令:"
	@echo ""
	@echo "依赖:     make install          - 安装所有依赖"
	@echo "开发:     make dev              - 启动开发模式"
	@echo "构建:     make build            - Debug 构建"
	@echo "          make build-release    - Release 构建"
	@echo "代码质量: make fmt              - 格式化代码"
	@echo "          make lint             - 检查代码"
	@echo "          make typecheck        - TypeScript 类型检查"
	@echo "测试:     make test            - 运行后端测试"
	@echo "          make test-e2e         - 运行 E2E 测试"
	@echo "          make test-e2e-ui      - E2E 测试（UI 模式）"
	@echo "清理:     make clean            - 清理构建产物"
	@echo "          make clean-all        - 清理所有（含依赖）"

## install: 安装所有依赖
install:
	@cd $(F) && npm install --legacy-peer-deps
	@cd $(B) && cargo build
	@echo "✅ 依赖安装完成"

## dev: 启动开发模式
dev:
	@cd $(F) && npm run tauri dev

## build: Debug 构建
build:
	@cd $(F) && npm run tauri build --debug

## build-release: Release 构建
build-release:
	@cd $(F) && npm run tauri build

## fmt: 格式化代码
fmt:
	@cd $(B) && cargo fmt
	@cd $(F) && npm run format

## lint: 检查代码
lint: typecheck
	@cd $(B) && cargo clippy -- -D warnings
	@cd $(F) && npm run lint

## typecheck: TypeScript 类型检查
typecheck:
	@cd $(F) && npm run typecheck

## test: 运行后端测试
test:
	@cd $(B) && cargo test --lib -- --test-threads=1

## test-e2e: 运行 E2E 测试
test-e2e:
	@cd $(F) && npm run test:e2e

## test-e2e-ui: E2E 测试（UI 模式）
test-e2e-ui:
	@cd $(F) && npm run test:e2e:ui

## clean: 清理构建产物
clean:
	@cd $(F) && rm -rf dist node_modules/.vite .vite
	@cd $(B) && cargo clean

## clean-all: 清理所有（含依赖）
clean-all: clean
	@cd $(F) && rm -rf node_modules package-lock.json
	@cd $(B) && rm -rf Cargo.lock
	@echo "⚠️  已删除依赖文件，运行 'make install' 重新安装"
