#!/bin/bash

# 自动化测试运行脚本
# 使用方法: ./run_tests.sh

set -e

echo "=========================================="
echo "Project Alpha - 自动化集成测试"
echo "=========================================="
echo ""

# 检查环境变量
if [ -z "$DATABASE_URL" ]; then
    echo "设置默认测试数据库 URL..."
    export DATABASE_URL="postgresql://postgres:postgres@localhost/project_alpha_test"
fi

export HOST="127.0.0.1"
export PORT="0"
export RUST_LOG="error"

echo "环境变量:"
echo "  DATABASE_URL: $DATABASE_URL"
echo "  HOST: $HOST"
echo "  PORT: $PORT"
echo ""

# 检查数据库连接
echo "检查数据库连接..."
if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo "错误: 无法连接到数据库"
    echo "请确保:"
    echo "  1. PostgreSQL 正在运行"
    echo "  2. 数据库 'project_alpha_test' 已创建"
    echo "  3. DATABASE_URL 环境变量正确设置"
    exit 1
fi
echo "✓ 数据库连接成功"
echo ""

# 运行测试
echo "运行集成测试..."
echo ""

cargo test --test integration_test -- --nocapture

echo ""
echo "=========================================="
echo "测试完成！"
echo "=========================================="
