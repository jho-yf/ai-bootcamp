import { Badge } from "./ui/badge";
import { TagBadge } from "./TagBadge";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Tag } from "@/types";

interface TagSelectorProps {
  tags: Tag[];
  selectedTagIds: string[];
  onSelectionChange: (tagIds: string[]) => void;
  className?: string;
}

/**
 * TagSelector 组件
 * 多选标签选择器，显示标签颜色
 */
export function TagSelector({
  tags,
  selectedTagIds,
  onSelectionChange,
  className,
}: TagSelectorProps) {
  const toggleTag = (tagId: string) => {
    if (selectedTagIds.includes(tagId)) {
      onSelectionChange(selectedTagIds.filter((id) => id !== tagId));
    } else {
      onSelectionChange([...selectedTagIds, tagId]);
    }
  };

  return (
    <div className={cn("space-y-2", className)}>
      <div className="flex flex-wrap gap-2">
        {tags.map((tag) => {
          const isSelected = selectedTagIds.includes(tag.id);
          return (
            <TagBadge
              key={tag.id}
              tag={tag}
              variant={isSelected ? "default" : "outline"}
              onClick={() => toggleTag(tag.id)}
              className={cn("transition-all", isSelected && "ring-2 ring-ring ring-offset-2")}
            />
          );
        })}
      </div>
      {selectedTagIds.length > 0 && (
        <div className="flex items-center gap-2 flex-wrap">
          <span className="text-sm text-muted-foreground">已选择:</span>
          {selectedTagIds.map((tagId) => {
            const tag = tags.find((t) => t.id === tagId);
            if (!tag) return null;
            return (
              <Badge key={tagId} variant="secondary" className="flex items-center gap-1">
                {tag.name}
                <button
                  onClick={() => toggleTag(tagId)}
                  className="ml-1 hover:opacity-70"
                  aria-label={`移除标签 ${tag.name}`}
                >
                  <X className="h-3 w-3" />
                </button>
              </Badge>
            );
          })}
        </div>
      )}
    </div>
  );
}
