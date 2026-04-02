# pg_mcp_small - 电商平台数据库 Schema

## 概述

小型电商订单管理系统，包含5张表、2个视图、10个索引、2个自定义类型。

## 自定义类型类型 (Enums)

### order_status
订单状态枚举：`pending`, `confirmed`, `shipped`, `delivered`, `cancelled`

### payment_method
支付方式枚举：`credit_card`, `alipay`, `wechat`, `bank_transfer`

## 表结构 (Tables)

### categories
产品分类表
- `id`: integer (NOT NULL, PRIMARY KEY) - 分类ID
- `name`: character varying (NOT NULL) - 分类名称
- `description`: text - 分类描述
- `parent_id`: integer - 父分类ID

### customers
客户表
- `id`: integer (NOT NULL, PRIMARY KEY) - 客户ID
- `email`: character varying (NOT NULL, UNIQUE) - 客户邮箱
- `name`: character varying (NOT NULL) - 客户姓名
- `phone`: character varying - 电话号码
- `city`: character varying - 城市
- `registered_at`: timestamp without time zone - 注册时间 (默认: CURRENT_TIMESTAMP)
- `status`: character varying - 客户状态 (默认: 'active')

### orders
订单表
- `id`: integer (NOT NULL, PRIMARY KEY) - 订单ID
- `customer_id`: integer - 客户ID (外键: customers.id)
- `order_date`: date - 订单日期 (默认: CURRENT_DATE)
- `status`: order_status - 订单状态 (默认: 'pending')
- `payment_method`: payment_method - 支付方式
- `total_amount`: numeric - 订单总金额
- `shipping_address`: text - 配送地址
- `notes`: text - 备注

### order_items
订单明细表
- `id`: integer (NOT NULL, PRIMARY KEY) - 订单明细ID
- `order_id`: integer - 订单ID (外键: orders.id)
- `product_id`: integer - 产品ID (外键: products.id)
- `quantity`: integer (NOT NULL) - 数量
- `unit_price`: numeric (NOT NULL) - 单价
- `subtotal`: numeric - 小计

### products
产品表
- `id`: integer (NOT NULL, PRIMARY KEY) - 产品ID
- `name`: character varying (NOT NULL) - 产品名称
- `category_id`: integer - 分类ID (外键: categories.id)
- `price`: numeric (NOT NULL) - 产品价格
- `stock_quantity`: integer - 库存数量 (默认: 0)
- `description`: text - 产品描述
- `created_at`: timestamp without time zone - 创建时间 (默认: CURRENT_TIMESTAMP)

## 视图 (Views)

### v_product_inventory
产品库存视图，包含以下列：
- `id`: 产品ID
- `name`: 产品名称
- `category_name`: 分类名称
- `price`: 产品价格
- `stock_quantity`: 库存数量
- `stock_status`: 库存状态 ('缺货', '库存紧张', '库存充足')

### v_order_summary
订单汇总视图，包含以下列：
- `id`: 订单ID
- `order_date`: 订单日期
- `customer_name`: 客户姓名
- `email`:` 客户邮箱
- `status`: 订单状态
- `item_count`: 订单项目数量
- `total`_amount`: 订单总金额

## 索引 (Indexes)

- `categories_pkey`: PRIMARY KEY on categories(id)
- `customers_pkey`: PRIMARY KEY on customers(id)
- `customers_email_key`: UNIQUE on customers(email)
- `idx_customers_email`: INDEX on customers(email)
- `orders_pkey`: PRIMARY KEY on orders(id)
- `idx_orders_customer`: INDEX on orders(customer_id)
- `idx_orders_date`: INDEX on orders(order_date)
- `products_pkey`: PRIMARY KEY on products(id)
- `idx_products_category`: INDEX on products(category_id)
- `order_items_pkey`: PRIMARY KEY on order_items(id)

## 业务规则

1. 订单状态流转: pending -> confirmed -> shipped -> delivered (或 cancelled)
2. 库存状态: stock_quantity = 0 为 '缺货', 1-9 为 '库存紧张', >=10 为 '库存充足'
3. 订单总金额通过 order_items 的小计汇总计算
