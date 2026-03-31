# PostgreSQL MCP Server - Test Databases Makefile
# 用于创建、重建和管理测试数据库

# PostgreSQL 连接配置
PG_HOST ?= localhost
PG_PORT ?= 5432
PG_USER ?= postgres
PG_PASSWORD ?= postgres

# 数据库列表
DATABASES = pg_mcp_small pg_mcp_medium pg_mcp_large
SQL_FILES = 01-small-ecommerce.sql 02-medium-school.sql 03-large-hospital.sql

# 颜色输出
COLOR_RESET = \033[0m
COLOR_GREEN = \033[32m
COLOR_YELLOW = \033[33m
COLOR_BLUE = \033[34m
COLOR_RED = \033[31m

# 默认目标
.PHONY: all
all: create-all

# ============================================================================
# 数据库管理
# ============================================================================

## 创建所有测试数据库
.PHONY: create-all
create-all: create-small create-medium create-large
	@echo "$(COLOR_GREEN)=========================================$(COLOR_RESET)"
	@echo "$(COLOR_GREEN)所有测试数据库创建完成！$(COLOR_RESET)"
	@echo "$(COLOR_GREEN)=========================================$(COLOR_RESET)"
	@echo ""
	@$(MAKE) --no-print-directory status

## 创建小型电商数据库
.PHONY: create-small
create-small:
	@echo "$(COLOR_BLUE)创建小型电商数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -f 01-small-ecommerce.sql
	@echo "$(COLOR_GREEN)✓ 小型电商数据库 (pg_mcp_small) 创建完成$(COLOR_RESET)"

## 创建中型学校数据库
.PHONY: create-medium
create-medium:
	@echo "$(COLOR_BLUE)创建中型学校管理数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -f 02-medium-school.sql
	@echo "$(COLOR_GREEN)✓ 中型学校管理数据库 (pg_mcp_medium) 创建完成$(COLOR_RESET)"

## 创建大型医院数据库
.PHONY: create-large
create-large:
	@echo "$(COLOR_BLUE)创建大型医院信息系统数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -f 03-large-hospital.sql
	@echo "$(COLOR_GREEN)✓ 大型医院信息系统数据库 (pg_mcp_large) 创建完成$(COLOR_RESET)"

## 重建所有数据库
.PHONY: rebuild-all
rebuild-all: drop-all create-all

## 重建单个数据库
.PHONY: rebuild-small rebuild-medium rebuild-large
rebuild-small: drop-small create-small
rebuild-medium: drop-medium create-medium
rebuild-large: drop-large create-large

## 删除所有测试数据库
.PHONY: drop-all
drop-all: drop-small drop-medium drop-large
	@echo "$(COLOR_GREEN)所有测试数据库已删除$(COLOR_RESET)"

.PHONY: drop-small
drop-small:
	@echo "$(COLOR_YELLOW)删除小型电商数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -c "DROP DATABASE IF EXISTS pg_mcp_small;" 2>/dev/null || true

.PHONY: drop-medium
drop-medium:
	@echo "$(COLOR_YELLOW)删除中型学校管理数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -c "DROP DATABASE IF EXISTS pg_mcp_medium;" 2>/dev/null || true

.PHONY: drop-large
drop-large:
	@echo "$(COLOR_YELLOW)删除大型医院信息系统数据库...$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -c "DROP DATABASE IF EXISTS pg_mcp_large;" 2>/dev/null || true

# ============================================================================
# 数据查询
# ============================================================================

## 显示数据库状态
.PHONY: status
status:
	@echo "$(COLOR_BLUE)=========================================$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)PostgreSQL MCP 测试数据库状态$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)=========================================$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)数据库列表:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -q -c "SELECT datname AS \"数据库\", pg_size_pretty(pg_database_size(datname)) AS \"大小\" FROM pg_database WHERE datname LIKE 'pg_mcp_%' ORDER BY datname;" 2>/dev/null || echo "  无法连接到数据库"
	@echo ""
	@echo "$(COLOR_YELLOW)数据库统计:$(COLOR_RESET)"
	@echo "$(COLOR_RESET)$(foreach db,$(DATABASES),$(shell psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d $(db) -q -t -c "SELECT '  $(db): ' || COUNT(*) || ' 张表' FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';" 2>/dev/null))"

## 查看特定数据库的表结构
.PHONY: describe-small describe-medium describe-large
describe-small:
	@echo "$(COLOR_BLUE)=== 小型电商数据库 (pg_mcp_small) 表结构 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c "\d"

describe-medium:
	@echo "$(COLOR_BLUE)=== 中型学校管理数据库 (pg_mcp_medium) 表结构 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c "\d"

describe-large:
	@echo "$(COLOR_BLUE)=== 大型医院信息系统数据库 (pg_mcp_large) 表结构 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "\d"

## 查看数据库视图
.PHONY: views-small views-medium views-large
views-small:
	@echo "$(COLOR_BLUE)=== 小型电商数据库视图 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c "\dv"

views-medium:
	@echo "$(COLOR_BLUE)=== 中型学校管理数据库视图 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c "\dv"

views-large:
	@echo "$(COLOR_BLUE)=== 大型医院信息系统数据库视图 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "\dv"

## 查看数据库类型
.PHONY: types-small types-medium types-large
types-small:
	@echo "$(COLOR_BLUE)=== 小型电商数据库自定义类型 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c "\dT"

types-medium:
	@echo "$(COLOR_BLUE)=== 中型学校管理数据库自定义类型 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c "\dT"

types-large:
	@echo "$(COLOR_BLUE)=== 大型医院信息系统数据库自定义类型 ===$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "\dT"

# ============================================================================
# 测试查询示例
# ============================================================================

## 小型数据库示例查询
.PHONY: test-small
test-small:
	@echo "$(COLOR_BLUE)=== 小型电商数据库测试查询 ===$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)1. 查询所有产品库存状态:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c "SELECT * FROM v_product_inventory ORDER BY stock_quantity;"
	@echo ""
	@echo "$(COLOR_YELLOW)2. 查询本月订单汇总:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c "SELECT * FROM v_order_summary WHERE order_date >= '2025-03-01' ORDER BY order_date DESC;"

## 中型数据库示例查询
.PHONY: test-medium
test-medium:
	@echo "$(COLOR_BLUE)=== 中型学校管理数据库测试查询 ===$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)1. 查询所有班级统计:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c "SELECT * FROM v_class_statistics ORDER BY grade_level, class_name;"
	@echo ""
	@echo "$(COLOR_YELLOW)2. 查询学生出勤统计:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c "SELECT * FROM v_attendance_summary WHERE attendance_rate < 95 ORDER BY attendance_rate;"

## 大型数据库示例查询
.PHONY: test-large
test-large:
	@echo "$(COLOR_BLUE)=== 大型医院信息系统数据库测试查询 ===$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_YELLOW)1. 查询今日挂号:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "SELECT * FROM v_today_registrations ORDER BY registration_time;"
	@echo ""
	@echo "$(COLOR_YELLOW)2. 查询在院患者:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "SELECT * FROM v_current_inpatients ORDER BY admission_date;"
	@echo ""
	@echo "$(COLOR_YELLOW)3. 查询床位使用率:$(COLOR_RESET)"
	@psql -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c "SELECT * FROM v_bed_utilization ORDER BY occupancy_rate DESC;"

## 运行所有测试查询
.PHONY: test-all
test-all: test-small test-medium test-large

# ============================================================================
# 导入导出
# ============================================================================

## 导出数据库备份
.PHONY: backup-small backup-medium backup-large backup-all
backup-small:
	@echo "$(COLOR_BLUE)备份小型电商数据库...$(COLOR_RESET)"
	@pg_dump -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -F c -f pg_mcp_small_backup.dump pg_mcp_small
	@echo "$(COLOR_GREEN)✓ 备份完成: pg_mcp_small_backup.dump$(COLOR_RESET)"

backup-medium:
	@echo "$(COLOR_BLUE)备份中型学校管理数据库...$(COLOR_RESET)"
	@pg_dump -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -F c -f pg_mcp_medium_backup.dump pg_mcp_medium
	@echo "$(COLOR_GREEN)✓ 备份完成: pg_mcp_medium_backup.dump$(COLOR_RESET)"

backup-large:
	@echo "$(COLOR_BLUE)备份大型医院信息系统数据库...$(COLOR_RESET)"
	@pg_dump -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -F c -f pg_mcp_large_backup.dump pg_mcp_large
	@echo "$(COLOR_GREEN)✓ 备份完成: pg_mcp_large_backup.dump$(COLOR_RESET)"

backup-all: backup-small backup-medium backup-large

## 恢复数据库备份
.PHONY: restore-small restore-medium restore-large
restore-small:
	@echo "$(COLOR_BLUE)恢复小型电商数据库...$(COLOR_RESET)"
	@pg_restore -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_small -c pg_mcp_small_backup.dump 2>/dev/null || $(MAKE) create-small
	@echo "$(COLOR_GREEN)✓ 恢复完成$(COLOR_RESET)"

restore-medium:
	@echo "$(COLOR_BLUE)恢复中型学校管理数据库...$(COLOR_RESET)"
	@pg_restore -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_medium -c pg_mcp_medium_backup.dump 2>/dev/null || $(MAKE) create-medium
	@echo "$(COLOR_GREEN)✓ 恢复完成$(COLOR_RESET)"

restore-large:
	@echo "$(COLOR_BLUE)恢复大型医院信息系统数据库...$(COLOR_RESET)"
	@pg_restore -h $(PG_HOST) -p $(PG_PORT) -U $(PG_USER) -d pg_mcp_large -c pg_mcp_large_backup.dump 2>/dev/null || $(MAKE) create-large
	@echo "$(COLOR_GREEN)✓ 恢复完成$(COLOR_RESET)"

# ============================================================================
# 帮助信息
# ============================================================================

.PHONY: help
help:
	@echo "$(COLOR_BLUE)=========================================$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)PostgreSQL MCP 测试数据库管理工具$(COLOR_RESET)"
	@echo "$(COLOR_BLUE)=========================================$(COLOR_RESET)"
	@echo ""
	@echo "$(COLOR_GREEN)使用方法:$(COLOR_RESET)"
	@echo "  make [target] [PG_HOST=localhost] [PG_PORT=5432] [PG_USER=postgres] [PG_PASSWORD=postgres]"
	@echo ""
	@echo "$(COLOR_GREEN)数据库管理:$(COLOR_RESET)"
	@echo "  make all              - 创建所有测试数据库"
	@echo "  make create-small     - 创建小型电商数据库"
	@echo "  make create-medium    - 创建中型学校数据库"
	@echo "  make create-large     - 创建大型医院数据库"
	@echo "  make rebuild-all      - 重建所有数据库"
	@echo "  make drop-all         - 删除所有测试数据库"
	@echo ""
	@echo "$(COLOR_GREEN)状态查看:$(COLOR_RESET)"
	@echo "  make status           - 显示数据库状态"
	@echo "  make describe-small   - 查看小型数据库表结构"
	@echo "  make describe-medium  - 查看中型数据库表结构"
	@echo "  make describe-large   - 查看大型数据库表结构"
	@echo "  make views-small      - 查看小型数据库视图"
	@echo "  make views-medium     - 查看中型数据库视图"
	@echo "  make views-large      - 查看大型数据库视图"
	@echo "  make types-small      - 查看小型数据库类型"
	@echo "  make types-medium     - 查看中型数据库类型"
	@echo "  make types-large      - 查看大型数据库类型"
	@echo ""
	@echo "$(COLOR_GREEN)测试查询:$(COLOR_RESET)"
	@echo "  make test-small       - 小型数据库测试查询"
	@echo "  make test-medium      - 中型数据库测试查询"
	@echo "  make test-large       - 大型数据库测试查询"
	@echo "  make test-all         - 运行所有测试查询"
	@echo ""
	@echo "$(COLOR_GREEN)备份恢复:$(COLOR_RESET)"
	@echo "  make backup-all       - 备份所有数据库"
	@echo "  make restore-small    - 恢复小型数据库"
	@echo "  make restore-medium   - 恢复中型数据库"
	@echo "  make restore-large    - 恢复大型数据库"
	@echo ""
	@echo "$(COLOR_GREEN)示例:$(COLOR_RESET)"
	@echo "  make status                           # 查看当前状态"
	@echo "  make create-small PG_HOST=myhost      # 创建小型数据库(指定主机)"
	@echo "  make rebuild-all                      # 重建所有数据库"
	@echo ""
	@echo "$(COLOR_BLUE)=========================================$(COLOR_RESET)"

# 设置默认密码环境变量
export PGPASSWORD = $(PG_PASSWORD)
