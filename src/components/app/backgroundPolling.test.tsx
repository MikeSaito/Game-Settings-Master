import { act, render } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { afterEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { RecoveryPanel } from "./RecoveryPanel";
import { DiscoveryWarnings } from "../library/DiscoveryWarnings";

const state = vi.hoisted(() => ({ focused: true }));
vi.mock("@/hooks/app/useBackgroundSafeEnabled", () => ({
  usePollingEnabled: (enabled = true) => state.focused && enabled,
}));
vi.mock("@/lib/api/tauriRuntime", () => ({ isTauriRuntime: () => true }));
afterEach(() => {
  vi.useRealTimers();
  vi.clearAllMocks();
});

it("suspends recovery and discovery polling while unfocused and resumes on return", async () => {
  vi.useFakeTimers();
  state.focused = true;
  vi.mocked(invoke).mockResolvedValue([]);
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });
  const view = () => (
    <QueryClientProvider client={client}>
      <RecoveryPanel />
      <DiscoveryWarnings />
    </QueryClientProvider>
  );
  const { rerender, unmount } = render(view());
  await act(async () => {
    await vi.advanceTimersByTimeAsync(31_000);
  });
  const foregroundCalls = vi.mocked(invoke).mock.calls.length;
  expect(foregroundCalls).toBeGreaterThan(2);
  state.focused = false;
  rerender(view());
  await act(async () => {
    await vi.advanceTimersByTimeAsync(60_000);
  });
  expect(invoke).toHaveBeenCalledTimes(foregroundCalls);
  state.focused = true;
  rerender(view());
  await act(async () => {
    await vi.advanceTimersByTimeAsync(31_000);
  });
  expect(vi.mocked(invoke).mock.calls.length).toBeGreaterThan(foregroundCalls);
  unmount();
  client.clear();
});
