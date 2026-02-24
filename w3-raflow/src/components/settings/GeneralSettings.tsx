// src/components/settings/GeneralSettings.tsx

import { useConfigStore } from "../../stores/configStore";
import { useUIStore } from "../../stores/uiStore";

export function GeneralSettings() {
  const { config, saveConfig, isLoading } = useConfigStore();
  const { showNotification } = useUIStore();

  const handleSave = async () => {
    if (!config) return;
    await saveConfig(config);
    showNotification({
      type: "success",
      title: "保存成功",
      message: "配置已保存",
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

    // Update the store (we'll need to add this method)
    useConfigStore.setState({ config: newConfig });
  };

  if (!config) {
    return <div>加载中...</div>;
  }

  return (
    <div className="space-y-4">
      {/* ElevenLabs API Key */}
      <div>
        <label className="block text-sm font-medium mb-1">
          ElevenLabs API Key
        </label>
        <input
          type="password"
          value={config.elevenlabs.api_key || ""}
          onChange={(e) => updateConfig("elevenlabs.api_key", e.target.value)}
          className="w-full px-3 py-2 border rounded dark:bg-gray-700 dark:border-gray-600"
          placeholder="xi-your-api-key"
        />
        <p className="text-xs text-gray-500 mt-1">
          从{" "}
          <a
            href="https://elevenlabs.io"
            target="_blank"
            rel="noopener noreferrer"
            className="text-blue-500 hover:underline"
          >
            ElevenLabs
          </a>{" "}
          获取 API Key
        </p>
      </div>

      {/* Language Selection */}
      <div>
        <label className="block text-sm font-medium mb-1">识别语言</label>
        <select
          value={config.elevenlabs.language || "auto"}
          onChange={(e) => updateConfig("elevenlabs.language", e.target.value)}
          className="w-full px-3 py-2 border rounded dark:bg-gray-700 dark:border-gray-600"
        >
          <option value="auto">自动检测</option>
          <option value="zh-CN">中文</option>
          <option value="en-US">英语</option>
          <option value="ja-JP">日语</option>
          <option value="ko-KR">韩语</option>
          <option value="es-ES">西班牙语</option>
          <option value="fr-FR">法语</option>
          <option value="de-DE">德语</option>
        </select>
      </div>

      {/* Text Insertion Strategy */}
      <div>
        <label className="block text-sm font-medium mb-1">文本插入方式</label>
        <select
          value={config.text.strategy || "auto"}
          onChange={(e) => updateConfig("text.strategy", e.target.value)}
          className="w-full px-3 py-2 border rounded dark:bg-gray-700 dark:border-gray-600"
        >
          <option value="auto">自动选择</option>
          <option value="keyboard">仅键盘输入</option>
          <option value="clipboard">仅剪贴板</option>
        </select>
        <p className="text-xs text-gray-500 mt-1">
          自动模式：优先使用键盘输入，失败时切换到剪贴板
        </p>
      </div>

      {/* Notifications */}
      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.ui.show_notifications || false}
            onChange={(e) => updateConfig("ui.show_notifications", e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm">显示通知</span>
        </label>
      </div>

      {/* Live Preview */}
      <div>
        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.ui.show_live_preview || false}
            onChange={(e) => updateConfig("ui.show_live_preview", e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm">显示实时转录预览</span>
        </label>
      </div>

      {/* Save Button */}
      <button
        onClick={handleSave}
        disabled={isLoading}
        className="w-full py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
      >
        {isLoading ? "保存中..." : "保存设置"}
      </button>
    </div>
  );
}
