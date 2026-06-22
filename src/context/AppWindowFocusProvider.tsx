import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  createContext,
  useContext,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from "react";
import { setAppBackgroundMode } from "@/lib/api";
import { isTauriRuntime } from "@/lib/api";

const BACKGROUND_DELAY_MS = 400;
const FOREGROUND_DELAY_MS = 120;

const AppWindowFocusContext = createContext(true);

export function useAppWindowFocused(): boolean {
  return useContext(AppWindowFocusContext);
}

/** OS focus tracking + background process priority (IPC throttling while unfocused). */
export function AppWindowFocusProvider({ children }: { children: ReactNode }) {
  const inTauri = isTauriRuntime();
  const backgroundModeEnabled = inTauri && !import.meta.env.DEV;
  const [tauriFocused, setTauriFocused] = useState(true);
  const [docVisible, setDocVisible] = useState(
    () => typeof document !== "undefined" && document.visibilityState === "visible",
  );
  const focusedRef = useRef(true);
  const focused = inTauri ? tauriFocused && docVisible : docVisible;

  focusedRef.current = focused;

  useEffect(() => {
    if (!inTauri) return;

    let unlisten: (() => void) | undefined;
    const window = getCurrentWindow();

    void window.isFocused().then((isFocused) => {
      setTauriFocused(isFocused);
    });
    void window.onFocusChanged(({ payload }) => {
      setTauriFocused(payload);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [inTauri]);

  useEffect(() => {
    const onVisibility = () => setDocVisible(document.visibilityState === "visible");
    document.addEventListener("visibilitychange", onVisibility);
    return () => document.removeEventListener("visibilitychange", onVisibility);
  }, []);

  useEffect(() => {
    if (!backgroundModeEnabled) return;

    const delay = focused ? FOREGROUND_DELAY_MS : BACKGROUND_DELAY_MS;
    const timer = window.setTimeout(() => {
      void setAppBackgroundMode(!focusedRef.current);
    }, delay);

    return () => window.clearTimeout(timer);
  }, [focused, backgroundModeEnabled]);

  return (
    <AppWindowFocusContext.Provider value={focused}>
      {children}
    </AppWindowFocusContext.Provider>
  );
}
