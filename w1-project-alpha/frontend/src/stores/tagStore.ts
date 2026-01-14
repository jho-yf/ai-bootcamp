import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { tagApi } from "../services/api";
import type { Tag, CreateTagRequest, UpdateTagRequest } from "../types";

interface TagState {
  // 状态
  tags: Tag[];
  loading: boolean;
  error: string | null;

  // Actions - 查询
  fetchTags: () => Promise<void>;
  fetchTag: (id: string) => Promise<Tag | null>;

  // Actions - CRUD
  createTag: (data: CreateTagRequest) => Promise<Tag | null>;
  updateTag: (id: string, data: UpdateTagRequest) => Promise<Tag | null>;
  deleteTag: (id: string) => Promise<boolean>;

  // Actions - 状态管理
  setError: (error: string | null) => void;
  clearError: () => void;
}

/**
 * Tag Store
 * 管理 Tag 相关的状态和操作
 */
export const useTagStore = create<TagState>()(
  devtools(
    (set) => ({
      // 初始状态
      tags: [],
      loading: false,
      error: null,

      // 获取所有标签
      fetchTags: async () => {
        set({ loading: true, error: null });
        try {
          const tags = await tagApi.getTags();
          set({ tags, loading: false });
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "获取标签列表失败";
          set({ error: errorMessage, loading: false });
        }
      },

      // 获取单个标签
      fetchTag: async (id: string) => {
        set({ loading: true, error: null });
        try {
          const tag = await tagApi.getTag(id);
          set({ loading: false });
          return tag;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "获取标签失败";
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 创建标签
      createTag: async (data: CreateTagRequest) => {
        set({ loading: true, error: null });
        try {
          const newTag = await tagApi.createTag(data);
          set((state) => ({
            tags: [...state.tags, newTag].sort((a, b) => a.name.localeCompare(b.name)),
            loading: false,
          }));
          return newTag;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "创建标签失败";
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 更新标签
      updateTag: async (id: string, data: UpdateTagRequest) => {
        set({ loading: true, error: null });
        try {
          const updatedTag = await tagApi.updateTag(id, data);
          set((state) => ({
            tags: state.tags
              .map((t) => (t.id === id ? updatedTag : t))
              .sort((a, b) => a.name.localeCompare(b.name)),
            loading: false,
          }));
          return updatedTag;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "更新标签失败";
          set({ error: errorMessage, loading: false });
          return null;
        }
      },

      // 删除标签
      deleteTag: async (id: string) => {
        set({ loading: true, error: null });
        try {
          await tagApi.deleteTag(id);
          set((state) => ({
            tags: state.tags.filter((t) => t.id !== id),
            loading: false,
          }));
          return true;
        } catch (error: any) {
          const errorMessage = error.response?.data?.error || error.message || "删除标签失败";
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
      name: "tag-store", // Redux DevTools 中的名称
    }
  )
);
