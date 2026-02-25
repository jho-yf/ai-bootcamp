// src/components/settings/About.tsx

import { Mic } from "lucide-react";

export function About() {
  return (
    <div className="space-y-4">
      {/* Logo and Title */}
      <div className="text-center py-4">
        <div className="inline-flex items-center justify-center w-16 h-16 bg-blue-500 rounded-full mb-3">
          <Mic className="w-8 h-8 text-white" />
        </div>
        <h2 className="text-xl font-bold text-gray-900">RaFlow</h2>
        <p className="text-sm text-gray-600">实时语音输入工具</p>
      </div>

      {/* Version Info */}
      <div className="border-t border-b border-gray-200 py-4">
        <div className="flex justify-between text-sm">
          <span className="text-gray-600">版本</span>
          <span className="text-gray-900 font-medium">0.1.0</span>
        </div>
        <div className="flex justify-between text-sm mt-2">
          <span className="text-gray-600">构建</span>
          <span className="text-gray-900 font-medium">2026.02.24</span>
        </div>
      </div>

      {/* Description */}
      <div className="text-sm text-gray-700">
        <p>
          RaFlow 是一款基于 ElevenLabs Scribe v2 API 的跨平台语音输入工具。
          通过简单的快捷键，即可将您的语音实时转换为文字并插入到任何应用程序中。
        </p>
      </div>

      {/* Features */}
      <div>
        <h3 className="text-sm font-medium text-gray-900 mb-2">主要功能</h3>
        <ul className="text-xs text-gray-600 space-y-1">
          <li>• 实时语音转文字</li>
          <li>• 低延迟转录 (&lt;150ms)</li>
          <li>• 多语言支持</li>
          <li>• 智能文本插入</li>
          <li>• 全局快捷键触发</li>
          <li>• 系统托盘常驻</li>
        </ul>
      </div>

      {/* Links */}
      <div className="pt-4 border-t border-gray-200">
        <h3 className="text-sm font-medium text-gray-900 mb-2">相关链接</h3>
        <div className="space-y-2">
          <a
            href="https://elevenlabs.io"
            target="_blank"
            rel="noopener noreferrer"
            className="block text-sm text-blue-600 hover:underline font-medium"
          >
            ElevenLabs 官网
          </a>
          <a
            href="https://github.com/ai-bootcamp/raflow"
            target="_blank"
            rel="noopener noreferrer"
            className="block text-sm text-blue-600 hover:underline font-medium"
          >
            GitHub 仓库
          </a>
          <a
            href="https://github.com/ai-bootcamp/raflow/blob/main/LICENSE"
            target="_blank"
            rel="noopener noreferrer"
            className="block text-sm text-blue-600 hover:underline font-medium"
          >
            许可证 (MIT)
          </a>
        </div>
      </div>

      {/* Credits */}
      <div className="pt-4 border-t border-gray-200 text-center text-xs text-gray-600">
        <p>© 2026 AI Bootcamp Team</p>
        <p className="mt-1">Built with Tauri 2 + React + TypeScript</p>
      </div>
    </div>
  );
}
