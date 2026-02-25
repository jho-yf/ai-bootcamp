// src/components/Notification.tsx

import { useEffect } from "react";
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from "lucide-react";
import { useUIStore } from "../stores/uiStore";
import type { Notification as NotificationType } from "../api/types";

const icons = {
  success: CheckCircle,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info,
};

const styles = {
  success: "bg-green-500",
  error: "bg-red-500",
  warning: "bg-yellow-500",
  info: "bg-blue-500",
};

export function Notification() {
  const { notifications, removeNotification } = useUIStore();

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col space-y-2">
      {notifications.map((notification) => {
        const Icon = icons[notification.type];
        const bgColor = styles[notification.type];

        return (
          <div
            key={notification.id}
            className={`flex items-start space-x-3 p-4 rounded-lg shadow-lg bg-white border-l-4 ${bgColor} min-w-[300px] max-w-md animate-in slide-in-from-right-4 duration-300`}
          >
            <Icon className={`w-5 h-5 ${bgColor} flex-shrink-0 mt-0.5`} />
            <div className="flex-1 min-w-0">
              <p className="text-sm font-medium text-gray-900">
                {notification.title}
              </p>
              {notification.message && (
                <p className="text-sm text-gray-500 mt-1">
                  {notification.message}
                </p>
              )}
            </div>
            <button
              onClick={() => removeNotification(notification.id)}
              className="flex-shrink-0 text-gray-400 hover:text-gray-600"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
}
