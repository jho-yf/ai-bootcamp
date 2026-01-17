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
    <div className={cn("flex flex-row items-center gap-12", className)}>
      <div className="flex items-center gap-6">
        <Label className="text-base font-medium text-foreground tracking-[-0.02em]">
          完成状态
        </Label>
        <div className="flex gap-2">
          <Button
            variant={showCompleted === null ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(null)}
            className="h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            全部
          </Button>
          <Button
            variant={showCompleted === true ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(true)}
            className="h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            已完成
          </Button>
          <Button
            variant={showCompleted === false ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(false)}
            className="h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            未完成
          </Button>
        </div>
      </div>

      <div className="flex items-center gap-6">
        <Label className="text-base font-medium text-foreground tracking-[-0.02em]">
          标签筛选
        </Label>
        <Select
          value={selectedTagId || "all"}
          onValueChange={(value) => onTagChange(value === "all" ? null : value)}
        >
          <SelectTrigger className="w-[220px] h-auto py-2 px-4 text-sm border-border transition-all duration-300 ease">
            <SelectValue placeholder="选择标签" />
          </SelectTrigger>
          <SelectContent>
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
