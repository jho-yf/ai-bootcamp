-- ============================================================================
-- Small E-commerce Database
-- 场景：小型电商系统，用于订单和产品管理
-- 规模：5张表，2个视图，4个索引，约100行测试数据
-- ============================================================================

-- Drop existing database
DROP DATABASE IF EXISTS pg_mcp_small;
CREATE DATABASE pg_mcp_small;

\c pg_mcp_small;

-- ----------------------------------------------------------------------------
-- Custom Types
-- ----------------------------------------------------------------------------

CREATE TYPE order_status AS ENUM ('pending', 'confirmed', 'shipped', 'delivered', 'cancelled');

CREATE TYPE payment_method AS ENUM ('credit_card', 'alipay', 'wechat', 'bank_transfer');

-- ----------------------------------------------------------------------------
-- Tables
-- ----------------------------------------------------------------------------

-- 用户表
CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    email VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    phone VARCHAR(20),
    city VARCHAR(50),
    registered_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(20) DEFAULT 'active'
);

-- 产品分类表
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    description TEXT,
    parent_id INTEGER REFERENCES categories(id)
);

-- 产品表
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    category_id INTEGER REFERENCES categories(id),
    price DECIMAL(10,2) NOT NULL,
    stock_quantity INTEGER DEFAULT 0,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 订单表
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER REFERENCES customers(id),
    order_date DATE DEFAULT CURRENT_DATE,
    status order_status DEFAULT 'pending',
    payment_method payment_method,
    total_amount DECIMAL(12,2),
    shipping_address TEXT,
    notes TEXT
);

-- 订单明细表
CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER REFERENCES orders(id) ON DELETE CASCADE,
    product_id INTEGER REFERENCES products(id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10,2) NOT NULL,
    subtotal DECIMAL(10,2) GENERATED ALWAYS AS (quantity * unit_price) STORED
);

-- ----------------------------------------------------------------------------
-- Indexes
-- ----------------------------------------------------------------------------

CREATE INDEX idx_customers_email ON customers(email);
CREATE INDEX idx_products_category ON products(category_id);
CREATE INDEX idx_orders_customer ON orders(customer_id);
CREATE INDEX idx_orders_date ON orders(order_date);

-- ----------------------------------------------------------------------------
-- Views
-- ----------------------------------------------------------------------------

-- 产品库存视图
CREATE VIEW v_product_inventory AS
SELECT
    p.id,
    p.name,
    c.name AS category_name,
    p.price,
    p.stock_quantity,
    CASE
        WHEN p.stock_quantity = 0 THEN '缺货'
        WHEN p.stock_quantity < 10 THEN '库存紧张'
        ELSE '库存充足'
    END AS stock_status
FROM products p
LEFT JOIN categories c ON p.category_id = c.id;

-- 订单汇总视图
CREATE VIEW v_order_summary AS
SELECT
    o.id,
    o.order_date,
    c.name AS customer_name,
    c.email,
    o.status,
    COUNT(oi.id) AS item_count,
    SUM(oi.subtotal) AS total_amount
FROM orders o
JOIN customers c ON o.customer_id = c.id
LEFT JOIN order_items oi ON o.id = oi.order_id
GROUP BY o.id, o.order_date, c.name, c.email, o.status;

-- ----------------------------------------------------------------------------
-- Sample Data
-- ----------------------------------------------------------------------------

-- 分类数据
INSERT INTO categories (name, description) VALUES
    ('电子产品', '各类电子设备和配件'),
    ('家用电器', '生活电器'),
    ('服装鞋帽', '衣物鞋类'),
    ('食品饮料', '食品和饮料'),
    ('图书文具', '图书和文具用品');

INSERT INTO categories (name, parent_id, description) VALUES
    ('手机', 1, '智能手机和配件'),
    ('电脑', 1, '笔记本和台式机'),
    ('小家电', 2, '小型家用电器');

-- 产品数据
INSERT INTO products (name, category_id, price, stock_quantity, description) VALUES
    ('iPhone 15 Pro', 6, 7999.00, 25, '苹果最新智能手机'),
    ('MacBook Air M3', 7, 8999.00, 15, '苹果笔记本电脑'),
    ('戴森吸尘器', 8, 2999.00, 30, '无线吸尘器'),
    ('空气炸锅', 8, 399.00, 50, '家用空气炸锅'),
    ('运动T恤', 3, 99.00, 100, '纯棉运动T恤'),
    ('牛仔裤', 3, 299.00, 80, '修身牛仔裤'),
    ('有机牛奶箱装', 4, 68.00, 200, '有机纯牛奶'),
    ('进口红酒', 4, 188.00, 60, '法国进口红酒'),
    ('Python编程书', 5, 89.00, 45, 'Python从入门到精通'),
    ('钢笔礼盒', 5, 128.00, 35, '高级钢笔套装');

-- 客户数据
INSERT INTO customers (email, name, phone, city) VALUES
    ('zhang.wei@example.com', '张伟', '13800138001', '北京'),
    ('li.na@example.com', '李娜', '13800138002', '上海'),
    ('wang.qiang@example.com', '王强', '13800138003', '深圳'),
    ('liu.yan@example.com', '刘艳', '13800138004', '杭州'),
    ('chen.min@example.com', '陈敏', '13800138005', '成都'),
    ('zhao.lei@example.com', '赵磊', '13800138006', '武汉'),
    ('sun.fang@example.com', '孙芳', '13800138007', '西安'),
    ('zhou.jie@example.com', '周杰', '13800138008', '南京');

-- 订单数据
INSERT INTO orders (customer_id, order_date, status, payment_method, total_amount, shipping_address) VALUES
    (1, '2025-03-15', 'delivered', 'credit_card', 7999.00, '北京市朝阳区xx路xx号'),
    (1, '2025-03-20', 'shipped', 'alipay', 498.00, '北京市朝阳区xx路xx号'),
    (2, '2025-03-18', 'delivered', 'wechat', 8999.00, '上海市浦东新区xx路xx号'),
    (2, '2025-03-25', 'confirmed', 'credit_card', 398.00, '上海市浦东新区xx路xx号'),
    (3, '2025-03-22', 'pending', 'alipay', 2999.00, '深圳市南山区xx路xx号'),
    (3, '2025-03-28', 'shipped', 'wechat', 8587.00, '深圳市南山区xx路xx号'),
    (4, '2025-03-26', 'delivered', 'credit_card', 188.00, '杭州市西湖区xx路xx号'),
    (5, '2025-03-29', 'confirmed', 'alipay', 596.00, '成都市武侯区xx路xx号'),
    (6, '2025-03-30', 'pending', 'wechat', 89.00, '武汉市洪山区xx路xx号'),
    (7, '2025-03-30', 'confirmed', 'bank_transfer', 10887.00, '西安市雁塔区xx路xx号');

-- 订单明细数据
INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES
    -- 订单1：iPhone
    (1, 1, 1, 7999.00),
    -- 订单2：T恤 + 牛仔裤
    (2, 5, 2, 99.00),
    (2, 6, 1, 299.00),
    -- 订单3：MacBook
    (3, 2, 1, 8999.00),
    -- 订单4：T恤 + 牛奶
    (4, 5, 1, 99.00),
    (4, 7, 3, 68.00),
    -- 订单5：戴森
    (5, 3, 1, 2999.00),
    -- 订单6：iPhone + 牛奶
    (6, 1, 1, 7999.00),
    (6, 7, 1, 68.00),
    -- 订单7：红酒
    (7, 8, 1, 188.00),
    -- 订单8：T恤 + 钢笔
    (8, 5, 3, 99.00),
    (8, 10, 1, 128.00),
    -- 订单9：Python书
    (9, 9, 1, 89.00),
    -- 订单10：MacBook + 空气炸锅 + 钢笔
    (10, 2, 1, 8999.00),
    (10, 4, 2, 399.00),
    (10, 10, 1, 128.00);

-- ----------------------------------------------------------------------------
-- Verification Query
-- ----------------------------------------------------------------------------

DO $$
DECLARE
    table_count INTEGER;
    view_count INTEGER;
    index_count INTEGER;
    type_count INTEGER;
    row_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO table_count FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';
    SELECT COUNT(*) INTO view_count FROM information_schema.views WHERE table_schema = 'public';
    SELECT COUNT(*) INTO index_count FROM pg_indexes WHERE schemaname = 'public';
    SELECT COUNT(*) INTO type_count FROM pg_type WHERE typnamespace = 'public'::regnamespace AND typtype = 'e';

    RAISE NOTICE '=== Small E-commerce Database Created ===';
    RAISE NOTICE 'Tables: %, Views: %, Indexes: %, Custom Types: %', table_count, view_count, index_count, type_count;
END $$;
