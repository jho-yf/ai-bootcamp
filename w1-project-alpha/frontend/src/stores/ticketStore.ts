import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { ticketApi } from "../services/api";
import type { TicketWithTags, CreateTicketRequest, UpdateTicketRequest, TicketQuery } from "../types";

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
          set({ tickets, loading: false });
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "获取 Ticket 列表失败";
          set({ error: errorMessage, loading: false });
        }
      },

      // 获取单个 Ticket
      fetchTicket: async (id: string) => {
        set({ loading: true, error: null });
        try {
          const ticket = await ticketApi.getTicket(id);
          set({ loading: false });
          return ticket;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "获取 Ticket 失败";
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 创建 Ticket
      createTicket: async (data: CreateTicketRequest) => {
        set({ loading: true, error: null });
        try {
          const newTicket = await ticketApi.createTicket(data);
          set((state) => ({
            tickets: [newTicket, ...state.tickets],
            loading: false,
          }));
          return newTicket;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "创建 Ticket 失败";
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 更新 Ticket
      updateTicket: async (id: string, data: UpdateTicketRequest) => {
        set({ loading: true, error: null });
        try {
          const updatedTicket = await ticketApi.updateTicket(id, data);
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === id ? updatedTicket : t)),
            loading: false,
          }));
          return updatedTicket;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "更新 Ticket 失败";
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
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "删除 Ticket 失败";
          set({ error: errorMessage, loading: false });
          return false;
        }
      },

      // 切换完成状态
      toggleCompleted: async (id: string) => {
        set({ loading: true, error: null });
        try {
          const updatedTicket = await ticketApi.toggleCompleted(id);
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === id ? updatedTicket : t)),
            loading: false,
          }));
          return true;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "切换完成状态失败";
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
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === ticketId ? updatedTicket : t)),
            loading: false,
          }));
          return true;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "添加标签失败";
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
          set((state) => ({
            tickets: state.tickets.map((t) => (t.id === ticketId ? updatedTicket : t)),
            loading: false,
          }));
          return true;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "移除标签失败";
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
