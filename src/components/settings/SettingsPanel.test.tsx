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
  reset: vi.fn(),
}));

vi.mock("../../hooks/useAppSettings", () => ({
  useAppSettings: () => ({
    settings: {
      theme: "dark",
      fontScale: 1,
      language: "ru",
      reducedMotion: false,
      compactDensity: false,
      defaultEditorPanel: "basic",
    },
    ...handlers,
  }),
}));

describe("SettingsPanel", () => {
  it("changes language through settings handler", async () => {
    render(<SettingsPanel open onClose={() => {}} />);
    await userEvent.click(screen.getByRole("tab", { name: "EN" }));
    expect(handlers.setLanguage).toHaveBeenCalledWith("en");
  });
});
