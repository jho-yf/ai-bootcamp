// src/api/tauri.ts

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AppConfig, AudioDevice, TranscriptionResult } from "./types";

/** 更新状态类型 */
export type UpdateStatus =
  | { type: "Checking" }
  | { type: "Available"; version: string; notes: string }
  | { type: "Downloading"; progress: number }
  | { type: "ReadyToInstall" }
  | { type: "UpToDate" }
  | { type: "Error"; message: string };

/** 更新信息类型 */
export interface UpdateInfo {
  version: string;
  notes: string;
  pub_date: string;
  download_url: string;
  signature_url?: string;
}

/** 音频相关 API */
export const audioApi = {
  /** 获取音频设备列表 */
  getDevices: () => invoke<AudioDevice[]>("get_audio_devices"),

  /** 开始录音 */
  startRecording: (deviceId?: string) =>
    invoke("start_recording", { deviceId }),

  /** 停止录音 */
  stopRecording: () =>
    invoke("stop_recording"),

  /** 测试麦克风 */
  testMicrophone: (deviceId: string) =>
    invoke<boolean>("test_microphone", { deviceId }),

  /** 获取录音状态 */
  getRecordingState: () =>
    invoke<boolean>("get_recording_state"),
};

/** 录音相关 API（带验证） */
export const recordingApi = {
  /** 热键触发的录音切换（包含前置验证） */
  toggleRecording: () =>
    invoke("toggle_recording"),

  /** 检查录音功能是否可用 */
  checkAvailability: () =>
    invoke<{ available: boolean; issues: string[] }>("check_recording_availability"),
};

/** 配置相关 API */
export const configApi = {
  /** 获取配置 */
  getConfig: () => invoke<AppConfig>("get_config"),

  /** 保存配置 */
  saveConfig: (config: AppConfig) =>
    invoke("save_config", { config }),

  /** 重置配置 */
  resetConfig: () => invoke<AppConfig>("reset_config"),

  /** 获取配置 Schema */
  getConfigSchema: () =>
    invoke<Record<string, unknown>>("get_config_schema"),
};

/** 事件监听 */
export const events = {
  /** 监听录音开始事件 */
  onRecordingStarted: (callback: () => void) =>
    listen("recording-started", () => callback()),

  /** 监听录音停止事件 */
  onRecordingStopped: (callback: (text?: string) => void) =>
    listen("recording-stopped", (event) => callback(event.payload as string | undefined)),

  /** 监听转录结果事件 */
  onTranscriptionResult: (callback: (result: TranscriptionResult) => void) =>
    listen("transcription-result", (event) => callback(event.payload as TranscriptionResult)),

  /** 监听部分转录结果事件 */
  onPartialTranscription: (callback: (text: string) => void) =>
    listen("partial_transcription", (event) => callback(event.payload as string)),

  /** 监听错误事件 */
  onError: (callback: (error: string) => void) =>
    listen("error", (event) => callback(event.payload as string)),

  /** 监听连接状态变化事件 */
  onConnectionStateChanged: (callback: (connected: boolean) => void) =>
    listen("connection-state-changed", (event) => callback(event.payload as boolean)),

  /** 监听显示错误事件（用于热键触发时的错误提示） */
  onShowError: (callback: (error: string) => void) =>
    listen("show-error", (event) => callback(event.payload as string)),
};

/** 热键相关 API */
export const hotkeyApi = {
  /** 注册热键 */
  registerHotkey: (modifiers: string[], key: string) =>
    invoke("register_hotkey", { modifiers, key }),

  /** 注销热键 */
  unregisterHotkey: () =>
    invoke("unregister_hotkey"),

  /** 测试热键 */
  testHotkey: () =>
    invoke<boolean>("test_hotkey"),
};

/** 更新相关 API */
export const updaterApi = {
  /** 检查更新 */
  checkForUpdates: (updateUrl: string) =>
    invoke<UpdateInfo>("check_for_updates", { updateUrl }),

  /** 获取更新状态 */
  getUpdateStatus: () =>
    invoke<UpdateStatus>("get_update_status"),
};

/** 更新事件监听 */
export const updateEvents = {
  /** 监听更新状态变化 */
  onUpdateStatusChanged: (callback: (status: UpdateStatus) => void) =>
    listen("update-status", (event) => callback(event.payload as UpdateStatus)),
};
