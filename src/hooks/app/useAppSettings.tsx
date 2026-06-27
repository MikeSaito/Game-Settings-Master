import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { useQueryClient } from "@tanstack/react-query";
import i18n from "@/i18n";
import { setBackendLanguage } from "@/lib/api";
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
} from "@/lib/settings";
import type { EditorPanel } from "@/lib/routing";

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

function activeLanguageCode(): SettingsLanguage | undefined {
  const code = (i18n.resolvedLanguage ?? i18n.language)?.slice(0, 2);
  return code === "en" || code === "ru" ? code : undefined;
}

export function AppSettingsProvider({ children }: { children: ReactNode }) {
  const queryClient = useQueryClient();
  const [settings, setSettings] = useState<AppSettings>(() => loadAppSettings());
  const settingsRef = useRef(settings);
  const backendLanguageRef = useRef<SettingsLanguage | null>(null);
  settingsRef.current = settings;

  const commit = useCallback((updater: (current: AppSettings) => AppSettings) => {
    setSettings((current) => {
      const next = updater(current);
      saveAppSettings(next);
      applyAppSettings(next);
      return next;
    });
  }, []);

  useEffect(() => {
    if (settings.theme !== "system" || typeof window === "undefined" || !window.matchMedia) {
      return;
    }
    const media = window.matchMedia("(prefers-color-scheme: light)");
    const onChange = () => applyAppSettings(settingsRef.current);
    media.addEventListener?.("change", onChange);
    return () => media.removeEventListener?.("change", onChange);
  }, [settings.theme]);

  useEffect(() => {
    const target = settings.language;
    if (activeLanguageCode() !== target && !i18n.isInitializing) {
      void i18n.changeLanguage(target);
    }
    if (backendLanguageRef.current !== target) {
      backendLanguageRef.current = target;
      void setBackendLanguage(target).catch(() => {});
    }
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
