import { useState } from "react";
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
import { TagBadge } from "./TagBadge";
import { Edit, Trash2, Plus } from "lucide-react";
import type { Tag, CreateTagRequest, UpdateTagRequest } from "@/types";

const tagSchema = z.object({
  name: z.string().min(1, "标签名称不能为空").max(50, "标签名称不能超过 50 个字符"),
  color: z.string().regex(/^#[0-9A-Fa-f]{6}$/, "颜色格式不正确（应为 #RRGGBB）").optional().nullable(),
});

type TagFormData = z.infer<typeof tagSchema>;

interface TagManagerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  tags: Tag[];
  onCreateTag: (data: CreateTagRequest) => Promise<void>;
  onUpdateTag: (id: string, data: UpdateTagRequest) => Promise<void>;
  onDeleteTag: (id: string) => Promise<void>;
}

/**
 * TagManager 组件
 * 标签管理器，支持创建、编辑和删除标签
 */
export function TagManager({
  open,
  onOpenChange,
  tags,
  onCreateTag,
  onUpdateTag,
  onDeleteTag,
}: TagManagerProps) {
  const [editingTag, setEditingTag] = useState<Tag | null>(null);
  const [showForm, setShowForm] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors, isSubmitting },
    reset,
    setValue,
    watch,
  } = useForm<TagFormData>({
    resolver: zodResolver(tagSchema),
    defaultValues: {
      name: "",
      color: null,
    },
  });

  const handleCreateClick = () => {
    setEditingTag(null);
    reset({ name: "", color: null });
    setShowForm(true);
  };

  const handleEditClick = (tag: Tag) => {
    setEditingTag(tag);
    reset({
      name: tag.name,
      color: tag.color || null,
    });
    setShowForm(true);
  };

  const handleFormSubmit = async (data: TagFormData) => {
    if (editingTag) {
      await onUpdateTag(editingTag.id, {
        name: data.name,
        color: data.color || null,
      });
    } else {
      await onCreateTag({
        name: data.name,
        color: data.color || null,
      });
    }
    reset();
    setShowForm(false);
    setEditingTag(null);
  };

  const handleCancel = () => {
    reset();
    setShowForm(false);
    setEditingTag(null);
  };

  const handleDelete = async (id: string) => {
    if (confirm("确定要删除这个标签吗？")) {
      await onDeleteTag(id);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>标签管理</DialogTitle>
          <DialogDescription>创建、编辑和删除标签</DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {!showForm && (
            <Button onClick={handleCreateClick} className="w-full">
              <Plus className="h-4 w-4 mr-2" />
              创建新标签
            </Button>
          )}

          {showForm && (
            <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-4 p-4 border rounded-lg">
              <div className="space-y-2">
                <Label htmlFor="tag-name">
                  标签名称 <span className="text-destructive">*</span>
                </Label>
                <Input
                  id="tag-name"
                  {...register("name")}
                  placeholder="输入标签名称"
                  className={errors.name ? "border-destructive" : ""}
                />
                {errors.name && (
                  <p className="text-sm text-destructive">{errors.name.message}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="tag-color">颜色（可选）</Label>
                <div className="flex gap-2">
                  <Input
                    id="tag-color"
                    type="color"
                    {...register("color")}
                    className="w-20 h-10"
                    value={watch("color") || "#000000"}
                    onChange={(e) => setValue("color", e.target.value)}
                  />
                  <Input
                    type="text"
                    placeholder="#RRGGBB"
                    {...register("color")}
                    className={errors.color ? "border-destructive" : ""}
                  />
                </div>
                {errors.color && (
                  <p className="text-sm text-destructive">{errors.color.message}</p>
                )}
              </div>

              <div className="flex gap-2">
                <Button type="submit" disabled={isSubmitting}>
                  {isSubmitting ? "保存中..." : editingTag ? "更新" : "创建"}
                </Button>
                <Button type="button" variant="outline" onClick={handleCancel}>
                  取消
                </Button>
              </div>
            </form>
          )}

          <div className="space-y-2">
            <Label>标签列表</Label>
            <div className="space-y-2 max-h-60 overflow-y-auto">
              {tags.length === 0 ? (
                <p className="text-sm text-muted-foreground text-center py-4">
                  暂无标签
                </p>
              ) : (
                tags.map((tag) => (
                  <div
                    key={tag.id}
                    className="flex items-center justify-between p-2 border rounded-lg hover:bg-accent"
                  >
                    <TagBadge tag={tag} />
                    <div className="flex gap-2">
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleEditClick(tag)}
                        aria-label={`编辑标签 ${tag.name}`}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleDelete(tag.id)}
                        aria-label={`删除标签 ${tag.name}`}
                      >
                        <Trash2 className="h-4 w-4 text-destructive" />
                      </Button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            关闭
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
