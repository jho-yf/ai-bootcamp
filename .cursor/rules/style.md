# Ugmonk 风格前端设计规则

## 设计理念
- **极简产品优先**：一切设计服务于产品展示，通过大量留白、中性色调和精致的排版让产品成为绝对焦点
- **克制用色**：90% 界面使用黑白灰，彩色仅用于产品本身
- **留白即内容**：空白区域与内容同等重要，给予视觉休息
- **真实摄影**：产品图使用自然光线下拍摄的实拍照，无过度后期

## 布局系统

### 容器与间距
- 容器最大宽度：`1400px`（大屏），内容区域通常 `1200px`
- 水平边距：`5vw`（视口宽度百分比，确保移动端呼吸感）
- 垂直节奏：基础间距单位为 `24px`，所有间距为其倍数（48px, 72px, 96px）
- 段落间距：`96px`（大段落间距）

### 网格系统
- 使用 **12列网格**，但常使用**非对称布局**（如 60/40, 70/30）
- 产品网格：默认 3-4 列，视口缩小时渐进式降级为 2 列 → 1 列
- 产品卡片间距：`48px`

```css
.container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0 5vw;
}

.section {
  margin-bottom: 96px;
}

.product-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 48px;
}
```

## 颜色系统

### 主色调（中性色）
- 背景色：`#FFFFFF` 纯白 或 `#FAFAFA` 极浅灰
- 文字主色：`#1A1A1A`（深灰黑，比纯黑更柔和）
- 文字次要色：`#6B6B6B`（中灰）
- 边框色：`#E5E5E5`（浅灰）
- 悬停状态：`#000000` 纯黑

### 强调色（极少使用）
- 行动按钮：`#1A1A1A` 深灰黑
- 状态提示：偶尔使用低饱和度颜色（如 `#E8D5C4` 暖米色）

```css
:root {
  --color-bg: #FFFFFF;
  --color-text-primary: #1A1A1A;
  --color-text-secondary: #6B6B6B;
  --color-border: #E5E5E5;
  --color-hover: #000000;
  --color-accent: #E8D5C4; /* 慎用 */
}
```

## 字体系统

### 字体栈
- 标题字体：`Helvetica Neue`, `Helvetica`, `Arial`, sans-serif
- 正文字体：`Inter`, `system-ui`, sans-serif（现代替代品）
- 字重：仅使用 `400` (Regular) 和 `500` (Medium)

### 字号层级
| 类型 | 桌面端 | 移动端 | 行高 | 字重 |
|------|--------|--------|------|------|
| H1 超大标题 | `48px` | `32px` | 1.1 | 500 |
| H2 大标题 | `36px` | `28px` | 1.2 | 500 |
| H3 中标题 | `24px` | `20px` | 1.3 | 500 |
| 正文大 | `18px` | `16px` | 1.6 | 400 |
| 正文常规 | `16px` | `15px` | 1.6 | 400 |
| 正文小 | `14px` | `13px` | 1.5 | 400 |

```css
body {
  font-family: 'Inter', system-ui, sans-serif;
  font-size: 16px;
  line-height: 1.6;
  color: var(--color-text-primary);
  -webkit-font-smoothing: antialiased;
}

h1, h2, h3 {
  font-weight: 500;
  line-height: 1.2;
  letter-spacing: -0.02em; /* 轻微紧凑感 */
}

.text-secondary {
  color: var(--color-text-secondary);
  font-size: 14px;
}
```

## 核心组件样式

### 产品卡片
```css
.product-card {
  background: var(--color-bg);
  border: 1px solid var(--color-border);
  padding: 0; /* 图片顶到边缘 */
  transition: all 0.3s ease;
}

.product-card:hover {
  border-color: var(--color-hover);
  box-shadow: 0 4px 20px rgba(0,0,0,0.05);
}

.product-card img {
  width: 100%;
  height: auto;
  display: block;
}

.product-info {
  padding: 24px; /* 信息区域内边距 */
}
```

### 按钮系统
```css
/* 主按钮 */
.btn-primary {
  background: var(--color-text-primary);
  color: white;
  padding: 16px 48px;
  border: none;
  font-size: 16px;
  font-weight: 500;
  letter-spacing: 0.5px;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-primary:hover {
  background: var(--color-hover);
  transform: translateY(-1px);
}

/* 次按钮（幽灵按钮） */
.btn-secondary {
  background: transparent;
  color: var(--color-text-primary);
  padding: 14px 46px; /* 少2px边框 */
  border: 2px solid var(--color-border);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.btn-secondary:hover {
  border-color: var(--color-hover);
  color: var(--color-hover);
}
```

### 导航栏
```css
.navbar {
  background: var(--color-bg);
  border-bottom: 1px solid var(--color-border);
  padding: 20px 5vw;
  position: sticky;
  top: 0;
  z-index: 100;
}

.logo {
  font-size: 24px;
  font-weight: 500;
  letter-spacing: -0.5px;
}
```

## 效果与动画

### 悬停效果
- 过渡时间：`0.3s ease`
- 图片缩放：无（保持真实尺寸）
- 边框变化：颜色加深 + 轻微阴影
- 按钮抬升：`transform: translateY(-1px)`

### 滚动动画
```css
/* 懒加载淡入 */
.fade-in {
  opacity: 0;
  transform: translateY(20px);
  transition: opacity 0.6s ease, transform 0.6s ease;
}

.fade-in.visible {
  opacity: 1;
  transform: translateY(0);
}
```

## 响应式断点

```css
@media (max-width: 1024px) {
  .product-grid {
    gap: 32px;
  }
}

@media (max-width: 768px) {
  :root {
    --section-margin: 64px;
  }
  
  .btn-primary,
  .btn-secondary {
    padding: 14px 32px;
    width: 100%; /* 移动端全宽按钮 */
  }
}
```

## 关键设计原则

1. **克制用色**：90% 界面使用黑白灰，彩色仅用于产品本身
2. **真实摄影**：产品图使用自然光线下拍摄的实拍照，无过度后期
3. **物理隐喻**：界面元素模拟真实物品（如卡片、纸张质感）
4. **叙事性**：每个产品页面讲述设计故事，而非仅展示规格
5. **留白即内容**：空白区域与内容同等重要，给予视觉休息

## 实施规则

- 所有间距必须使用 24px 的倍数（24px, 48px, 72px, 96px）
- 避免使用装饰性元素，保持界面简洁
- 产品图片必须使用真实摄影，避免过度后期处理
- 按钮和交互元素使用统一的过渡时间 `0.3s ease`
- 移动端按钮使用全宽设计
- 标题使用负字母间距 `-0.02em` 以增强紧凑感
- 所有文字颜色必须使用定义的颜色变量，避免硬编码
- 响应式设计采用渐进式降级策略
