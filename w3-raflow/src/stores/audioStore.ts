// src/stores/audioStore.ts

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { AudioDevice, TranscriptionResult } from "../api/types";

interface AudioState {
  // Recording state
  isRecording: boolean;
  recordingDuration: number;
  recordingStartTime: number | null;

  // Audio devices
  devices: AudioDevice[];
  currentDevice: string | null;

  // Transcription
  partialText: string;
  finalText: string;
  confidence: number;
  lastResult: TranscriptionResult | null;

  // Connection state
  isConnected: boolean;
  isConnecting: boolean;

  // Actions
  setRecording: (recording: boolean) => void;
  setRecordingDuration: (duration: number) => void;
  startRecording: () => void;
  stopRecording: () => void;
  setDevices: (devices: AudioDevice[]) => void;
  setCurrentDevice: (device: string) => void;
  setPartialText: (text: string) => void;
  setFinalText: (text: string) => void;
  setConfidence: (confidence: number) => void;
  setLastResult: (result: TranscriptionResult | null) => void;
  clearTranscription: () => void;
  setConnected: (connected: boolean) => void;
  setConnecting: (connecting: boolean) => void;
}

export const useAudioStore = create<AudioState>()(
  persist(
    (set, get) => ({
      // Initial state
      isRecording: false,
      recordingDuration: 0,
      recordingStartTime: null,
      devices: [],
      currentDevice: null,
      partialText: "",
      finalText: "",
      confidence: 0,
      lastResult: null,
      isConnected: false,
      isConnecting: false,

      // Recording actions
      setRecording: (recording) => set({ isRecording: recording }),

      setRecordingDuration: (duration) => set({ recordingDuration: duration }),

      startRecording: () => {
        set({
          isRecording: true,
          recordingStartTime: Date.now(),
          recordingDuration: 0,
          partialText: "",
          finalText: "",
        });

        // Start duration timer
        const startTime = Date.now();
        const interval = setInterval(() => {
          const state = get();
          if (!state.isRecording) {
            clearInterval(interval);
            return;
          }
          set({ recordingDuration: Date.now() - startTime });
        }, 100);
      },

      stopRecording: () => {
        set({
          isRecording: false,
          recordingStartTime: null,
        });
      },

      // Device actions
      setDevices: (devices) => set({ devices }),
      setCurrentDevice: (device) => set({ currentDevice: device }),

      // Transcription actions
      setPartialText: (text) => set({ partialText: text }),
      setFinalText: (text) => set({ finalText: text }),
      setConfidence: (confidence) => set({ confidence }),
      setLastResult: (result) => set({ lastResult: result }),
      clearTranscription: () => set({ partialText: "", finalText: "", confidence: 0, lastResult: null }),

      // Connection actions
      setConnected: (connected) => set({ isConnected: connected }),
      setConnecting: (connecting) => set({ isConnecting: connecting }),
    }),
    {
      name: "audio-storage",
      partialize: (state) => ({
        currentDevice: state.currentDevice,
      }),
    }
  )
);
