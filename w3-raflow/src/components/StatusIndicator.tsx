// src/components/StatusIndicator.tsx

import { Mic, X } from "lucide-react";
import { useAudioStore } from "../stores/audioStore";

export function StatusIndicator() {
  const {
    isRecording,
    partialText,
    recordingDuration,
    setRecording,
  } = useAudioStore();

  if (!isRecording) return null;

  const formatDuration = (ms: number): string => {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
  };

  const handleStop = () => {
    setRecording(false);
  };

  return (
    <div className="fixed top-20 left-1/2 -translate-x-1/2 bg-black/80 dark:bg-black/90 backdrop-blur text-white px-6 py-3 rounded-full shadow-2xl flex items-center space-x-4 z-50 animate-in fade-in slide-in-from-top-4 duration-300">
      {/* Recording Icon Animation */}
      <div className="relative">
        <Mic className="w-5 h-5 text-red-500" />
        <div className="absolute inset-0 bg-red-500/30 rounded-full animate-ping" />
      </div>

      {/* Separator */}
      <div className="w-px h-6 bg-gray-600" />

      {/* Real-time Text Preview */}
      <div className="max-w-md">
        <p className="text-sm">
          {partialText || (
            <span className="text-gray-400">正在聆听...</span>
          )}
        </p>
      </div>

      {/* Separator */}
      <div className="w-px h-6 bg-gray-600" />

      {/* Duration */}
      <div className="font-mono text-sm min-w-[50px]">
        {formatDuration(recordingDuration)}
      </div>

      {/* Close Button */}
      <button
        onClick={handleStop}
        className="ml-2 p-1 hover:bg-white/10 rounded-full transition-colors"
        title="停止录音"
      >
        <X className="w-4 h-4" />
      </button>
    </div>
  );
}
