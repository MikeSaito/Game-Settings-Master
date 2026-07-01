import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import "../../i18n";
import { SettingsPanel } from "./SettingsPanel";

const handlers = vi.hoisted(() => ({
  setLanguage: vi.fn(),
  setTheme: vi.fn(),
  setFontScale: vi.fn(),
  setReducedMotion: vi.fn(),
  setCompactDensity: vi.fn(),
  setDefaultEditorPanel: vi.fn(),
  setCrashReportsEnabled: vi.fn(),
  reset: vi.fn(),
}));

vi.mock("@/lib/api", () => ({
  listCrashReports: vi.fn(() => Promise.resolve([])),
  clearCrashReports: vi.fn(() => Promise.resolve()),
}));

vi.mock("@/hooks/app/useAppSettings", () => ({
  useAppSettings: () => ({
    settings: {
      theme: "dark",
      fontScale: 1,
      language: "ru",
      reducedMotion: false,
      compactDensity: false,
      defaultEditorPanel: "basic",
      crashReportsEnabled: false,
    },
    ...handlers,
  }),
}));

function renderPanel() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <SettingsPanel open onClose={() => {}} />
    </QueryClientProvider>,
  );
}

describe("SettingsPanel", () => {
  it("changes language through settings handler", async () => {
    renderPanel();
    await userEvent.selectOptions(screen.getByLabelText(/interface language|язык интерфейса/i), "en");
    expect(handlers.setLanguage).toHaveBeenCalledWith("en");
  });
});
