// src/components/settings/HotkeySettings.tsx

import { useState, useEffect } from "react";
import { RotateCcw } from "lucide-react";
import { useConfigStore } from "../../stores/configStore";
import { useUIStore } from "../../stores/uiStore";

type KeyModifier = "ctrl" | "alt" | "shift" | "super";

export function HotkeySettings() {
  const { config } = useConfigStore();
  const { showNotification } = useUIStore();
  const [capturing, setCapturing] = useState(false);
  const [currentHotkey, setCurrentHotkey] = useState<string>("");

  const startCapture = () => {
    setCapturing(true);
    setCurrentHotkey("按下快捷键...");
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!capturing) return;
    e.preventDefault();
    e.stopPropagation();

    const modifiers: KeyModifier[] = [];
    if (e.ctrlKey) modifiers.push("ctrl");
    if (e.altKey) modifiers.push("alt");
    if (e.shiftKey) modifiers.push("shift");
    if (e.metaKey) modifiers.push("super");

    // Get the key (ignore modifiers alone)
    const key = e.key.toLowerCase();
    if (["control", "alt", "shift", "meta"].includes(key)) {
      return;
    }

    const hotkeyStr = [...modifiers, key].join("+");
    setCurrentHotkey(hotkeyStr);
    setCapturing(false);

    // Update config
    updateConfig("hotkey.modifiers", modifiers);
    updateConfig("hotkey.key", key);

    showNotification({
      type: "success",
      title: "快捷键已更新",
      message: `新快捷键: ${hotkeyStr}`,
    });
  };

  useEffect(() => {
    if (capturing) {
      window.addEventListener("keydown", handleKeyDown, { capture: true });
      return () => {
        window.removeEventListener("keydown", handleKeyDown, { capture: true });
      };
    }
  }, [capturing]);

  const resetHotkey = () => {
    updateConfig("hotkey.modifiers", ["ctrl", "shift"]);
    updateConfig("hotkey.key", "o");
    showNotification({
      type: "info",
      title: "快捷键已重置",
      message: "已恢复为默认快捷键: Ctrl+Shift+O",
    });
  };

  const updateConfig = (path: string, value: any) => {
    if (!config) return;

    const keys = path.split(".");
    const newConfig = { ...config };

    let current: any = newConfig;
    for (let i = 0; i < keys.length - 1; i++) {
      current[keys[i]] = { ...current[keys[i]] };
      current = current[keys[i]];
    }
    current[keys[keys.length - 1]] = value;

    useConfigStore.setState({ config: newConfig });
  };

  const displayHotkey = () => {
    if (!config) return "";
    const modifiers = config.hotkey.modifiers || [];
    const key = config.hotkey.key || "";
    return [...modifiers, key].join("+").toUpperCase();
  };

  if (!config) {
    return <div className="text-gray-900">加载中...</div>;
  }

  return (
    <div className="space-y-4">
      {/* Hotkey Display and Capture */}
      <div>
        <label className="block text-sm font-medium text-gray-900 mb-1">全局快捷键</label>
        <button
          onClick={startCapture}
          disabled={!config.hotkey.enabled}
          className={`w-full px-4 py-3 border-2 rounded text-center font-mono transition-colors font-medium ${
            capturing
              ? "border-blue-500 bg-blue-50 text-blue-900"
              : "border-gray-300 bg-white text-gray-900 hover:border-gray-400"
          } ${!config.hotkey.enabled ? "opacity-50 cursor-not-allowed" : ""}`}
        >
          {capturing ? currentHotkey : displayHotkey()}
        </button>
        <p className="text-xs text-gray-600 mt-1">
          点击上方按钮，然后按下您想设置的快捷键
        </p>
      </div>

      {/* Reset Button */}
      <button
        onClick={resetHotkey}
        className="px-4 py-2 text-gray-700 border border-gray-300 rounded hover:bg-gray-50 flex items-center font-medium"
      >
        <RotateCcw className="w-4 h-4 mr-2" />
        重置为默认快捷键
      </button>

      {/* Enable Toggle */}
      <label className="flex items-center">
        <input
          type="checkbox"
          checked={config.hotkey.enabled || false}
          onChange={(e) => updateConfig("hotkey.enabled", e.target.checked)}
          className="mr-2"
        />
        <span className="text-sm text-gray-900">启用全局快捷键</span>
      </label>

      {/* Instructions */}
      <div className="pt-4 border-t border-gray-200">
        <h3 className="text-sm font-medium text-gray-900 mb-2">使用说明</h3>
        <ul className="text-xs text-gray-600 space-y-1">
          <li>• 按下快捷键开始/停止录音</li>
          <li>• 默认快捷键: Ctrl+Shift+O</li>
          <li>• 支持 Ctrl, Alt, Shift, Super (Win/Cmd) 修饰键</li>
          <li>• 确保快捷键不与其他应用冲突</li>
        </ul>
      </div>
    </div>
  );
}
