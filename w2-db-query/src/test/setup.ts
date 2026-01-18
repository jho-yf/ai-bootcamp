import { expect, afterEach } from "vitest";
import { cleanup } from "@testing-library/react";
import * as matchers from "@testing-library/jest-dom/matchers";

// 扩展 Vitest 的 expect 以包含 jest-dom 的匹配器
expect.extend(matchers);

// 每个测试后清理
afterEach(() => {
  cleanup();
});

// Mock Tauri API
if (typeof window !== "undefined") {
  (window as any).__TAURI__ = {
    core: {
      invoke: async (cmd: string, args?: any) => {
        console.log("[Test Mock] Tauri invoke:", cmd, args);
        return null;
      },
    },
  };
}
