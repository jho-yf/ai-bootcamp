import { Card, CardContent, CardFooter, CardHeader } from "./ui/card";
import { Button } from "./ui/button";
import { TagBadge } from "./TagBadge";
import { CheckCircle2, Circle, Edit, Trash2 } from "lucide-react";
import type { TicketWithTags } from "@/types";

interface TicketCardProps {
  ticket: TicketWithTags;
  onToggleCompleted?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
  onTagClick?: (tagId: string) => void;
}

/**
 * TicketCard 组件
 * 显示 Ticket 的标题、描述、标签和完成状态
 * 提供编辑、删除和切换完成状态的操作按钮
 */
export function TicketCard({
  ticket,
  onToggleCompleted,
  onEdit,
  onDelete,
  onTagClick,
}: TicketCardProps) {
  return (
    <Card className="group cursor-pointer transition-all duration-300 ease">
      <CardHeader className="pb-0">
        <div className="flex items-start justify-between gap-6">
          <div className="flex-1 min-w-0">
            <h3
              className={`text-2xl font-medium leading-[1.2] tracking-[-0.02em] ${
                ticket.completed
                  ? "line-through text-muted-foreground opacity-50"
                  : "text-foreground"
              }`}
            >
              {ticket.title}
            </h3>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={(e) => {
              e.stopPropagation();
              onToggleCompleted?.(ticket.id);
            }}
            className="shrink-0 h-10 w-10 transition-all duration-300 ease hover:bg-accent"
            aria-label={ticket.completed ? "标记为未完成" : "标记为已完成"}
          >
            {ticket.completed ? (
              <CheckCircle2 className="h-5 w-5 text-foreground transition-all duration-300 ease" />
            ) : (
              <Circle className="h-5 w-5 text-muted-foreground transition-all duration-300 ease group-hover:text-foreground" />
            )}
          </Button>
        </div>
      </CardHeader>
      {ticket.description && (
        <CardContent className="pt-6 pb-0">
          <p className="text-sm text-muted-foreground leading-[1.5] whitespace-pre-wrap line-clamp-3">
            {ticket.description}
          </p>
        </CardContent>
      )}
      <CardFooter className="flex flex-col gap-6 pt-6 pb-0">
        {ticket.tags && ticket.tags.length > 0 && (
          <div className="flex flex-wrap gap-2 w-full">
            {ticket.tags.map((tag) => (
              <TagBadge key={tag.id} tag={tag} onClick={() => onTagClick?.(tag.id)} />
            ))}
          </div>
        )}
        <div className="flex gap-2 w-full justify-end pt-6 border-t border-border">
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onEdit?.(ticket.id);
            }}
            className="h-auto px-4 py-2 text-sm font-normal transition-all duration-300 ease hover:bg-accent"
          >
            <Edit className="h-4 w-4 mr-2" />
            编辑
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onDelete?.(ticket.id);
            }}
            className="h-auto px-4 py-2 text-sm font-normal text-foreground transition-all duration-300 ease hover:bg-accent"
          >
            <Trash2 className="h-4 w-4 mr-2" />
            删除
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}
