// src/components/StatusIndicator.tsx

import { useState, useEffect, useRef } from "react";
import { Mic, Square, Copy, Check } from "lucide-react";
import { useAudioStore } from "../stores/audioStore";
import { recordingApi } from "../api/tauri";

export function StatusIndicator() {
  const {
    isRecording,
    partialText,
    finalText,
    recordingDuration,
    setRecording,
  } = useAudioStore();

  const [copied, setCopied] = useState(false);
  const [transcriptHistory, setTranscriptHistory] = useState<string[]>([]);
  const scrollRef = useRef<HTMLDivElement>(null);

  // 当 finalText 更新时，添加到历史记录
  useEffect(() => {
    if (finalText && finalText.trim()) {
      setTranscriptHistory((prev) => [...prev, finalText]);
    }
  }, [finalText]);

  // 自动滚动到底部
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [partialText, transcriptHistory]);

  // 录制结束时清空
  useEffect(() => {
    if (!isRecording) {
      setTranscriptHistory([]);
    }
  }, [isRecording]);

  if (!isRecording) return null;

  const formatDuration = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
  };

  const handleStop = async () => {
    try {
      await recordingApi.toggleRecording();
    } catch (error) {
      console.error("Failed to stop recording:", error);
      setRecording(false);
    }
  };

  const handleCopy = async () => {
    const allText = [...transcriptHistory, partialText].filter(Boolean).join(" ");
    if (allText) {
      try {
        await navigator.clipboard.writeText(allText);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
      } catch (error) {
        console.error("Failed to copy:", error);
      }
    }
  };

  // 合并所有文本用于显示
  const allText = [...transcriptHistory, partialText].filter(Boolean).join(" ");

  return (
    <div className="fixed top-4 left-1/2 -translate-x-1/2 w-[600px] max-w-[95vw] bg-black/90 backdrop-blur-lg text-white rounded-2xl shadow-2xl z-50 animate-in fade-in slide-in-from-top-4 duration-300 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-white/10">
        <div className="flex items-center space-x-3">
          {/* Recording Icon */}
          <div className="relative">
            <div className="w-3 h-3 bg-red-500 rounded-full" />
            <div className="absolute inset-0 w-3 h-3 bg-red-500 rounded-full animate-ping" />
          </div>
          <span className="text-sm font-medium">正在录音</span>
          <span className="text-xs text-gray-400 font-mono">
            {formatDuration(recordingDuration)}
          </span>
        </div>

        {/* Actions */}
        <div className="flex items-center space-x-2">
          {/* Copy Button */}
          <button
            onClick={handleCopy}
            className="p-2 hover:bg-white/10 rounded-lg transition-colors"
            title="复制文本"
            disabled={!allText}
          >
            {copied ? (
              <Check className="w-4 h-4 text-green-400" />
            ) : (
              <Copy className="w-4 h-4 text-gray-400" />
            )}
          </button>

          {/* Stop Button */}
          <button
            onClick={handleStop}
            className="flex items-center space-x-2 px-4 py-2 bg-red-500 hover:bg-red-600 rounded-lg transition-colors"
          >
            <Square className="w-4 h-4 fill-current" />
            <span className="text-sm font-medium">停止</span>
          </button>
        </div>
      </div>

      {/* Transcription Display */}
      <div
        ref={scrollRef}
        className="max-h-[200px] overflow-y-auto px-4 py-3"
      >
        {/* History */}
        {transcriptHistory.length > 0 && (
          <div className="text-gray-300 text-sm leading-relaxed mb-2">
            {transcriptHistory.map((text, index) => (
              <span key={index}>
                {text}
                {index < transcriptHistory.length - 1 && " "}
              </span>
            ))}
          </div>
        )}

        {/* Partial Text */}
        {partialText && (
          <div className="text-white text-sm leading-relaxed">
            <span className="text-gray-400 italic">{partialText}</span>
            <span className="inline-block w-1 h-4 bg-white/70 ml-0.5 animate-pulse" />
          </div>
        )}

        {/* Empty State */}
        {!partialText && transcriptHistory.length === 0 && (
          <div className="text-gray-500 text-sm text-center py-4">
            <Mic className="w-6 h-6 mx-auto mb-2 opacity-50" />
            正在聆听，请说话...
          </div>
        )}
      </div>
    </div>
  );
}
