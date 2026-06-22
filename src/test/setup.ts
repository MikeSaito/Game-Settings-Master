import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import React from "react";
import { afterEach, vi } from "vitest";
import { mockInvoke } from "./mockTauri";

afterEach(() => {
  cleanup();
});

vi.mock("@tauri-apps/api/core", () => ({
  invoke: mockInvoke,
}));

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => {
    const visibleCount = Math.min(count, 8);
    const ROW_ESTIMATE_PX = 120;
    return {
      getTotalSize: () => count * ROW_ESTIMATE_PX,
      getVirtualItems: () =>
        Array.from({ length: visibleCount }, (_, index) => ({
          index,
          start: index * ROW_ESTIMATE_PX,
        })),
      measureElement: () => {},
      scrollToIndex: () => {},
    };
  },
}));

vi.mock("@/components/app/UpdateGate", () => ({
  UpdateGate: ({ children }: { children: React.ReactNode }) =>
    React.createElement(React.Fragment, null, children),
}));
