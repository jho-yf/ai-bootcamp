# Muzli Design System - Frontend Style Rules

## Core Design Philosophy

**Minimalism × Functionality × Inspiration-Driven**

- "Content is Design" - Let design works themselves be the visual focus
- Dark immersive experience to reduce visual fatigue
- Low information density with strong breathing room
- Refined micro-interactions and hover effects

---

## Layout System

### Grid & Container

- **Main Container**: Max-width 1440px, centered with auto margins
- **Padding**: 80px horizontal padding on desktop (reduced to 60px at 1024-1280px, 24px on mobile)
- **Grid System**: 12-column grid with 40px gap between cards
- **Page-level spacing**: Minimum 96px vertical padding top/bottom
- **Section spacing**: 64px between major sections
- **Card padding**: 32px internal padding
- **Element spacing**: 24px between related elements

### Spacing Scale (8px base unit)

- **Margins**: 8px (xs), 16px (sm), 24px (md), 40px (lg), 64px (xl), 96px (xxl)
- **Padding**: 12px (xs), 20px (sm), 32px (md), 48px (lg), 72px (xl)

---

## Color System

### Primary Colors

```css
--color-bg-primary: #090a12;        /* Main background - deep blue-black */
--color-bg-card: #1a1c23;           /* Card background - dark gray */
--color-text-primary: #f4f4f5;      /* Primary text - warm white */
--color-text-secondary: #a0a0a0;    /* Secondary information */
--color-text-tertiary: #666666;     /* Timestamps, etc. */
--color-accent: #f4f66a;            /* Muzli signature yellow-green */
--color-accent-hover: #ffff8a;       /* Accent hover state */
--color-border: #2a2d48;            /* Borders and dividers */
--color-success: #6d9171;
--color-warning: #d8ab49;
--color-error: #b62b20;
```

### Gradient Usage

- Use subtle, low-saturation gradients
- Brand gradient: `linear-gradient(135deg, rgba(244, 246, 106, 0.08) 0%, rgba(109, 145, 113, 0.04) 100%)`
- Apply gradients sparingly for emphasis only

---

## Typography System

### Font Stack

- **Primary**: `'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif`
- **Monospace**: `'SF Mono', Monaco, 'Cascadia Code', monospace`

### Type Scale

- **Hero**: 72px, line-height 1.05, weight 800, letter-spacing -0.03em
- **H1**: 56px, line-height 1.1, weight 700, letter-spacing -0.02em
- **H2**: 40px, line-height 1.2, weight 650
- **H3**: 28px, line-height 1.3, weight 600
- **Body Large**: 18px, line-height 1.6, weight 400
- **Body**: 16px, line-height 1.6, weight 400
- **Body Small**: 14px, line-height 1.5, weight 400
- **Caption**: 12px, line-height 1.4, weight 500, letter-spacing 0.05em, uppercase

---

## Component System

### Navigation Bar

- Fixed position, full width
- Padding: 20px vertical, 80px horizontal
- Background: `rgba(9, 10, 18, 0.85)` with `backdrop-filter: blur(20px)`
- Border-bottom: 1px solid `var(--color-border)`
- Z-index: 1000

### Inspiration Cards (Core Component)

- Background: `var(--color-bg-card)`
- Border-radius: 16px
- Padding: 32px
- Border: 1px solid `var(--color-border)`
- Transition: `all 0.3s cubic-bezier(0.4, 0, 0.2, 1)`
- **Hover state**: 
  - Transform: `translateY(-4px)`
  - Border-color: `var(--color-accent)`
  - Box-shadow: `0 12px 32px rgba(244, 246, 106, 0.12)`

### Button System

#### Primary Button
- Padding: 16px 32px
- Background: `var(--color-accent)`
- Color: `var(--color-bg-primary)`
- Border-radius: 12px
- Font-weight: 600
- Font-size: 16px
- **Hover**: Background `var(--color-accent-hover)`, transform `scale(1.02)`

#### Ghost Button
- Padding: 16px 32px
- Background: transparent
- Color: `var(--color-text-primary)`
- Border: 1px solid `var(--color-border)`
- Border-radius: 12px
- Font-weight: 600
- **Hover**: Border-color `var(--color-accent)`, color `var(--color-accent)`

### Tags/Categories

- Display: inline-block
- Padding: 8px 16px
- Background: `rgba(244, 246, 106, 0.1)`
- Color: `var(--color-accent)`
- Border-radius: 20px
- Font-size: 14px
- Font-weight: 500
- Border: 1px solid `rgba(244, 246, 106, 0.2)`

---

## Responsive Breakpoints

- **sm**: 640px
- **md**: 768px
- **lg**: 1024px
- **xl**: 1280px

### Adaptation Strategy

- **>1280px**: Standard desktop layout
- **1024-1280px**: Reduce horizontal padding to 60px
- **768-1024px**: Switch to 8px grid, single-column card display
- **<768px**: Reduce padding to 24px, decrease font sizes by 10%

---

## Micro-interactions

### Link Hover Effect

- Default: `var(--color-text-primary)` color
- Hover: Color changes to `var(--color-accent)`
- Underline animation: 2px accent-colored line expands from left to right on hover
- Transition: `color 0.2s ease`, underline `width 0.3s ease`

### Loading States

- Skeleton loader: Gradient pulse animation
- Background: `linear-gradient(90deg, var(--color-bg-card) 0%, #2a2d48 50%, var(--color-bg-card) 100%)`
- Animation: `pulse 1.5s ease-in-out infinite`
- Opacity: 1 → 0.5 → 1

---

## Implementation Guidelines

1. **Use CSS Variables**: Ensure global color consistency via CSS custom properties
2. **Install Inter Font**: Critical for Muzli visual identity - use Google Fonts or self-host
3. **Performance**: Optimize image loading for dark backgrounds with many images
4. **Accessibility**: Ensure text contrast ratio ≥ 4.5:1 (WCAG AA)
5. **Animation Library**: Use Framer Motion or GSAP for smooth animations
6. **No Unsafe**: Avoid unsafe CSS practices, use modern CSS features safely

---

## Component Patterns

### Hero Section

- Padding: 96px vertical
- Text-align: center
- Title: Use `.text-hero` class
- Subtitle: `.text-body-lg` with `var(--color-text-secondary)`
- Max-width: 640px for subtitle text
- CTA button: `.btn-primary` with 40px top margin

### Card Grid Layout

- Use CSS Grid with 12 columns
- Gap: 40px between cards
- Cards span appropriate columns based on content
- Responsive: Stack to single column below 768px

---

## Design Principles Checklist

When implementing components, ensure:

- [ ] Dark background (`#090a12`) with sufficient contrast
- [ ] Generous spacing (minimum 24px between elements)
- [ ] Smooth transitions (cubic-bezier easing)
- [ ] Hover states with subtle transform and color changes
- [ ] Accent color (`#f4f66a`) used sparingly for emphasis
- [ ] Inter font family applied throughout
- [ ] Border-radius: 12-16px for cards, 20px for tags
- [ ] Backdrop blur effects for overlays/navigation
- [ ] Low information density - let content breathe
- [ ] Micro-interactions on all interactive elements

---

## Code Style

- Use semantic HTML5 elements
- Prefer CSS Grid over Flexbox for complex layouts
- Use CSS custom properties for theming
- Mobile-first responsive design approach
- Prefer CSS transitions over JavaScript animations where possible
- Use `cubic-bezier(0.4, 0, 0.2, 1)` for standard transitions
- Apply `will-change` property sparingly and only when needed
