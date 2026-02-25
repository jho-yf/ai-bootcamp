// src/App.tsx

import { useEffect, useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useConfigStore } from "./stores/configStore";
import { useAudioStore } from "./stores/audioStore";
import { useUIStore } from "./stores/uiStore";
import { events } from "./api/tauri";
import type { TranscriptionResult } from "./api/types";
import { Settings } from "./components/Settings";
import { StatusIndicator } from "./components/StatusIndicator";
import { Notification } from "./components/Notification";

function App() {
  const { loadConfig, config } = useConfigStore();
  const {
    setRecording,
    setPartialText,
    setFinalText,
    setConnected,
    setConnecting,
    isRecording,
  } = useAudioStore();
  const { showNotification, openSettings, closeSettings } = useUIStore();
  const [hotkeyRegistered, setHotkeyRegistered] = useState(false);
  const hotkeyUnlistenRef = useRef<(() => void) | null>(null);

  // Register global hotkey
  const registerHotkey = async () => {
    try {
      const hotkeyConfig = config?.hotkey;
      if (hotkeyConfig) {
        const modifiers = hotkeyConfig.modifiers || [];
        const key = typeof hotkeyConfig.key === 'string' ? hotkeyConfig.key : 'o';
        await invoke("register_hotkey", { modifiers, key });
        setHotkeyRegistered(true);
        console.log("Hotkey registered:", modifiers, key);
      }
    } catch (error) {
      console.error("Failed to register hotkey:", error);
    }
  };

  // Initialize config and event listeners on mount
  useEffect(() => {
    loadConfig();

    // Setup hotkey event listener
    const setupHotkeyListener = async () => {
      try {
        const unlisten = await listen("hotkey-triggered", async () => {
          console.log("Frontend: Hotkey triggered, isRecording:", isRecording);
          try {
            if (isRecording) {
              console.log("Stopping recording...");
              await invoke("stop_recording");
            } else {
              console.log("Starting recording...");
              await invoke("start_recording", { deviceId: null });
            }
          } catch (error) {
            console.error("Failed to toggle recording:", error);
            showNotification({
              type: "error",
              title: "操作失败",
              message: String(error),
            });
          }
        });
        hotkeyUnlistenRef.current = unlisten;
        console.log("Hotkey event listener registered");
      } catch (error) {
        console.error("Failed to setup hotkey listener:", error);
      }
    };

    // Setup other event listeners
    const setupOtherListeners = () => {
      // Recording events
      events.onRecordingStarted(() => {
        console.log("Recording started event received");
        setRecording(true);
        showNotification({
          type: "info",
          title: "录音开始",
          message: "再次按下快捷键结束录音",
          duration: 2000,
        });
      });

      events.onRecordingStopped((text) => {
        console.log("Recording stopped event received, text:", text);
        setRecording(false);
        if (text) {
          showNotification({
            type: "success",
            title: "转录完成",
            message: `识别文本: ${text.substring(0, 50)}${text.length > 50 ? "..." : ""}`,
          });
        }
      });

      // Transcription events
      events.onTranscriptionResult((result: TranscriptionResult) => {
        if (result.is_final) {
          setFinalText(result.text);
          setPartialText("");
        }
      });

      events.onPartialTranscription((text: string) => {
        setPartialText(text);
      });

      // Connection events
      events.onConnectionStateChanged((connected: boolean) => {
        setConnected(connected);
        if (connected) {
          showNotification({
            type: "success",
            title: "已连接",
            message: "已连接到转录服务",
            duration: 2000,
          });
        } else {
          showNotification({
            type: "warning",
            title: "连接断开",
            message: "与转录服务的连接已断开",
          });
        }
      });

      // Error events
      events.onError((error: string) => {
        showNotification({
          type: "error",
          title: "发生错误",
          message: error,
        });
      });
    };

    setupHotkeyListener();
    setupOtherListeners();
    registerHotkey();

    // Cleanup
    return () => {
      if (hotkeyUnlistenRef.current) {
        hotkeyUnlistenRef.current();
      }
    };
  }, []);

  // Global keyboard shortcut to open settings (Escape)
  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        closeSettings();
      }
      // Ctrl+, to open settings
      if (e.ctrlKey && e.key === ",") {
        e.preventDefault();
        openSettings();
      }
    };

    window.addEventListener("keydown", handleKeyPress);
    return () => window.removeEventListener("keydown", handleKeyPress);
  }, []);

  return (
    <div className="min-h-screen bg-gray-50">
      {/* Main Content */}
      <div className="container mx-auto px-4 py-8">
        <div className="max-w-2xl mx-auto">
          {/* Header */}
          <div className="text-center mb-8">
            <h1 className="text-3xl font-bold text-gray-900">
              RaFlow
            </h1>
            <p className="text-gray-500 mt-2">
              实时语音输入工具
            </p>
          </div>

          {/* Status Card */}
          <div className="bg-white rounded-lg shadow-md p-6 mb-6">
            <h2 className="text-lg font-semibold mb-4">状态</h2>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-500">快捷键:</span>
                <span className="font-mono">Ctrl + Shift + O</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">当前状态:</span>
                <span>{isRecording ? "录音中" : "就绪"}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-500">热键注册:</span>
                <span>{hotkeyRegistered ? "已注册" : "未注册"}</span>
              </div>
            </div>
          </div>

          {/* Quick Actions */}
          <div className="bg-white rounded-lg shadow-md p-6 mb-6">
            <h2 className="text-lg font-semibold mb-4">快速操作</h2>
            <div className="grid grid-cols-2 gap-4">
              <button
                onClick={openSettings}
                className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
              >
                打开设置
              </button>
              <button
                onClick={() => invoke("open_tray_menu")}
                className="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
              >
                托盘菜单
              </button>
            </div>
          </div>

          {/* Instructions */}
          <div className="bg-blue-50 rounded-lg p-6">
            <h3 className="font-semibold mb-2">使用说明</h3>
            <ol className="text-sm text-gray-600 space-y-2 list-decimal list-inside">
              <li>按下快捷键 <kbd className="px-2 py-1 bg-gray-200 rounded">Ctrl + Shift + O</kbd> 开始录音</li>
              <li>说话，应用会实时转录您的语音</li>
              <li>再次按下快捷键结束录音</li>
              <li>识别的文字会自动插入到光标位置</li>
            </ol>
          </div>
        </div>
      </div>

      {/* Overlays */}
      <Settings />
      <StatusIndicator />
      <Notification />
    </div>
  );
}

export default App;
