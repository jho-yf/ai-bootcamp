import { Button } from "./ui/button";
import { Label } from "./ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "./ui/select";
import { cn } from "@/lib/utils";
import type { Tag } from "@/types";

interface FilterPanelProps {
  tags: Tag[];
  selectedTagId: string | null;
  showCompleted: boolean | null;
  onTagChange: (tagId: string | null) => void;
  onCompletedChange: (completed: boolean | null) => void;
  className?: string;
}

/**
 * FilterPanel 组件
 * 筛选面板，支持完成状态和标签筛选
 */
export function FilterPanel({
  tags,
  selectedTagId,
  showCompleted,
  onTagChange,
  onCompletedChange,
  className,
}: FilterPanelProps) {
  return (
    <div className={cn("flex flex-row items-center gap-10", className)}>
      <div className="flex items-center gap-4">
        <Label className="text-[15px] font-semibold text-foreground tracking-[-0.01em]">
          完成状态
        </Label>
        <div className="flex gap-2.5">
          <Button
            variant={showCompleted === null ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(null)}
            className="h-10 px-5 text-[14px] font-medium rounded-xl transition-all duration-300"
          >
            全部
          </Button>
          <Button
            variant={showCompleted === true ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(true)}
            className="h-10 px-5 text-[14px] font-medium rounded-xl transition-all duration-300"
          >
            已完成
          </Button>
          <Button
            variant={showCompleted === false ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(false)}
            className="h-10 px-5 text-[14px] font-medium rounded-xl transition-all duration-300"
          >
            未完成
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <Label className="text-[15px] font-semibold text-foreground tracking-[-0.01em]">
          标签筛选
        </Label>
        <Select
          value={selectedTagId || "all"}
          onValueChange={(value) => onTagChange(value === "all" ? null : value)}
        >
          <SelectTrigger className="w-[220px] h-10 text-[14px] rounded-xl border-border/50 hover:border-border transition-all duration-300">
            <SelectValue placeholder="选择标签" />
          </SelectTrigger>
          <SelectContent className="rounded-xl">
            <SelectItem value="all">全部标签</SelectItem>
            {tags.map((tag) => (
              <SelectItem key={tag.id} value={tag.id}>
                {tag.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );
}
