import axios, { type AxiosInstance, type AxiosError } from "axios";
import type {
  TicketWithTags,
  CreateTicketRequest,
  UpdateTicketRequest,
  TicketQuery,
  Tag,
  CreateTagRequest,
  UpdateTagRequest,
  ErrorResponse,
} from "../types";

/**
 * API 基础 URL
 * 从环境变量读取，开发环境默认为 http://localhost:3000
 */
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";

/**
 * 创建 Axios 实例
 */
const apiClient: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    "Content-Type": "application/json",
  },
});

/**
 * 请求拦截器
 */
apiClient.interceptors.request.use(
  (config) => {
    // 可以在这里添加认证 token 等
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

/**
 * 响应拦截器 - 统一错误处理
 */
apiClient.interceptors.response.use(
  (response) => {
    return response;
  },
  (error: AxiosError<ErrorResponse>) => {
    // 统一处理错误响应
    if (error.response) {
      const errorResponse = error.response.data;
      console.error("API Error:", errorResponse);
    } else if (error.request) {
      console.error("Network Error:", error.request);
    } else {
      console.error("Error:", error.message);
    }
    return Promise.reject(error);
  }
);

/**
 * Ticket API
 */
export const ticketApi = {
  /**
   * 获取 Ticket 列表
   * @param query - 查询参数（标签、搜索、完成状态）
   */
  getTickets: async (query?: TicketQuery): Promise<TicketWithTags[]> => {
    const params = new URLSearchParams();
    if (query?.tag) params.append("tag", query.tag);
    if (query?.search) params.append("search", query.search);
    if (query?.completed !== undefined) params.append("completed", String(query.completed));

    const response = await apiClient.get<TicketWithTags[]>(
      `/api/tickets${params.toString() ? `?${params.toString()}` : ""}`
    );
    return response.data;
  },

  /**
   * 获取单个 Ticket
   * @param id - Ticket ID
   */
  getTicket: async (id: string): Promise<TicketWithTags> => {
    const response = await apiClient.get<TicketWithTags>(`/api/tickets/${id}`);
    return response.data;
  },

  /**
   * 创建 Ticket
   * @param data - Ticket 数据
   */
  createTicket: async (data: CreateTicketRequest): Promise<TicketWithTags> => {
    const response = await apiClient.post<TicketWithTags>("/api/tickets", data);
    return response.data;
  },

  /**
   * 更新 Ticket
   * @param id - Ticket ID
   * @param data - 更新的数据
   */
  updateTicket: async (id: string, data: UpdateTicketRequest): Promise<TicketWithTags> => {
    const response = await apiClient.put<TicketWithTags>(`/api/tickets/${id}`, data);
    return response.data;
  },

  /**
   * 删除 Ticket
   * @param id - Ticket ID
   */
  deleteTicket: async (id: string): Promise<void> => {
    await apiClient.delete(`/api/tickets/${id}`);
  },

  /**
   * 切换 Ticket 完成状态
   * @param id - Ticket ID
   */
  toggleCompleted: async (id: string): Promise<TicketWithTags> => {
    const response = await apiClient.patch<TicketWithTags>(`/api/tickets/${id}/toggle`);
    return response.data;
  },

  /**
   * 添加标签到 Ticket
   * @param ticketId - Ticket ID
   * @param tagId - Tag ID
   */
  addTag: async (ticketId: string, tagId: string): Promise<void> => {
    await apiClient.post(`/api/tickets/${ticketId}/tags/${tagId}`);
  },

  /**
   * 从 Ticket 移除标签
   * @param ticketId - Ticket ID
   * @param tagId - Tag ID
   */
  removeTag: async (ticketId: string, tagId: string): Promise<void> => {
    await apiClient.delete(`/api/tickets/${ticketId}/tags/${tagId}`);
  },
};

/**
 * Tag API
 */
export const tagApi = {
  /**
   * 获取所有标签
   */
  getTags: async (): Promise<Tag[]> => {
    const response = await apiClient.get<Tag[]>("/api/tags");
    return response.data;
  },

  /**
   * 获取单个标签
   * @param id - Tag ID
   */
  getTag: async (id: string): Promise<Tag> => {
    const response = await apiClient.get<Tag>(`/api/tags/${id}`);
    return response.data;
  },

  /**
   * 创建标签
   * @param data - Tag 数据
   */
  createTag: async (data: CreateTagRequest): Promise<Tag> => {
    const response = await apiClient.post<Tag>("/api/tags", data);
    return response.data;
  },

  /**
   * 更新标签
   * @param id - Tag ID
   * @param data - 更新的数据
   */
  updateTag: async (id: string, data: UpdateTagRequest): Promise<Tag> => {
    const response = await apiClient.put<Tag>(`/api/tags/${id}`, data);
    return response.data;
  },

  /**
   * 删除标签
   * @param id - Tag ID
   */
  deleteTag: async (id: string): Promise<void> => {
    await apiClient.delete(`/api/tags/${id}`);
  },
};

export default apiClient;
