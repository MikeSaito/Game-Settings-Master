import type { EditorPanel } from "./editorPanels";

export type ThemeMode = "dark" | "light" | "system";
export type FontScale = 0.875 | 1 | 1.125 | 1.25;
export type SettingsLanguage = "ru" | "en";

export interface AppSettings {
  theme: ThemeMode;
  fontScale: FontScale;
  language: SettingsLanguage;
  reducedMotion: boolean;
  compactDensity: boolean;
  defaultEditorPanel: EditorPanel;
}

export const APP_SETTINGS_STORAGE_KEY = "gsm-app-settings";
const I18N_LANGUAGE_STORAGE_KEY = "uesm:language";

export const DEFAULT_APP_SETTINGS: AppSettings = {
  theme: "dark",
  fontScale: 1,
  language: "ru",
  reducedMotion: false,
  compactDensity: false,
  defaultEditorPanel: "basic",
};

const FONT_SCALES: FontScale[] = [0.875, 1, 1.125, 1.25];

function isThemeMode(value: unknown): value is ThemeMode {
  return value === "dark" || value === "light" || value === "system";
}

function isLanguage(value: unknown): value is SettingsLanguage {
  return value === "ru" || value === "en";
}

function isFontScale(value: unknown): value is FontScale {
  return typeof value === "number" && FONT_SCALES.includes(value as FontScale);
}

function isEditorPanel(value: unknown): value is EditorPanel {
  return value === "basic" || value === "advanced";
}

function detectedLanguage(): SettingsLanguage {
  if (typeof localStorage !== "undefined") {
    const stored = localStorage.getItem(I18N_LANGUAGE_STORAGE_KEY);
    if (isLanguage(stored)) return stored;
  }
  if (typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("en")) {
    return "en";
  }
  return "ru";
}

export function resolveTheme(theme: ThemeMode): "dark" | "light" {
  if (theme !== "system") return theme;
  if (typeof window === "undefined" || !window.matchMedia) return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

export function sanitizeAppSettings(value: unknown): AppSettings {
  const input = value && typeof value === "object" ? (value as Partial<AppSettings>) : {};
  return {
    theme: isThemeMode(input.theme) ? input.theme : DEFAULT_APP_SETTINGS.theme,
    fontScale: isFontScale(input.fontScale) ? input.fontScale : DEFAULT_APP_SETTINGS.fontScale,
    language: isLanguage(input.language) ? input.language : detectedLanguage(),
    reducedMotion:
      typeof input.reducedMotion === "boolean"
        ? input.reducedMotion
        : DEFAULT_APP_SETTINGS.reducedMotion,
    compactDensity:
      typeof input.compactDensity === "boolean"
        ? input.compactDensity
        : DEFAULT_APP_SETTINGS.compactDensity,
    defaultEditorPanel: isEditorPanel(input.defaultEditorPanel)
      ? input.defaultEditorPanel
      : DEFAULT_APP_SETTINGS.defaultEditorPanel,
  };
}

export function loadAppSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(APP_SETTINGS_STORAGE_KEY);
    return sanitizeAppSettings(raw ? JSON.parse(raw) : null);
  } catch {
    return sanitizeAppSettings(null);
  }
}

export function saveAppSettings(settings: AppSettings): void {
  try {
    localStorage.setItem(APP_SETTINGS_STORAGE_KEY, JSON.stringify(settings));
    localStorage.setItem(I18N_LANGUAGE_STORAGE_KEY, settings.language);
  } catch {
    /* ignore */
  }
}

export function applyAppSettings(settings: AppSettings): void {
  if (typeof document === "undefined") return;
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(settings.theme);
  root.dataset.themeMode = settings.theme;
  root.dataset.reducedMotion = settings.reducedMotion ? "true" : "false";
  root.dataset.density = settings.compactDensity ? "compact" : "comfortable";
  root.style.setProperty("--app-font-scale", String(settings.fontScale));
}

export function resetAppSettings(): AppSettings {
  const next = { ...DEFAULT_APP_SETTINGS, language: detectedLanguage() };
  saveAppSettings(next);
  applyAppSettings(next);
  return next;
}

if (typeof document !== "undefined") {
  applyAppSettings(loadAppSettings());
}
