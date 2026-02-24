// src/api/types.ts

/** 音频设备信息 */
export interface AudioDevice {
  name: string;
  id: string;
  is_default: boolean;
}

/** 键修饰键 */
export type KeyModifier = 'ctrl' | 'alt' | 'shift' | 'super';

/** 按键码 */
export type KeyCode = 'backslash' | 'space' | string;

/** 通用配置 */
export interface GeneralConfig {
  language: string;
  autostart: boolean;
  minimize_to_tray: boolean;
}

/** 音频配置 */
export interface AudioConfig {
  device_id: string;
  sample_rate: number;
  echo_cancellation: boolean;
  noise_suppression: boolean;
  auto_gain: boolean;
}

/** ElevenLabs 配置 */
export interface ElevenLabsConfig {
  api_key: string;
  language: string;
  timeout: number;
}

/** 热键配置 */
export interface HotkeyConfig {
  modifiers: KeyModifier[];
  key: KeyCode;
  enabled: boolean;
}

/** 文本配置 */
export interface TextConfig {
  strategy: 'auto' | 'keyboard' | 'clipboard';
  insertion_delay: number;
}

/** UI 配置 */
export interface UIConfig {
  show_notifications: boolean;
  indicator_opacity: number;
  show_live_preview: boolean;
}

/** 应用配置 */
export interface AppConfig {
  general: GeneralConfig;
  audio: AudioConfig;
  elevenlabs: ElevenLabsConfig;
  hotkey: HotkeyConfig;
  text: TextConfig;
  ui: UIConfig;
}

/** 转录结果 */
export interface TranscriptionResult {
  text: string;
  is_final: boolean;
  confidence: number;
  language?: string;
  timestamp: number;
}

/** 通知 */
export interface Notification {
  id: string;
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message: string;
  duration?: number;
}
