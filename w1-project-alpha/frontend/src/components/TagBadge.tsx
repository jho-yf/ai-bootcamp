import { Badge } from "./ui/badge";
import { cn } from "@/lib/utils";
import type { Tag } from "@/types";

interface TagBadgeProps {
  tag: Tag;
  onClick?: () => void;
  variant?: "default" | "secondary" | "destructive" | "outline";
  className?: string;
}

/**
 * TagBadge 组件
 * 显示标签名称和颜色
 */
export function TagBadge({ tag, onClick, variant = "default", className }: TagBadgeProps) {
  const style = tag.color
    ? {
        backgroundColor: tag.color,
        color: getContrastColor(tag.color),
        borderColor: tag.color,
      }
    : undefined;

  return (
    <Badge
      variant={variant}
      className={cn(
        "cursor-pointer transition-all duration-300 hover:opacity-90 hover:scale-110 active:scale-95",
        className
      )}
      style={style}
      onClick={onClick}
    >
      {tag.name}
    </Badge>
  );
}

/**
 * 根据背景色计算对比色（黑色或白色）
 */
function getContrastColor(hexColor: string): string {
  // 移除 # 号
  const hex = hexColor.replace("#", "");
  const r = parseInt(hex.substr(0, 2), 16);
  const g = parseInt(hex.substr(2, 2), 16);
  const b = parseInt(hex.substr(4, 2), 16);
  // 计算亮度
  const brightness = (r * 299 + g * 587 + b * 114) / 1000;
  return brightness > 128 ? "#000000" : "#ffffff";
}
