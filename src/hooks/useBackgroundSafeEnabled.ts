import { useAppWindowFocused } from "../context/AppWindowFocusProvider";

/** Do not block UI queries on focus — Rust already throttles background load via setAppBackgroundMode. */
export function useBackgroundSafeEnabled(enabled = true): boolean {
  return enabled;
}

/** Process polling — only while the GSM window is focused. */
export function usePollingEnabled(enabled = true): boolean {
  const focused = useAppWindowFocused();
  return focused && enabled;
}
