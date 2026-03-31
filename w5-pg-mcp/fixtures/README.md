# PostgreSQL MCP 测试数据库

本目录包含 PostgreSQL MCP Server 的测试数据库脚本和管理工具。

## 数据库概览

| 数据库名称 | 业务场景 | 规模 | 描述 |
|-----------|---------|------|------|
| `pg_mcp_small` | 电商平台 | 小型 | 5张表，2个视图，4个索引，2个类型，~100行数据 |
| `pg_mcp_medium` | 学校管理 | 中型 | 18张表，8个视图，15个索引，5个类型，~500行数据 |
| `pg_mcp_large` | 医院信息系统 | 大型 | 55+张表，25+视图，40+索引，12个类型，~2000行数据 |

## 快速开始

### 前置要求

- PostgreSQL 12+ 已安装并运行
- 有创建数据库的权限

### 使用 Makefile 管理数据库

```bash
# 查看帮助
make help

# 创建所有测试数据库
make all

# 查看数据库状态
make status

# 重建所有数据库
make rebuild-all

# 删除所有数据库
make drop-all
```

### 直接使用 SQL 脚本

```bash
# 创建小型数据库
psql -U postgres -f 01-small-ecommerce.sql

# 创建中型数据库
psql -U postgres -f 02-medium-school.sql

# 创建大型数据库
psql -U postgres -f 03-large-hospital.sql
```

## 数据库详情

### 1. pg_mcp_small - 电商平台

**业务场景**: 小型电商订单管理系统

**核心实体**:
- 客户 (customers)
- 产品 (products)
- 分类 (categories)
- 订单 (orders)
- 订单明细 (order_items)

**自定义类型**:
- `order_status`: 订单状态枚举
- `payment_method`: 支付方式枚举

**视图**:
- `v_product_inventory`: 产品库存视图
- `v_order_summary`: 订单汇总视图

### 2. pg_mcp_medium - 学校管理

**业务场景**: 学校综合管理系统

**核心实体**:
- 学生/教师/员工
- 班级/课程/科目
- 成绩/考勤/请假
- 奖惩记录/活动/设施

**自定义类型**:
- `gender_type`: 性别
- `grade_level`: 年级
- `attendance_status`: 考勤状态
- `leave_type`: 请假类型
- `employee_type`: 员工类型

**视图**:
- `v_student_info`: 学生基本信息
- `v_teacher_courses`: 教师授课视图
- `v_student_grades_summary`: 成绩汇总
- `v_class_statistics`: 班级统计
- `v_attendance_summary`: 出勤统计
- 等更多...

### 3. pg_mcp_large - 医院信息系统

**业务场景**: 综合医院信息系统 (HIS)

**核心模块**:
- 患者管理 (基本信息、档案、过敏史、病史)
- 门诊管理 (挂号、预约、病历、诊断)
- 住院管理 (病房、床位、入院、医嘱、护理)
- 检查检验 (申请、报告、结果)
- 药房管理 (药品、库存、处方)
- 手术管理 (手术室、申请、记录)
- 费用管理 (项目、明细、账单、支付)
- 急诊管理 (分诊、就诊记录)

**自定义类型**:
- `blood_type_enum`: 血型
- `gender_enum`: 性别
- `patient_type`: 患者类型
- `severity_level`: 严重程度
- `priority_level`: 优先级
- `payment_method`: 付费方式
- `surgery_status`: 手术状态
- `nursing_level`: 护理等级
- 等更多...

**视图**:
- `v_patient_basic_info`: 患者基本信息
- `v_today_registrations`: 今日挂号
- `v_current_inpatients`: 在院患者
- `v_doctor_schedule_today`: 医生今日排班
- `v_pending_examinations`: 待检查项目
- `v_bed_utilization`: 床位使用统计
- 等更多...

## Makefile 命令参考

### 数据库管理

```bash
make all              # 创建所有测试数据库
make create-small     # 创建小型数据库
make create-medium    # 创建中型数据库
make create-large     # 创建大型数据库
make rebuild-all      # 重建所有数据库
make drop-all         # 删除所有数据库
```

### 状态查看

```bash
make status           # 显示数据库状态
make describe-small   # 查看表结构
make views-small      # 查看视图
make types-small      # 查看自定义类型
```

### 测试查询

```bash
make test-small       # 小型数据库测试查询
make test-medium      # 中型数据库测试查询
make test-large       # 大型数据库测试查询
make test-all         # 运行所有测试查询
```

### 备份恢复

```bash
make backup-all       # 备份所有数据库
make restore-small    # 恢复小型数据库
```

### 自定义连接参数

```bash
make PG_HOST=myhost PG_PORT=5433 PG_USER=myuser status
```

## 测试查询示例

### 小型数据库 (电商)

```sql
-- 查询库存状态
SELECT * FROM v_product_inventory ORDER BY stock_quantity;

-- 查询本月订单
SELECT * FROM v_order_summary
WHERE order_date >= '2025-03-01'
ORDER BY order_date DESC;
```

### 中型数据库 (学校)

```sql
-- 查询班级统计
SELECT * FROM v_class_statistics
ORDER BY grade_level, class_name;

-- 查询出勤率低于95%的学生
SELECT * FROM v_attendance_summary
WHERE attendance_rate < 95
ORDER BY attendance_rate;
```

### 大型数据库 (医院)

```sql
-- 查询今日挂号
SELECT * FROM v_today_registrations
ORDER BY registration_time;

-- 查询在院患者
SELECT * FROM v_current_inpatients
ORDER BY admission_date;

-- 查询床位使用率
SELECT * FROM v_bed_utilization
ORDER BY occupancy_rate DESC;
```

## 文件结构

```
fixtures/
├── 01-small-ecommerce.sql      # 小型电商平台数据库
├── 02-medium-school.sql        # 中型学校管理数据库
├── 03-large-hospital.sql       # 大型医院信息系统数据库
├── Makefile                    # 数据库管理工具
└── README.md                   # 本文档
```

## 注意事项

1. 所有脚本执行时会删除已存在的同名数据库
2. 脚本中包含验证查询，执行完成后会显示数据库统计信息
3. 建议在开发/测试环境使用，不要在生产环境执行
4. 大型数据库脚本执行时间约10-30秒，属于正常现象

## 扩展建议

如需添加更多测试场景：

1. 创建新的 SQL 文件 (如 `04-custom.sql`)
2. 在 Makefile 中添加相应的目标
3. 更新本文档的数据库概览表
