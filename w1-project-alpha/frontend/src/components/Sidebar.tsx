import { Button } from "./ui/button";
import { TagBadge } from "./TagBadge";
import { Plus, Settings } from "lucide-react";
import { cn } from "@/lib/utils";
import type { Tag } from "@/types";

interface SidebarProps {
  tags: Tag[];
  selectedTagId: string | null;
  onTagClick: (tagId: string | null) => void;
  onCreateTagClick: () => void;
  onManageTagsClick: () => void;
  className?: string;
}

/**
 * Sidebar 组件
 * 侧边栏组件，显示标签列表和操作按钮
 */
export function Sidebar({
  tags,
  selectedTagId,
  onTagClick,
  onCreateTagClick,
  onManageTagsClick,
  className,
}: SidebarProps) {
  return (
    <div className={cn("flex flex-col h-full", className)}>
      <div className="p-6 border-b border-border">
        <h2 className="text-2xl font-medium mb-6 tracking-[-0.02em] text-foreground">标签</h2>
        <div className="flex flex-col gap-2">
          <Button
            variant={selectedTagId === null ? "default" : "ghost"}
            size="sm"
            onClick={() => onTagClick(null)}
            className="justify-start h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            全部
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onCreateTagClick}
            className="justify-start h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            <Plus className="h-4 w-4 mr-2" />
            创建标签
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onManageTagsClick}
            className="justify-start h-auto py-2 px-4 text-sm font-normal transition-all duration-300 ease"
          >
            <Settings className="h-4 w-4 mr-2" />
            管理标签
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="space-y-2">
          {tags.length === 0 ? (
            <p className="text-sm text-muted-foreground text-center py-12 font-normal">暂无标签</p>
          ) : (
            tags.map((tag) => (
              <div
                key={tag.id}
                className={cn(
                  "p-4 cursor-pointer transition-all duration-300 ease",
                  selectedTagId === tag.id ? "bg-accent" : "hover:bg-accent"
                )}
                onClick={() => onTagClick(tag.id)}
              >
                <TagBadge tag={tag} />
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
