// src/components/settings/AudioSettings.tsx

import { useState, useEffect } from "react";
import { RefreshCw, Mic } from "lucide-react";
import { useConfigStore } from "../../stores/configStore";
import { useAudioStore } from "../../stores/audioStore";
import { audioApi } from "../../api/tauri";
import { useUIStore } from "../../stores/uiStore";

export function AudioSettings() {
  const { config } = useConfigStore();
  const { devices, setDevices } = useAudioStore();
  const { showNotification } = useUIStore();
  const [testing, setTesting] = useState(false);
  const [volumeLevel, setVolumeLevel] = useState(0);

  useEffect(() => {
    loadDevices();
  }, []);

  const loadDevices = async () => {
    try {
      const devs = await audioApi.getDevices();
      setDevices(devs);
    } catch (e) {
      showNotification({
        type: "error",
        title: "加载失败",
        message: "无法加载音频设备",
      });
    }
  };

  const testMicrophone = async (deviceId: string) => {
    setTesting(true);
    setVolumeLevel(0);

    // Simulate volume changes
    const interval = setInterval(() => {
      setVolumeLevel(Math.random() * 100);
    }, 100);

    try {
      const success = await audioApi.testMicrophone(deviceId);
      clearInterval(interval);
      setVolumeLevel(0);

      if (success) {
        showNotification({
          type: "success",
          title: "测试成功",
          message: "麦克风工作正常",
        });
      } else {
        showNotification({
          type: "error",
          title: "测试失败",
          message: "麦克风无响应",
        });
      }
    } catch (e) {
      clearInterval(interval);
      setVolumeLevel(0);
      showNotification({
        type: "error",
        title: "测试失败",
        message: typeof e === "string" ? e : "无法测试麦克风",
      });
    } finally {
      setTesting(false);
    }
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

  if (!config) {
    return <div>加载中...</div>;
  }

  return (
    <div className="space-y-4">
      {/* Device Selection */}
      <div>
        <div className="flex items-center justify-between mb-1">
          <label className="text-sm font-medium">音频输入设备</label>
          <button
            onClick={loadDevices}
            className="text-blue-500 text-sm flex items-center hover:text-blue-600"
          >
            <RefreshCw className="w-4 h-4 mr-1" />
            刷新
          </button>
        </div>
        <select
          value={config.audio.device_id || ""}
          onChange={(e) => updateConfig("audio.device_id", e.target.value)}
          className="w-full px-3 py-2 border rounded dark:bg-gray-700 dark:border-gray-600"
        >
          <option value="">默认设备</option>
          {devices.map((dev) => (
            <option key={dev.id} value={dev.id}>
              {dev.name} {dev.is_default && "(默认)"}
            </option>
          ))}
        </select>
      </div>

      {/* Microphone Test */}
      <div>
        <label className="text-sm font-medium mb-2 block">麦克风测试</label>
        <div className="flex items-center space-x-2">
          <button
            onClick={() => testMicrophone(config.audio.device_id || "")}
            disabled={testing}
            className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50 flex items-center"
          >
            <Mic className="w-4 h-4 mr-2" />
            {testing ? "测试中..." : "测试麦克风"}
          </button>
          {testing && (
            <div className="flex-1">
              <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all duration-100"
                  style={{ width: `${volumeLevel}%` }}
                />
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Audio Enhancement Options */}
      <div className="space-y-2 pt-4 border-t dark:border-gray-700">
        <h3 className="text-sm font-medium">音频增强</h3>

        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.audio.echo_cancellation || false}
            onChange={(e) => updateConfig("audio.echo_cancellation", e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm">回声消除</span>
        </label>

        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.audio.noise_suppression || false}
            onChange={(e) => updateConfig("audio.noise_suppression", e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm">噪声抑制</span>
        </label>

        <label className="flex items-center">
          <input
            type="checkbox"
            checked={config.audio.auto_gain || false}
            onChange={(e) => updateConfig("audio.auto_gain", e.target.checked)}
            className="mr-2"
          />
          <span className="text-sm">自动增益控制</span>
        </label>
      </div>

      {/* Sample Rate Info */}
      <div className="pt-4 border-t dark:border-gray-700">
        <p className="text-xs text-gray-500">
          采样率: {config.audio.sample_rate} Hz (固定，用于 ElevenLabs API)
        </p>
      </div>
    </div>
  );
}
