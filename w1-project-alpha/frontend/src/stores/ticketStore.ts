import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { ticketApi } from "../services/api";
import type {
  TicketWithTags,
  CreateTicketRequest,
  UpdateTicketRequest,
  TicketQuery,
} from "../types";

// Helper function to extract error message
function getErrorMessage(error: unknown, defaultMessage: string): string {
  if (error && typeof error === "object") {
    const err = error as { response?: { data?: { error?: string } }; message?: string };
    return err.response?.data?.error || err.message || defaultMessage;
  }
  return defaultMessage;
}

interface TicketState {
  // 状态
  tickets: TicketWithTags[];
  loading: boolean;
  error: string | null;

  // Actions - 查询
  fetchTickets: (query?: TicketQuery) => Promise<void>;
  fetchTicket: (id: string) => Promise<TicketWithTags | null>;

  // Actions - CRUD
  createTicket: (data: CreateTicketRequest) => Promise<TicketWithTags | null>;
  updateTicket: (id: string, data: UpdateTicketRequest) => Promise<TicketWithTags | null>;
  deleteTicket: (id: string) => Promise<boolean>;
  toggleCompleted: (id: string) => Promise<boolean>;

  // Actions - 标签管理
  addTagToTicket: (ticketId: string, tagId: string) => Promise<boolean>;
  removeTagFromTicket: (ticketId: string, tagId: string) => Promise<boolean>;

  // Actions - 状态管理
  setError: (error: string | null) => void;
  clearError: () => void;
}

/**
 * Ticket Store
 * 管理 Ticket 相关的状态和操作
 */
export const useTicketStore = create<TicketState>()(
  devtools(
    (set) => ({
      // 初始状态
      tickets: [],
      loading: false,
      error: null,

      // 获取 Ticket 列表
      fetchTickets: async (query?: TicketQuery) => {
        set({ loading: true, error: null });
        try {
          const tickets = await ticketApi.getTickets(query);
          // 确保所有 ticket 都有 tags 字段
          const ticketsWithTags = tickets.map((ticket) => ({
            ...ticket,
            tags: ticket.tags || [],
          }));
          set({ tickets: ticketsWithTags, loading: false });
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "获取 Ticket 列表失败");
          set({ error: errorMessage, loading: false });
        }
      },

      // 获取单个 Ticket
      fetchTicket: async (id: string) => {
        set({ loading: true, error: null });
        try {
          const ticket = await ticketApi.getTicket(id);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...ticket,
            tags: ticket.tags || [],
          };
          set({ loading: false });
          return ticketWithTags;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "获取 Ticket 失败");
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 创建 Ticket
      createTicket: async (data: CreateTicketRequest) => {
        set({ loading: true, error: null });
        try {
          const newTicket = await ticketApi.createTicket(data);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...newTicket,
            tags: newTicket.tags || [],
          };
          set((state) => ({
            tickets: [ticketWithTags, ...state.tickets],
            loading: false,
          }));
          return ticketWithTags;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "创建 Ticket 失败");
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 更新 Ticket
      updateTicket: async (id: string, data: UpdateTicketRequest) => {
        set({ loading: true, error: null });
        try {
          const updatedTicket = await ticketApi.updateTicket(id, data);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...updatedTicket,
            tags: updatedTicket.tags || [],
          };
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === id ? ticketWithTags : t)),
            loading: false,
          }));
          return ticketWithTags;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "更新 Ticket 失败");
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 删除 Ticket
      deleteTicket: async (id: string) => {
        set({ loading: true, error: null });
        try {
          await ticketApi.deleteTicket(id);
          set((state) => ({
            tickets: state.tickets.filter((t) => t.id !== id),
            loading: false,
          }));
          return true;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "删除 Ticket 失败");
          set({ error: errorMessage, loading: false });
          return false;
        }
      },

      // 切换完成状态
      toggleCompleted: async (id: string) => {
        set({ loading: true, error: null });
        try {
          const updatedTicket = await ticketApi.toggleCompleted(id);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...updatedTicket,
            tags: updatedTicket.tags || [],
          };
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === id ? ticketWithTags : t)),
            loading: false,
          }));
          return true;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "切换完成状态失败");
          set({ error: errorMessage, loading: false });
          return false;
        }
      },

      // 添加标签到 Ticket
      addTagToTicket: async (ticketId: string, tagId: string) => {
        set({ loading: true, error: null });
        try {
          await ticketApi.addTag(ticketId, tagId);
          // 重新获取 Ticket 以更新标签列表
          const updatedTicket = await ticketApi.getTicket(ticketId);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...updatedTicket,
            tags: updatedTicket.tags || [],
          };
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === ticketId ? ticketWithTags : t)),
            loading: false,
          }));
          return true;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "添加标签失败");
          set({ error: errorMessage, loading: false });
          return false;
        }
      },

      // 从 Ticket 移除标签
      removeTagFromTicket: async (ticketId: string, tagId: string) => {
        set({ loading: true, error: null });
        try {
          await ticketApi.removeTag(ticketId, tagId);
          // 重新获取 Ticket 以更新标签列表
          const updatedTicket = await ticketApi.getTicket(ticketId);
          // 确保 tags 字段存在
          const ticketWithTags: TicketWithTags = {
            ...updatedTicket,
            tags: updatedTicket.tags || [],
          };
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === ticketId ? ticketWithTags : t)),
            loading: false,
          }));
          return true;
        } catch (error: unknown) {
          const errorMessage = getErrorMessage(error, "移除标签失败");
          set({ error: errorMessage, loading: false });
          return false;
        }
      },

      // 设置错误
      setError: (error: string | null) => set({ error }),

      // 清除错误
      clearError: () => set({ error: null }),
    }),
    {
      name: "ticket-store", // Redux DevTools 中的名称
    }
  )
);
