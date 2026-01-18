import { useEffect, useMemo } from "react";
import { useTicketStore, useTagStore, useUIStore } from "@/stores";
import { Sidebar } from "@/components/Sidebar";
import { SearchBar } from "@/components/SearchBar";
import { FilterPanel } from "@/components/FilterPanel";
import { TicketCard } from "@/components/TicketCard";
import { TicketForm } from "@/components/TicketForm";
import { TagManager } from "@/components/TagManager";
import { Toaster } from "@/components/ui/toaster";
import { useToast } from "@/hooks/use-toast";
import { Button } from "@/components/ui/button";
import { Plus } from "lucide-react";
import type { CreateTicketRequest, UpdateTicketRequest } from "@/types";

/**
 * TicketsPage 组件
 * 主页面：Ticket 列表页
 * 布局：侧边栏 + 主内容区
 */
export function TicketsPage() {
  const { toast } = useToast();

  // Stores
  const {
    tickets,
    loading: ticketsLoading,
    error: ticketsError,
    fetchTickets,
    createTicket,
    updateTicket,
    deleteTicket,
    toggleCompleted,
  } = useTicketStore();

  const { tags, loading: tagsLoading, fetchTags, createTag, updateTag, deleteTag } = useTagStore();

  const {
    searchQuery,
    selectedTagId,
    showCompleted,
    ticketFormOpen,
    tagManagerOpen,
    editingTicketId,
    setSearchQuery,
    setSelectedTagId,
    setShowCompleted,
    setTicketFormOpen,
    setTagManagerOpen,
    setEditingTicketId,
  } = useUIStore();

  // 获取编辑中的 Ticket
  const editingTicket = useMemo(() => {
    if (!editingTicketId) return null;
    return tickets.find((t) => t.id === editingTicketId) || null;
  }, [editingTicketId, tickets]);

  // 初始化加载数据
  useEffect(() => {
    fetchTags();
  }, [fetchTags]);

  // 根据筛选条件获取 Tickets
  useEffect(() => {
    const query = {
      search: searchQuery || undefined,
      tag: selectedTagId || undefined,
      completed: showCompleted !== null ? showCompleted : undefined,
    };
    fetchTickets(query);
  }, [searchQuery, selectedTagId, showCompleted, fetchTickets]);

  // 错误处理
  useEffect(() => {
    if (ticketsError) {
      toast({
        variant: "destructive",
        title: "错误",
        description: ticketsError,
      });
    }
  }, [ticketsError, toast]);

  // 处理表单提交
  const handleFormSubmit = async (data: CreateTicketRequest | UpdateTicketRequest) => {
    if (editingTicket) {
      // 编辑模式
      const result = await updateTicket(editingTicketId!, data as UpdateTicketRequest);
      if (result) {
        toast({
          title: "成功",
          description: "Ticket 更新成功",
        });
        setTicketFormOpen(false);
        setEditingTicketId(null);
      }
    } else {
      // 创建模式
      const result = await createTicket(data as CreateTicketRequest);
      if (result) {
        toast({
          title: "成功",
          description: "Ticket 创建成功",
        });
        setTicketFormOpen(false);
      }
    }
  };

  // 处理删除 Ticket
  const handleDeleteTicket = async (id: string) => {
    if (confirm("确定要删除这个 Ticket 吗？")) {
      const success = await deleteTicket(id);
      if (success) {
        toast({
          title: "成功",
          description: "Ticket 删除成功",
        });
      }
    }
  };

  // 处理切换完成状态
  const handleToggleCompleted = async (id: string) => {
    const success = await toggleCompleted(id);
    if (success) {
      toast({
        title: "成功",
        description: "状态已更新",
      });
    }
  };

  // 处理编辑 Ticket
  const handleEditTicket = (id: string) => {
    setEditingTicketId(id);
    setTicketFormOpen(true);
  };

  // 处理创建标签
  const handleCreateTag = async (data: { name: string; color?: string | null }) => {
    const result = await createTag(data);
    if (result) {
      toast({
        title: "成功",
        description: "标签创建成功",
      });
    }
  };

  // 处理更新标签
  const handleUpdateTag = async (
    id: string,
    data: { name?: string | null; color?: string | null }
  ) => {
    const result = await updateTag(id, data);
    if (result) {
      toast({
        title: "成功",
        description: "标签更新成功",
      });
    }
  };

  // 处理删除标签
  const handleDeleteTag = async (id: string) => {
    const success = await deleteTag(id);
    if (success) {
      toast({
        title: "成功",
        description: "标签删除成功",
      });
    }
  };

  // 处理标签点击（筛选）
  const handleTagClick = (tagId: string | null) => {
    setSelectedTagId(tagId);
  };

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      {/* 侧边栏 */}
      <div className="w-80 shrink-0 border-r border-border bg-card">
        <Sidebar
          tags={tags}
          selectedTagId={selectedTagId}
          onTagClick={handleTagClick}
          onCreateTagClick={() => setTagManagerOpen(true)}
          onManageTagsClick={() => setTagManagerOpen(true)}
        />
      </div>

      {/* 主内容区 */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* 顶部工具栏 */}
        <div className="border-b border-border bg-card sticky top-0 z-10">
          <div className="max-w-container mx-auto px-[5vw] py-6">
            <div className="flex items-center justify-between gap-12">
              <div className="flex-1 max-w-2xl">
                <SearchBar value={searchQuery} onChange={setSearchQuery} />
              </div>
              <Button onClick={() => setTicketFormOpen(true)} size="default">
                <Plus className="h-4 w-4 mr-2" />
                创建 Ticket
              </Button>
            </div>
          </div>
        </div>

        {/* 筛选面板 */}
        <div className="border-b border-border bg-background">
          <div className="max-w-container mx-auto px-[5vw] py-6">
            <FilterPanel
              tags={tags}
              selectedTagId={selectedTagId}
              showCompleted={showCompleted}
              onTagChange={setSelectedTagId}
              onCompletedChange={setShowCompleted}
            />
          </div>
        </div>

        {/* Ticket 列表 */}
        <div className="flex-1 overflow-y-auto">
          <div className="max-w-container mx-auto px-[5vw] py-24">
            {ticketsLoading || tagsLoading ? (
              <div className="flex items-center justify-center h-[60vh]">
                <div className="text-center">
                  <div className="inline-block h-10 w-10 animate-spin rounded-full border-2 border-solid border-border border-t-foreground mb-6"></div>
                  <p className="text-muted-foreground text-base font-normal">加载中...</p>
                </div>
              </div>
            ) : tickets.length === 0 ? (
              <div className="flex flex-col items-center justify-center h-[60vh]">
                <p className="text-muted-foreground mb-12 text-lg font-normal">暂无 Ticket</p>
                <Button onClick={() => setTicketFormOpen(true)} size="default">
                  <Plus className="h-4 w-4 mr-2" />
                  创建第一个 Ticket
                </Button>
              </div>
            ) : (
              <div className="grid gap-12 grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
                {tickets.map((ticket) => (
                  <TicketCard
                    key={ticket.id}
                    ticket={ticket}
                    onToggleCompleted={handleToggleCompleted}
                    onEdit={handleEditTicket}
                    onDelete={handleDeleteTicket}
                    onTagClick={handleTagClick}
                  />
                ))}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Ticket 表单对话框 */}
      <TicketForm
        open={ticketFormOpen}
        onOpenChange={(open) => {
          setTicketFormOpen(open);
          if (!open) {
            setEditingTicketId(null);
          }
        }}
        onSubmit={handleFormSubmit}
        tags={tags}
        ticket={editingTicket}
        mode={editingTicket ? "edit" : "create"}
      />

      {/* 标签管理器对话框 */}
      <TagManager
        open={tagManagerOpen}
        onOpenChange={setTagManagerOpen}
        tags={tags}
        onCreateTag={handleCreateTag}
        onUpdateTag={handleUpdateTag}
        onDeleteTag={handleDeleteTag}
      />

      {/* Toast 通知 */}
      <Toaster />
    </div>
  );
}
