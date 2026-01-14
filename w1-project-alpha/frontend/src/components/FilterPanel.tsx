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
    <div className={cn("flex flex-col gap-4 p-4 border rounded-lg bg-card", className)}>
      <div className="space-y-2">
        <Label>完成状态</Label>
        <div className="flex gap-2">
          <Button
            variant={showCompleted === null ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(null)}
          >
            全部
          </Button>
          <Button
            variant={showCompleted === true ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(true)}
          >
            已完成
          </Button>
          <Button
            variant={showCompleted === false ? "default" : "outline"}
            size="sm"
            onClick={() => onCompletedChange(false)}
          >
            未完成
          </Button>
        </div>
      </div>

      <div className="space-y-2">
        <Label>标签筛选</Label>
        <Select
          value={selectedTagId || "all"}
          onValueChange={(value) => onTagChange(value === "all" ? null : value)}
        >
          <SelectTrigger>
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
