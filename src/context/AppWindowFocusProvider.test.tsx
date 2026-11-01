import { act, render, screen } from "@testing-library/react";
import { beforeEach, expect, it, vi } from "vitest";
import {
  AppWindowFocusProvider,
  useAppWindowFocused,
} from "./AppWindowFocusProvider";

const mocks = vi.hoisted(() => ({
  isFocused: vi.fn(),
  onFocusChanged: vi.fn(),
}));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => mocks }));
vi.mock("@/lib/api/tauriRuntime", () => ({ isTauriRuntime: () => true }));

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
function Probe() {
  return <output>{useAppWindowFocused() ? "focused" : "background"}</output>;
}
beforeEach(() => {
  vi.clearAllMocks();
});

it("releases a focus subscription that finishes after unmount", async () => {
  const subscription = deferred<() => void>();
  mocks.isFocused.mockResolvedValue(true);
  mocks.onFocusChanged.mockReturnValue(subscription.promise);
  const unlisten = vi.fn();
  const { unmount } = render(
    <AppWindowFocusProvider>
      <Probe />
    </AppWindowFocusProvider>,
  );
  unmount();
  await act(async () => subscription.resolve(unlisten));
  expect(unlisten).toHaveBeenCalledOnce();
});

it("does not overwrite a newer focus event with a late initial response", async () => {
  const initial = deferred<boolean>();
  mocks.isFocused.mockReturnValue(initial.promise);
  mocks.onFocusChanged.mockResolvedValue(vi.fn());
  render(
    <AppWindowFocusProvider>
      <Probe />
    </AppWindowFocusProvider>,
  );
  act(() => mocks.onFocusChanged.mock.calls[0][0]({ payload: false }));
  expect(screen.getByText("background")).toBeInTheDocument();
  await act(async () => initial.resolve(true));
  expect(screen.getByText("background")).toBeInTheDocument();
});
