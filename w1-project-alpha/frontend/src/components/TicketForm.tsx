import { useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "./ui/dialog";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Label } from "./ui/label";
import { TagSelector } from "./TagSelector";
import { Checkbox } from "./ui/checkbox";
import { Textarea } from "./ui/textarea";
import type { TicketWithTags, CreateTicketRequest, UpdateTicketRequest, Tag } from "@/types";

const ticketSchema = z.object({
  title: z.string().min(1, "标题不能为空").max(255, "标题不能超过 255 个字符"),
  description: z.string().optional().nullable(),
  tag_ids: z.array(z.string()).optional().nullable(),
  completed: z.boolean().optional().nullable(),
});

type TicketFormData = z.infer<typeof ticketSchema>;

interface TicketFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (data: CreateTicketRequest | UpdateTicketRequest) => Promise<void>;
  tags: Tag[];
  ticket?: TicketWithTags | null;
  mode?: "create" | "edit";
}

/**
 * TicketForm 组件
 * Ticket 表单组件，支持创建和编辑模式
 * 使用 React Hook Form + Zod 验证
 */
export function TicketForm({
  open,
  onOpenChange,
  onSubmit,
  tags,
  ticket,
  mode = "create",
}: TicketFormProps) {
  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    reset,
    watch,
    setValue,
  } = useForm<TicketFormData>({
    resolver: zodResolver(ticketSchema),
    defaultValues: {
      title: "",
      description: "",
      tag_ids: [],
      completed: false,
    },
  });

  const selectedTagIds = watch("tag_ids") || [];

  useEffect(() => {
    if (ticket && mode === "edit") {
      reset({
        title: ticket.title,
        description: ticket.description || "",
        tag_ids: ticket.tags.map((tag) => tag.id),
        completed: ticket.completed,
      });
    } else {
      reset({
        title: "",
        description: "",
        tag_ids: [],
        completed: false,
      });
    }
  }, [ticket, mode, reset]);

  const handleFormSubmit = async (data: TicketFormData) => {
    const submitData: CreateTicketRequest | UpdateTicketRequest = {
      title: data.title,
      description: data.description || null,
      tag_ids: data.tag_ids && data.tag_ids.length > 0 ? data.tag_ids : null,
      ...(mode === "edit" && { completed: data.completed ?? null }),
    };
    await onSubmit(submitData);
    if (mode === "create") {
      reset();
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{mode === "create" ? "创建 Ticket" : "编辑 Ticket"}</DialogTitle>
          <DialogDescription>
            {mode === "create"
              ? "填写以下信息创建新的 Ticket"
              : "修改 Ticket 信息"}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="title">
              标题 <span className="text-destructive">*</span>
            </Label>
            <Input
              id="title"
              {...register("title")}
              placeholder="输入 Ticket 标题"
              className={errors.title ? "border-destructive" : ""}
            />
            {errors.title && (
              <p className="text-sm text-destructive">{errors.title.message}</p>
            )}
          </div>

          <div className="space-y-2">
            <Label htmlFor="description">描述</Label>
            <Textarea
              id="description"
              {...register("description")}
              placeholder="输入 Ticket 描述（可选）"
              rows={4}
            />
          </div>

          <div className="space-y-2">
            <Label>标签</Label>
            <TagSelector
              tags={tags}
              selectedTagIds={selectedTagIds}
              onSelectionChange={(tagIds) => setValue("tag_ids", tagIds)}
            />
          </div>

          {mode === "edit" && (
            <div className="flex items-center space-x-2">
              <Checkbox
                id="completed"
                checked={watch("completed") || false}
                onCheckedChange={(checked) => setValue("completed", checked === true)}
              />
              <Label htmlFor="completed" className="cursor-pointer">
                已完成
              </Label>
            </div>
          )}

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
              disabled={isSubmitting}
            >
              取消
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting ? "保存中..." : "保存"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
