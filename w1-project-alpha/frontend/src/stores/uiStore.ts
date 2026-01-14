import { create } from "zustand";
import { devtools, persist } from "zustand/middleware";

interface UIState {
  // 搜索和筛选状态
  searchQuery: string;
  selectedTagId: string | null;
  showCompleted: boolean | null; // null = 全部, true = 已完成, false = 未完成

  // UI 状态
  sidebarOpen: boolean;
  ticketFormOpen: boolean;
  tagManagerOpen: boolean;
  editingTicketId: string | null; // 正在编辑的 Ticket ID

  // Actions - 搜索和筛选
  setSearchQuery: (query: string) => void;
  setSelectedTagId: (tagId: string | null) => void;
  setShowCompleted: (completed: boolean | null) => void;
  clearFilters: () => void;

  // Actions - UI 状态
  setSidebarOpen: (open: boolean) => void;
  setTicketFormOpen: (open: boolean) => void;
  setTagManagerOpen: (open: boolean) => void;
  setEditingTicketId: (id: string | null) => void;
}

/**
 * UI Store
 * 管理 UI 相关的状态（搜索、筛选、对话框等）
 * 使用 persist 中间件持久化部分状态到 localStorage
 */
export const useUIStore = create<UIState>()(
  devtools(
    persist(
      (set) => ({
        // 初始状态
        searchQuery: "",
        selectedTagId: null,
        showCompleted: null,
        sidebarOpen: true,
        ticketFormOpen: false,
        tagManagerOpen: false,
        editingTicketId: null,

        // 设置搜索关键词
        setSearchQuery: (query: string) => set({ searchQuery: query }),

        // 设置选中的标签
        setSelectedTagId: (tagId: string | null) => set({ selectedTagId: tagId }),

        // 设置完成状态筛选
        setShowCompleted: (completed: boolean | null) => set({ showCompleted: completed }),

        // 清除所有筛选条件
        clearFilters: () =>
          set({
            searchQuery: "",
            selectedTagId: null,
            showCompleted: null,
          }),

        // 设置侧边栏打开/关闭
        setSidebarOpen: (open: boolean) => set({ sidebarOpen: open }),

        // 设置 Ticket 表单对话框打开/关闭
        setTicketFormOpen: (open: boolean) => set({ ticketFormOpen: open }),

        // 设置标签管理器对话框打开/关闭
        setTagManagerOpen: (open: boolean) => set({ tagManagerOpen: open }),

        // 设置正在编辑的 Ticket ID
        setEditingTicketId: (id: string | null) => set({ editingTicketId: id }),
      }),
      {
        name: "ui-store", // localStorage 中的 key
        // 只持久化搜索和筛选状态，不持久化 UI 状态（如对话框打开状态）
        partialize: (state) => ({
          searchQuery: state.searchQuery,
          selectedTagId: state.selectedTagId,
          showCompleted: state.showCompleted,
          sidebarOpen: state.sidebarOpen,
        }),
      }
    ),
    {
      name: "ui-store", // Redux DevTools 中的名称
    }
  )
);
