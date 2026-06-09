import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { setAppBackgroundMode } from "../lib/api";

const HIDE_DELAY_MS = 400;
const SHOW_DELAY_MS = 120;

const AppWindowFocusContext = createContext(true);

export function useAppWindowFocused(): boolean {
  return useContext(AppWindowFocusContext);
}

/** Фокус ОС + скрытие окна в фоне (WebView2 не конкурирует с fullscreen-игрой). */
export function AppWindowFocusProvider({ children }: { children: ReactNode }) {
  const [tauriFocused, setTauriFocused] = useState(true);
  const [docVisible, setDocVisible] = useState(
    () => typeof document !== "undefined" && document.visibilityState === "visible",
  );
  const hideTimer = useRef<number | undefined>(undefined);
  const showTimer = useRef<number | undefined>(undefined);
  const hiddenRef = useRef(false);
  const focusedRef = useRef(true);
  const focused = tauriFocused && docVisible;

  focusedRef.current = focused;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const window = getCurrentWindow();

    void window.isFocused().then(setTauriFocused);
    void window.onFocusChanged(({ payload }) => setTauriFocused(payload)).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const onVisibility = () => setDocVisible(document.visibilityState === "visible");
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  useEffect(() => {
    window.clearTimeout(hideTimer.current);
    window.clearTimeout(showTimer.current);

    if (focused) {
      if (!hiddenRef.current) return;

      showTimer.current = window.setTimeout(() => {
        if (!focusedRef.current || !hiddenRef.current) return;
        hiddenRef.current = false;
        void getCurrentWindow().show();
        void setAppBackgroundMode(false);
      }, SHOW_DELAY_MS);

      return () => window.clearTimeout(showTimer.current);
    }

    hideTimer.current = window.setTimeout(() => {
      if (focusedRef.current) return;
      hiddenRef.current = true;
      void setAppBackgroundMode(true);
      void getCurrentWindow().hide();
    }, HIDE_DELAY_MS);

    return () => window.clearTimeout(hideTimer.current);
  }, [focused]);

  return (
    <AppWindowFocusContext.Provider value={focused}>
      {children}
    </AppWindowFocusContext.Provider>
  );
}
