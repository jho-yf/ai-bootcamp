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
      <div className="p-8 border-b border-border/30">
        <h2 className="text-[24px] font-semibold mb-8 tracking-[-0.02em] text-foreground">标签</h2>
        <div className="flex flex-col gap-2.5">
          <Button
            variant={selectedTagId === null ? "default" : "ghost"}
            size="sm"
            onClick={() => onTagClick(null)}
            className="justify-start h-11 text-[15px] font-medium rounded-xl transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
          >
            全部
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onCreateTagClick}
            className="justify-start h-11 text-[15px] font-medium rounded-xl transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
          >
            <Plus className="h-4 w-4 mr-2.5" />
            创建标签
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={onManageTagsClick}
            className="justify-start h-11 text-[15px] font-medium rounded-xl transition-all duration-300 hover:scale-[1.02] active:scale-[0.98]"
          >
            <Settings className="h-4 w-4 mr-2.5" />
            管理标签
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-8">
        <div className="space-y-2">
          {tags.length === 0 ? (
            <p className="text-[15px] text-muted-foreground text-center py-12 font-medium">
              暂无标签
            </p>
          ) : (
            tags.map((tag) => (
              <div
                key={tag.id}
                className={cn(
                  "p-4 rounded-2xl cursor-pointer transition-all duration-300",
                  selectedTagId === tag.id
                    ? "bg-accent shadow-sm scale-[1.02]"
                    : "hover:bg-accent/60 hover:scale-[1.01] active:scale-[0.99]"
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
