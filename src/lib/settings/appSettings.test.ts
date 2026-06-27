import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  APP_SETTINGS_STORAGE_KEY,
  applyAppSettings,
  loadAppSettings,
  resolveTheme,
  saveAppSettings,
  sanitizeAppSettings,
  type AppSettings,
} from "./appSettings";

const base: AppSettings = {
  theme: "dark",
  fontScale: 1,
  language: "ru",
  reducedMotion: false,
  compactDensity: false,
  defaultEditorPanel: "basic",
};

describe("appSettings", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.removeAttribute("data-reduced-motion");
    document.documentElement.removeAttribute("data-density");
    document.documentElement.style.removeProperty("--app-font-scale");
  });

  it("persists and loads settings", () => {
    saveAppSettings({ ...base, theme: "light", fontScale: 1.25, language: "en" });
    expect(loadAppSettings()).toMatchObject({
      theme: "light",
      fontScale: 1.25,
      language: "en",
    });
    expect(localStorage.getItem(APP_SETTINGS_STORAGE_KEY)).toContain("\"theme\":\"light\"");
  });

  it("sanitizes malformed values", () => {
    localStorage.setItem("uesm:language", "ru");
    expect(
      sanitizeAppSettings({
        theme: "neon",
        fontScale: 2,
        language: "de",
        defaultEditorPanel: "engine",
      }),
    ).toMatchObject({
      theme: "system",
      fontScale: 1,
      language: "ru",
      defaultEditorPanel: "basic",
    });
  });

  it("defaults to English for non-Russian system locale on first run", () => {
    vi.stubGlobal("navigator", { language: "de-DE", languages: ["de-DE"] });
    expect(sanitizeAppSettings(null).language).toBe("en");
    vi.unstubAllGlobals();
  });

  it("resolves system theme through matchMedia", () => {
    vi.stubGlobal("matchMedia", (query: string) => ({
      matches: query.includes("light"),
      media: query,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    }));
    expect(resolveTheme("system")).toBe("light");
    vi.unstubAllGlobals();
  });

  it("applies theme, font scale, motion, and density to html", () => {
    applyAppSettings({
      ...base,
      theme: "light",
      fontScale: 1.125,
      reducedMotion: true,
      compactDensity: true,
    });
    expect(document.documentElement.dataset.theme).toBe("light");
    expect(document.documentElement.dataset.reducedMotion).toBe("true");
    expect(document.documentElement.dataset.density).toBe("compact");
    expect(document.documentElement.style.getPropertyValue("--app-font-scale")).toBe("1.125");
  });
});
