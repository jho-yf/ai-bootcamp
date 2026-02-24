// src/stores/configStore.ts

import { create } from "zustand";
import type { AppConfig } from "../api/types";
import { configApi } from "../api/tauri";

interface ConfigState {
  config: AppConfig | null;
  isLoading: boolean;
  error: string | null;
  loadConfig: () => Promise<void>;
  saveConfig: (config: AppConfig) => Promise<void>;
  resetConfig: () => Promise<void>;
}

export const useConfigStore = create<ConfigState>((set, get) => ({
  config: null,
  isLoading: false,
  error: null,

  loadConfig: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await configApi.getConfig();
      set({ config, isLoading: false });
    } catch (e) {
      set({
        error: typeof e === "string" ? e : "加载配置失败",
        isLoading: false,
      });
    }
  },

  saveConfig: async (config) => {
    set({ isLoading: true, error: null });
    try {
      await configApi.saveConfig(config);
      set({ config, isLoading: false });
    } catch (e) {
      set({
        error: typeof e === "string" ? e : "保存配置失败",
        isLoading: false,
      });
    }
  },

  resetConfig: async () => {
    set({ isLoading: true, error: null });
    try {
      const config = await configApi.resetConfig();
      set({ config, isLoading: false });
    } catch (e) {
      set({
        error: typeof e === "string" ? e : "重置配置失败",
        isLoading: false,
      });
    }
  },
}));
