import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { useQueryClient } from "@tanstack/react-query";
import i18n from "../i18n";
import { setBackendLanguage } from "../lib/api";
import {
  applyAppSettings,
  DEFAULT_APP_SETTINGS,
  loadAppSettings,
  resetAppSettings,
  saveAppSettings,
  type AppSettings,
  type FontScale,
  type SettingsLanguage,
  type ThemeMode,
} from "../lib/appSettings";
import type { EditorPanel } from "../lib/editorPanels";

interface AppSettingsContextValue {
  settings: AppSettings;
  setTheme: (theme: ThemeMode) => void;
  setFontScale: (fontScale: FontScale) => void;
  setLanguage: (language: SettingsLanguage) => void;
  setReducedMotion: (enabled: boolean) => void;
  setCompactDensity: (enabled: boolean) => void;
  setDefaultEditorPanel: (panel: EditorPanel) => void;
  reset: () => void;
}

const AppSettingsContext = createContext<AppSettingsContextValue | null>(null);

export function AppSettingsProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const [settings, setSettings] = useState<AppSettings>(() => loadAppSettings());

  const commit = useCallback((updater: (current: AppSettings) => AppSettings) => {
    setSettings((current) => {
      const next = updater(current);
      saveAppSettings(next);
      applyAppSettings(next);
      return next;
    });
  }, []);

  useEffect(() => {
    applyAppSettings(settings);
  }, [settings]);

  useEffect(() => {
    if (settings.theme !== "system" || typeof window === "undefined" || !window.matchMedia) {
      return;
    }
    const media = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () => applyAppSettings(settings);
    media.addEventListener?.("change", onChange);
    return () => media.removeEventListener?.("change", onChange);
  }, [settings]);

  useEffect(() => {
    if (i18n.resolvedLanguage?.slice(0, 2) !== settings.language) {
      void i18n.changeLanguage(settings.language);
    }
    void setBackendLanguage(settings.language).catch(() => {});
  }, [settings.language]);

  const value = useMemo<AppSettingsContextValue>(
    () => ({
      settings,
      setTheme: (theme) => commit((current) => ({ ...current, theme })),
      setFontScale: (fontScale) => commit((current) => ({ ...current, fontScale })),
      setLanguage: (language) => {
        commit((current) => ({ ...current, language }));
        void queryClient.invalidateQueries({ queryKey: ["parameters"] });
      },
      setReducedMotion: (reducedMotion) =>
        commit((current) => ({ ...current, reducedMotion })),
      setCompactDensity: (compactDensity) =>
        commit((current) => ({ ...current, compactDensity })),
      setDefaultEditorPanel: (defaultEditorPanel) =>
        commit((current) => ({ ...current, defaultEditorPanel })),
      reset: () => {
        const next = resetAppSettings();
        setSettings(next);
        void queryClient.invalidateQueries({ queryKey: ["parameters"] });
      },
    }),
    [commit, queryClient, settings],
  );

  return (
    <AppSettingsContext.Provider value={value}>
      {children}
    </AppSettingsContext.Provider>
  );
}

export function useAppSettings(): AppSettingsContextValue {
  const value = useContext(AppSettingsContext);
  if (!value) {
    return {
      settings: DEFAULT_APP_SETTINGS,
      setTheme: () => {},
      setFontScale: () => {},
      setLanguage: () => {},
      setReducedMotion: () => {},
      setCompactDensity: () => {},
      setDefaultEditorPanel: () => {},
      reset: () => {},
    };
  }
  return value;
}
