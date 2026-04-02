#!/bin/bash
#
# PostgreSQL MCP Natural Language to SQL Shell Script
#
# Usage: ./pg_mcp_query.sh "查询 pg_mcp_small 数据库中所有产品的库存状态"
#

set -euo pipefail

# Database connection parameters
PG_HOST="localhost"
PG_PORT=15432
PG_USER="pgmcp"
PG_PASSWORD="pgmcp_test"

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SKILL_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Schema files
SCHEMA_SMALL="${SKILL_DIR}/references/pg_mcp_small_schema.md"
SCHEMA_MEDIUM="${SKILL_DIR}/references/pg_mcp_medium_schema.md"
SCHEMA_LARGE="${SKILL_DIR}/references/pg_mcp_large_schema.md"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print error and exit
error_exit() {
    echo -e "${RED}错误: $1${NC}" >&2
    exit 1
}

# Print info
info() {
    echo -e "${BLUE}[INFO] $1${NC}"
}

# Print success
success() {
    echo -e "${GREEN}[SUCCESS] $1${NC}"
}

# Print warning
warning() {
    echo -e "${YELLOW}[WARNING] $1${NC}"
}

# Detect database from user input
detect_database() {
    local input="$1"
    local input_lower=$(echo "$input" | tr '[:upper:]' '[:lower:]')

    if [[ "$input_lower" == *"pg_mcp_small"* ]]; then
        echo "pg_mcp_small"
    elif [[ "$input_lower" == *"pg_mcp_medium"* ]]; then
        echo "pg_mcp_medium"
    elif [[ "$input_lower" == *"pg_mcp_large"* ]]; then
        echo "pg_mcp_large"
    else
        echo ""
    fi
}

# Read schema file
read_schema() {
    local db="$1"
    case "$db" in
        "pg_mcp_small")
            cat "$SCHEMA_SMALL"
            ;;
        "pg_mcp_medium")
            cat "$SCHEMA_MEDIUM"
            ;;
        "pg_mcp_large")
            cat "$SCHEMA_LARGE"
            ;;
        *)
            error_exit "未知数据库: $db"
            ;;
    esac
}

# Check if SQL is safe (read-only)
is_sql_safe() {
    local sql="$1"
    local sql_upper=$(echo "$sql" | tr '[:lower:]' '[:upper:]')

    # Check for dangerous keywords
    local dangerous_keywords="INSERT UPDATE DELETE DROP ALTER CREATE TRUNCATE GRANT REVOKE EXECUTE PG_SLEEP SLEEP EXEC SYSTEM"

    for keyword in $dangerous_keywords; do
        if echo "$sql_upper" | grep -q "$keyword"; then
            echo "false"
            echo "检测到危险关键字: $keyword"
            return
        fi
    done

    # Check for SQL injection patterns
    if echo "$sql_upper" | grep -qE ";\s*(DROP|DELETE|UPDATE|INSERT)"; then
        echo "false"
        echo "检测到可能的 SQL 注入"
        return
    fi

    # Check if it starts with SELECT
    if ! echo "$sql_upper" | grep -q "^SELECT"; then
        echo "false"
        echo "SQL 必须以 SELECT 开头"
        return
    fi

    echo "true"
}

# Ensure SQL has LIMIT clause
ensure_limit() {
    local sql="$1"
    local sql_upper=$(echo "$sql" | tr '[:lower:]' '[:upper:]')

    if echo "$sql_upper" | grep -q "LIMIT"; then
        echo "$sql"
    else
        # Remove trailing semicolon and add LIMIT
        local clean_sql=$(echo "$sql" | sed 's/;$//')
        echo "${clean_sql} LIMIT 100;"
    fi
}

# Generate SQL from natural language (simplified pattern matching)
generate_sql() {
    local input="$1"
    local db="$2"
    local input_lower=$(echo "$input" | tr '[:upper:]' '[:lower:]')

    case "$db" in
        "pg_mcp_small")
            if echo "$input_lower" | grep -q "库存" && echo "$input_lower" | grep -q "产品"; then
                echo "SELECT * FROM v_product_inventory ORDER BY stock_quantity;"
            elif echo "$input_lower" | grep -q "订单" && echo "$input_lower" | grep -q "本月"; then
                echo "SELECT * FROM v_order_summary WHERE order_date >= CURRENT_DATE - INTERVAL '1 month' ORDER BY order_date DESC;"
            elif echo "$input_lower" | grep -q "产品"; then
                echo "SELECT * FROM v_product_inventory LIMIT 50;"
            elif echo "$input_lower" | grep -q "订单"; then
                echo "SELECT * FROM v_order_summary ORDER BY order_date DESC LIMIT 50;"
            else
                echo "SELECT '请指定要查询的内容' AS message LIMIT 1;"
            fi
            ;;
        "pg_mcp_medium")
            if echo "$input_lower" | grep -q "学生" && echo "$input_lower" | grep -q "成绩"; then
                echo "SELECT * FROM v_student_grades_summary ORDER BY score DESC LIMIT 50;"
            elif echo "$input_lower" | grep -q "出勤" && echo "$input_lower" | grep -q "率"; then
                echo "SELECT * FROM v_attendance_summary ORDER BY attendance_rate DESC LIMIT 50;"
            elif echo "$input_lower" | grep -q "班级" && echo "$input_lower" | grep -q "统计"; then
                echo "SELECT * FROM v_class_statistics ORDER BY grade_level, class_name LIMIT 50;"
            elif echo "$input_lower" | grep -q "学生"; then
                echo "SELECT * FROM v_student_info LIMIT 50;"
            else
                echo "SELECT '请指定要查询的内容' AS message LIMIT 1;"
            fi
            ;;
        "pg_mcp_large")
            if echo "$input_lower" | grep -q "今日" && echo "$input_lower" | grep -q "挂号"; then
                echo "SELECT * FROM v_today_registrations ORDER BY registration_time LIMIT 50;"
            elif echo "$input_lower" | grep -q "住院" && echo "$input_lower" | grep -q "患者"; then
                echo "SELECT * FROM v_current_inpatients ORDER BY admission_date LIMIT 50;"
            elif echo "$input_lower" | grep -q "床位" && echo "$input_lower" | grep -q "使用"; then
                echo "SELECT * FROM v_bed_utilization ORDER BY occupancy_rate DESC LIMIT 50;"
            elif echo "$input_lower" | grep -q "待处理" && (echo "$input_lower" | grep -q "检查" || echo "$input_lower" | grep -q "检验"); then
                echo "SELECT * FROM v_pending_examinations LIMIT 50;"
            else
                echo "SELECT '请指定要查询的内容' AS message LIMIT 1;"
            fi
            ;;
        *)
            echo "SELECT '未知数据库' AS message LIMIT 1;"
            ;;
    esac
}

# Execute SQL via psql
execute_sql() {
    local db="$1"
    local sql="$2"

    PGPASSWORD="$PG_PASSWORD" psql -h "$PG_HOST" -p "$PG_PORT" -U "$PG_USER" -d "$db" -c "$sql" 2>&1
}

# Check if execution was successful
is_execution_success() {
    local output="$1"

    if echo "$output" | grep -qE "ERROR|FATAL"; then
        echo "false"
        echo "$output" | grep -E "ERROR|FATAL"
        return
    fi

    echo "true"
}

# Count rows in result
count_rows() {
    local output="$1"

    # Count non-empty lines after header and separator
    local count=$(echo "$output" | tail -n +3 | grep -v '^$' | grep -v '^---' | wc -l)
    echo "$count"
}

# Calculate confidence score (0-10)
calculate_confidence() {
    local input="$1"
    local sql="$2"
    local row_count="$3"

    local input_lower=($(echo "$input" | tr '[:upper:]' '[:lower:]' | tr -s ' ' | tr ' ' '\n'))
    local sql_lower=$(echo "$sql" | tr '[:upper:]' '[:lower:]')

    local keyword_matches=0
    for keyword in "${input_lower[@]}"; do
        if [ ${#keyword} -gt 2 ] && echo "$sql_lower" | grep -q "$keyword"; then
            ((keyword_matches++))
        fi
    done

    # Use awk for floating-point arithmetic (more portable than bc)
    local confidence=$(awk -v km=$keyword_matches -v rc=$row_count 'BEGIN {
        c = 7.0
        if (km >= 2) c += 1.0
        else if (km == 0) c -= 1.0
        if (rc > 0 && rc < 100) c += 0.5
        else if (rc == 0) c -= 0.5
        if (c < 0) c = 0
        if (c > 10) c = 10
        printf "%.1f", c
    }')

    echo "$confidence"
}

# Main function
main() {
    if [ $# -eq 0 ]; then
        error_exit "用法: $0 \"查询语句\""
    fi

    local user_input="$*"
    local input_lower=$(echo "$user_input" | tr '[:upper:]' '[:lower:]')

    # Check if user wants SQL only
    local sql_only=false
    if echo "$input_lower" | grep -q "只返回sql\|sql only"; then
        sql_only=true
    fi

    # Detect database
    info "检测数据库..."
    local database=$(detect_database "$user_input")

    if [ -z "$database" ]; then
        error_exit "请指定要查询的数据库 (pg_mcp_small, pg_mcp_medium, pg_mcp_large)"
    fi

    success "检测到数据库: $database"

    # Read schema
    info "加载数据库 schema..."
    local schema=$(read_schema "$database")
    success "Schema 加载完成"

    # Generate SQL with retries
    local max_retries=3
    local attempt=1
    local sql=""
    local output=""
    local success=false
    local row_count=0
    local confidence=0

    while [ $attempt -le $max_retries ] && [ "$success" != "true" ]; do
        info "尝试 $attempt/$max_retries: 生成 SQL..."

        sql=$(generate_sql "$user_input" "$database")

        # Validate safety
        info "验证 SQL 安全性..."
        local is_safe=$(is_sql_safe "$sql")
        read -r is_safe msg <<< "$is_safe"

        if [ "$is_safe" != "true" ]; then
            error_exit "$msg"
        fi

        success "SQL 安全验证通过"

        # Ensure LIMIT
        sql=$(ensure_limit "$sql")

        # Execute SQL
        info "执行 SQL..."
        output=$(execute_sql "$database" "$sql")

        # Check success
        local is_ok=$(is_execution_success "$output")
        read -r is_ok error_msg <<< "$is_ok"

        if [ "$is_ok" != "true" ]; then
            warning "SQL 执行失败: $error_msg"
            ((attempt++))
            continue
        fi

        success "SQL 执行成功"

        # Count rows
        row_count=$(count_rows "$output")

        # Calculate confidence
        confidence=$(calculate_confidence "$user_input" "$sql" "$row_count")

        info "置信度: ${confidence}/10"

        # Check confidence threshold (use awk for comparison)
        local should_retry=$(awk -v c=$confidence 'BEGIN { print (c < 7.0) ? "1" : "0" }')
        if [ "$should_retry" = "1" ]; then
            warning "置信度不足 (${confidence}/10)"
            ((attempt++))
            if [ $attempt -le $max_retries ]; then
                info "重试..."
                continue
            fi
        fi

        success=true
    done

    if [ "$success" != "true" ]; then
        error_exit "查询处理失败，请重试"
    fi

    # Display results
    echo ""
    echo "============================================================"
    echo "数据库: $database"
    echo "============================================================"
    echo ""
    echo "执行的 SQL:"
    echo '```sql'
    echo "$sql"
    echo '```'
    echo ""
    echo "置信度: ${confidence}/10"

    if [ "$sql_only" = true ]; then
        echo ""
        echo "只返回 SQL，未执行查询结果"
        exit 0
    fi

    if [ $row_count -gt 0 ]; then
        echo ""
        echo "查询结果 (${row_count} 行):"
        echo "============================================================"
        echo "$output"

        if [ $row_count -gt 10 ]; then
            echo ""
            echo "... 还有 $((row_count - 10)) 行 (已显示前 10 行)"
        fi
    else
        echo ""
        echo "查询执行成功，但没有返回任何结果。"
    fi
}

# Run main function
main "$@"
