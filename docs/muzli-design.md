# Muzli 设计系统 - 完整复刻指南

基于对 muz.li 的深度分析，以下是可复刻其核心设计风格的系统化设计规范：

---

## 🎨 核心设计哲学

**极简主义 × 功能性 × 灵感驱动**
- "内容即设计" - 让设计作品本身成为视觉焦点
- 深色沉浸式体验，减少视觉疲劳
- 信息密度低，呼吸感强
- 精致的微交互与悬停效果

---

## 📐 布局系统

### 网格与容器
```css
/* 主容器 */
.container {
  max-width: 1440px;
  margin: 0 auto;
  padding: 0 80px; /* 超宽左右留白 */
}

/* 12列网格系统 */
.grid {
  display: grid;
  grid-template-columns: repeat(12, 1fr);
  gap: 40px; /* 卡片间距 */
}
```

### 间距系统（基于 8px 基准）
```css
/* Margin 层级 */
.margin-xs { margin: 8px; }
.margin-sm { margin: 16px; }
.margin-md { margin: 24px; }
.margin-lg { margin: 40px; }
.margin-xl { margin: 64px; }
.margin-xxl { margin: 96px; }

/* Padding 层级 */
.padding-xs { padding: 12px; }
.padding-sm { padding: 20px; }
.padding-md { padding: 32px; }
.padding-lg { padding: 48px; }
.padding-xl { padding: 72px; }
```

**关键规则**：
- 页面级上下留白最小 `96px`
- 区块间间距 `64px`
- 卡片内边距 `32px`
- 元素间间距 `24px`

---

## 🖌️ 色彩系统

### 主色调
```css
:root {
  /* 主背景 - 极深蓝黑 */
  --color-bg-primary: #090a12;
  
  /* 卡片背景 - 深色灰 */
  --color-bg-card: #1a1c23;
  
  /* 文本主色 */
  --color-text-primary: #f4f4f5; /* 纯白偏暖 */
  --color-text-secondary: #a0a0a0; /* 次要信息 */
  --color-text-tertiary: #666666; /* 时间戳等 */
  
  /* 点缀色 - 强调重要元素 */
  --color-accent: #f4f66a; /* Muzli 标志性黄绿 */
  --color-accent-hover: #ffff8a;
  
  /* 边框与分隔线 */
  --color-border: #2a2d48;
  
  /* 成功/状态色 */
  --color-success: #6d9171;
  --color-warning: #d8ab49;
  --color-error: #b62b20;
}
```

### 渐变应用
```css
/* 品牌渐变 - 极低饱和度 */
.gradient-subtle {
  background: linear-gradient(
    135deg, 
    rgba(244, 246, 106, 0.08) 0%, 
    rgba(109, 145, 113, 0.04) 100%
  );
}
```

---

## 🔤 字体系统

### 字体栈
```css
:root {
  --font-primary: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  --font-mono: 'SF Mono', Monaco, 'Cascadia Code', monospace;
}
```

### 字阶与行高
```css
/* 标题层级 */
.text-hero { 
  font-size: 72px; 
  line-height: 1.05; 
  font-weight: 800;
  letter-spacing: -0.03em;
}

.text-h1 { 
  font-size: 56px; 
  line-height: 1.1; 
  font-weight: 700;
  letter-spacing: -0.02em;
}

.text-h2 { 
  font-size: 40px; 
  line-height: 1.2; 
  font-weight: 650;
}

.text-h3 { 
  font-size: 28px; 
  line-height: 1.3; 
  font-weight: 600;
}

/* 正文 */
.text-body-lg { 
  font-size: 18px; 
  line-height: 1.6; 
  font-weight: 400;
}

.text-body { 
  font-size: 16px; 
  line-height: 1.6; 
  font-weight: 400;
}

.text-body-sm { 
  font-size: 14px; 
  line-height: 1.5; 
  font-weight: 400;
}

.text-caption { 
  font-size: 12px; 
  line-height: 1.4; 
  font-weight: 500;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}
```

---

## 🎴 组件系统

### 1. 导航栏
```css
.navbar {
  position: fixed;
  top: 0;
  width: 100%;
  padding: 20px 80px;
  background: rgba(9, 10, 18, 0.85);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--color-border);
  z-index: 1000;
}
```

### 2. 灵感卡片（核心组件）
```css
.inspiration-card {
  background: var(--color-bg-card);
  border-radius: 16px;
  padding: 32px;
  border: 1px solid var(--color-border);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.inspiration-card:hover {
  transform: translateY(-4px);
  border-color: var(--color-accent);
  box-shadow: 0 12px 32px rgba(244, 246, 106, 0.12);
}
```

### 3. 按钮系统
```css
/* 主按钮 */
.btn-primary {
  padding: 16px 32px;
  background: var(--color-accent);
  color: var(--color-bg-primary);
  border-radius: 12px;
  font-weight: 600;
  font-size: 16px;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary:hover {
  background: var(--color-accent-hover);
  transform: scale(1.02);
}

/* 幽灵按钮 */
.btn-ghost {
  padding: 16px 32px;
  background: transparent;
  color: var(--color-text-primary);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  font-weight: 600;
  transition: all 0.2s ease;
}

.btn-ghost:hover {
  border-color: var(--color-accent);
  color: var(--color-accent);
}
```

### 4. 标签/分类
```css
.tag {
  display: inline-block;
  padding: 8px 16px;
  background: rgba(244, 246, 106, 0.1);
  color: var(--color-accent);
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  border: 1px solid rgba(244, 246, 106, 0.2);
}
```

---

## 📱 响应式断点
```css
--breakpoint-sm: 640px;
--breakpoint-md: 768px;
--breakpoint-lg: 1024px;
--breakpoint-xl: 1280px;
```

**适配策略**：
- **>1280px**：标准桌面布局
- **1024-1280px**：减少左右 padding 至 60px
- **768-1024px**：转为 8px 网格，卡片单列显示
- **<768px**：padding 缩减至 24px，字体尺寸整体下调 10%

---

## ✨ 微交互细节

### 悬停效果
```css
/* 链接悬停 */
a {
  color: var(--color-text-primary);
  text-decoration: none;
  position: relative;
  transition: color 0.2s ease;
}

a::after {
  content: '';
  position: absolute;
  bottom: -2px;
  left: 0;
  width: 0;
  height: 2px;
  background: var(--color-accent);
  transition: width 0.3s ease;
}

a:hover::after {
  width: 100%;
}

a:hover {
  color: var(--color-accent);
}
```

### 加载动画
```css
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.loading-skeleton {
  background: linear-gradient(
    90deg,
    var(--color-bg-card) 0%,
    #2a2d48 50%,
    var(--color-bg-card) 100%
  );
  background-size: 200% 100%;
  animation: pulse 1.5s ease-in-out infinite;
}
```

---

## 🎯 实现建议

1. **使用 CSS Variables**：确保全局色彩一致性
2. **安装 Inter 字体**：这是 Muzli 视觉识别的关键
3. **注重性能**：深色背景配合大量图片，需优化加载策略
4. **无障碍性**：确保文本对比度 ≥ 4.5:1
5. **微交互库**：推荐使用 Framer Motion 或 GSAP 实现流畅动画

---

## 📋 页面模板示例

### Hero 区域
```html
<section class="hero" style="padding: 96px 0; text-align: center;">
  <h1 class="text-hero" style="margin-bottom: 24px;">
    Designers' Secret Source
  </h1>
  <p class="text-body-lg" style="color: var(--color-text-secondary); max-width: 640px; margin: 0 auto 40px;">
    The best design inspiration - expertly curated for you.
  </p>
  <button class="btn-primary">Get Muzli for Chrome</button>
</section>
```

此设计系统完整复刻了 Muzli 的**极简美学、深色基调、精致间距**和**灵感导向**的设计语言，可直接用于项目开发。
