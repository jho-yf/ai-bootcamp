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
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader>
        <div className="flex items-start justify-between gap-2">
          <div className="flex-1">
            <h3
              className={`text-lg font-semibold ${
                ticket.completed ? "line-through text-muted-foreground" : ""
              }`}
            >
              {ticket.title}
            </h3>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => onToggleCompleted?.(ticket.id)}
            className="shrink-0"
            aria-label={ticket.completed ? "标记为未完成" : "标记为已完成"}
          >
            {ticket.completed ? (
              <CheckCircle2 className="h-5 w-5 text-green-600" />
            ) : (
              <Circle className="h-5 w-5" />
            )}
          </Button>
        </div>
      </CardHeader>
      {ticket.description && (
        <CardContent>
          <p className="text-sm text-muted-foreground whitespace-pre-wrap">
            {ticket.description}
          </p>
        </CardContent>
      )}
      <CardFooter className="flex flex-col gap-3">
        {ticket.tags.length > 0 && (
          <div className="flex flex-wrap gap-2 w-full">
            {ticket.tags.map((tag) => (
              <TagBadge
                key={tag.id}
                tag={tag}
                onClick={() => onTagClick?.(tag.id)}
              />
            ))}
          </div>
        )}
        <div className="flex gap-2 w-full justify-end">
          <Button
            variant="outline"
            size="sm"
            onClick={() => onEdit?.(ticket.id)}
          >
            <Edit className="h-4 w-4 mr-2" />
            编辑
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={() => onDelete?.(ticket.id)}
          >
            <Trash2 className="h-4 w-4 mr-2" />
            删除
          </Button>
        </div>
      </CardFooter>
    </Card>
  );
}
