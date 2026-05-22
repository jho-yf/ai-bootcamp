.PHONY: help install dev backend frontend build clean

# Vite 8 需要 Node.js >= 22.12
NODE_VERSION := 22.12.0

help:
	@echo "GenSlides Commands:"
	@echo "  make install    - 安装前后端依赖"
	@echo "  make dev        - 同时启动前后端（前端:5173 后端:3000）"
	@echo "  make backend    - 仅启动后端（port 3000）"
	@echo "  make frontend   - 仅启动前端（port 5173）"
	@echo "  make build      - 构建前后端"
	@echo "  make clean      - 清理 node_modules"
	@echo ""
	@echo "注意：Vite 8 需要 Node.js >= 22.12，请先运行: nvm use $(NODE_VERSION)"

install:
	cd frontend && npm install
	cd backend && npm install

dev:
	@trap 'kill 0' EXIT; \
	cd frontend && npm run dev & \
	cd backend && npm run dev & \
	wait

backend:
	cd backend && npm run dev

frontend:
	cd frontend && npm run dev

build:
	cd frontend && npm run build
	cd backend && npm run build

clean:
	rm -rf frontend/node_modules backend/node_modules
