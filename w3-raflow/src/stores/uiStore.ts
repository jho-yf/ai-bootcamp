// src/stores/uiStore.ts

import { create } from "zustand";
import type { Notification } from "../api/types";

interface UIState {
  showSettings: boolean;
  showIndicator: boolean;
  notifications: Notification[];
  openSettings: () => void;
  closeSettings: () => void;
  showNotification: (notification: Omit<Notification, "id">) => void;
  removeNotification: (id: string) => void;
  clearNotifications: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  showSettings: false,
  showIndicator: false,
  notifications: [],

  openSettings: () => set({ showSettings: true }),
  closeSettings: () => set({ showSettings: false }),

  showNotification: (notification) => {
    const id = Math.random().toString(36).substring(7);
    const newNotification: Notification = {
      ...notification,
      id,
    };

    set((state) => ({
      notifications: [...state.notifications, newNotification],
    }));

    // 自动移除通知
    if (notification.duration !== 0) {
      const duration = notification.duration || 5000;
      setTimeout(() => {
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        }));
      }, duration);
    }
  },

  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),

  clearNotifications: () => set({ notifications: [] }),
}));
