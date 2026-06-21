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
import { isTauriRuntime } from "../lib/tauriRuntime";

const HIDE_DELAY_MS = 400;
const SHOW_DELAY_MS = 120;

const AppWindowFocusContext = createContext(true);

export function useAppWindowFocused(): boolean {
  return useContext(AppWindowFocusContext);
}

/** OS focus + background window hiding (WebView2 does not compete with fullscreen game). */
export function AppWindowFocusProvider({ children }: { children: ReactNode }) {
  const inTauri = isTauriRuntime();
  const backgroundHideEnabled = inTauri && !import.meta.env.DEV;
  const [tauriFocused, setTauriFocused] = useState(true);
  const [docVisible, setDocVisible] = useState(
    () => typeof document !== "undefined" && document.visibilityState === "visible",
  );
  const hideTimer = useRef<number | undefined>(undefined);
  const showTimer = useRef<number | undefined>(undefined);
  const hiddenRef = useRef(false);
  const focusedRef = useRef(true);
  const hasBeenFocusedRef = useRef(false);
  const startupGraceUntilRef = useRef(
    typeof performance !== "undefined" ? performance.now() + 2000 : 0,
  );
  const focused = inTauri ? tauriFocused && docVisible : docVisible;

  focusedRef.current = focused;

  useEffect(() => {
    if (!inTauri) return;

    let unlisten: (() => void) | undefined;
    const window = getCurrentWindow();

    void window.isFocused().then((isFocused) => {
      setTauriFocused(isFocused);
      if (isFocused) hasBeenFocusedRef.current = true;
    });
    void window.onFocusChanged(({ payload }) => {
      setTauriFocused(payload);
      if (payload) hasBeenFocusedRef.current = true;
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
    if (!backgroundHideEnabled) return;

    window.clearTimeout(hideTimer.current);
    window.clearTimeout(showTimer.current);

    if (focused) {
      if (!hiddenRef.current) return;

      showTimer.current = window.setTimeout(() => {
        if (!focusedRef.current || !hiddenRef.current) return;
        hiddenRef.current = false;
        void setAppBackgroundMode(false);
      }, SHOW_DELAY_MS);

      return () => window.clearTimeout(showTimer.current);
    }

    hideTimer.current = window.setTimeout(() => {
      if (focusedRef.current) return;
      if (!hasBeenFocusedRef.current) return;
      if (typeof performance !== "undefined" && performance.now() < startupGraceUntilRef.current) {
        return;
      }
      hiddenRef.current = true;
      void setAppBackgroundMode(true);
    }, HIDE_DELAY_MS);

    return () => window.clearTimeout(hideTimer.current);
  }, [focused, backgroundHideEnabled]);

  return (
    <AppWindowFocusContext.Provider value={focused}>
      {children}
    </AppWindowFocusContext.Provider>
  );
}
