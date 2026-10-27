import { renderHook } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useStartupWindow } from "./useStartupWindow";

vi.mock("@/lib/api/tauriRuntime", () => ({ isTauriRuntime: () => true }));
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

it("reveals the themed native window even when hidden WebView2 suspends animation frames", async () => {
  vi.useFakeTimers();
  let paint: FrameRequestCallback | undefined;
  vi.stubGlobal(
    "requestAnimationFrame",
    vi.fn((callback: FrameRequestCallback) => {
      paint = callback;
      return 1;
    }),
  );
  vi.stubGlobal("cancelAnimationFrame", vi.fn());
  vi.mocked(invoke).mockResolvedValue(undefined);
  document.documentElement.dataset.theme = "light";
  renderHook(useStartupWindow);
  await vi.advanceTimersByTimeAsync(100);
  expect(invoke).toHaveBeenCalledWith("show_main_window", { theme: "light" });
  paint?.(101);
  expect(invoke).toHaveBeenCalledOnce();
});

it("cancels pending reveal work on unmount", async () => {
  vi.useFakeTimers();
  vi.stubGlobal(
    "requestAnimationFrame",
    vi.fn(() => 1),
  );
  vi.stubGlobal("cancelAnimationFrame", vi.fn());
  const { unmount } = renderHook(useStartupWindow);
  unmount();
  await vi.advanceTimersByTimeAsync(500);
  expect(invoke).not.toHaveBeenCalled();
});
