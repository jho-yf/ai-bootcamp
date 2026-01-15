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
    <Card className="group cursor-pointer transition-all duration-500 ease-out hover:shadow-xl hover:-translate-y-2 hover:border-border/80 bg-card/95 backdrop-blur-sm">
      <CardHeader className="pb-5">
        <div className="flex items-start justify-between gap-4">
          <div className="flex-1 min-w-0">
            <h3
              className={`text-[19px] font-semibold leading-[1.3] tracking-[-0.01em] ${
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
            className="shrink-0 h-10 w-10 rounded-xl transition-all duration-300 hover:bg-accent/80 hover:scale-110 active:scale-95"
            aria-label={ticket.completed ? "标记为未完成" : "标记为已完成"}
          >
            {ticket.completed ? (
              <CheckCircle2 className="h-5 w-5 text-primary transition-all duration-300" />
            ) : (
              <Circle className="h-5 w-5 text-muted-foreground transition-all duration-300 group-hover:text-primary" />
            )}
          </Button>
        </div>
      </CardHeader>
      {ticket.description && (
        <CardContent className="pt-0 pb-5">
          <p className="text-[15px] text-muted-foreground leading-[1.6] whitespace-pre-wrap line-clamp-3">
            {ticket.description}
          </p>
        </CardContent>
      )}
      <CardFooter className="flex flex-col gap-5 pt-0 pb-6">
        {ticket.tags.length > 0 && (
          <div className="flex flex-wrap gap-2.5 w-full">
            {ticket.tags.map((tag) => (
              <TagBadge
                key={tag.id}
                tag={tag}
                onClick={() => onTagClick?.(tag.id)}
              />
            ))}
          </div>
        )}
        <div className="flex gap-2.5 w-full justify-end pt-3 border-t border-border/30">
          <Button
            variant="ghost"
            size="sm"
            onClick={(e) => {
              e.stopPropagation();
              onEdit?.(ticket.id);
            }}
            className="h-9 px-4 text-[14px] font-medium rounded-lg transition-all duration-300 hover:bg-accent/80 hover:scale-105 active:scale-95"
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
            className="h-9 px-4 text-[14px] font-medium text-destructive hover:text-destructive hover:bg-destructive/10 rounded-lg transition-all duration-300 hover:scale-105 active:scale-95"
          >
            <Trash2 className="h-4 w-4 mr-2" />
            删除
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}
