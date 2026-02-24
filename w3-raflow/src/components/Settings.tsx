// src/components/Settings.tsx

import { useState } from "react";
import { X } from "lucide-react";
import { useUIStore } from "../stores/uiStore";
import { GeneralSettings } from "./settings/GeneralSettings";
import { AudioSettings } from "./settings/AudioSettings";
import { HotkeySettings } from "./settings/HotkeySettings";
import { About } from "./settings/About";

type TabId = "general" | "audio" | "hotkey" | "about";

interface Tab {
  id: TabId;
  label: string;
}

const tabs: Tab[] = [
  { id: "general", label: "常规" },
  { id: "audio", label: "音频" },
  { id: "hotkey", label: "快捷键" },
  { id: "about", label: "关于" },
];

export function Settings() {
  const { showSettings, closeSettings } = useUIStore();
  const [activeTab, setActiveTab] = useState<TabId>("general");

  if (!showSettings) return null;

  return (
    <div className="fixed inset-0 flex items-center justify-center bg-black/50 z-50">
      <div className="w-[600px] h-[500px] bg-white dark:bg-gray-800 rounded-lg shadow-xl flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b dark:border-gray-700">
          <h2 className="text-xl font-semibold">设置</h2>
          <button
            onClick={closeSettings}
            className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded"
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Tab Navigation */}
        <div className="flex border-b dark:border-gray-700">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`px-4 py-2 text-sm font-medium transition-colors ${
                activeTab === tab.id
                  ? "border-b-2 border-blue-500 text-blue-500"
                  : "text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200"
              }`}
            >
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content Area */}
        <div className="flex-1 overflow-y-auto p-4">
          {activeTab === "general" && <GeneralSettings />}
          {activeTab === "audio" && <AudioSettings />}
          {activeTab === "hotkey" && <HotkeySettings />}
          {activeTab === "about" && <About />}
        </div>
      </div>
    </div>
  );
}
