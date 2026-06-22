import { useAppWindowFocused } from "@/context/AppWindowFocusProvider";

/** Heavy IPC queries — only while the GSM window is visible and focused. */
export function useBackgroundSafeEnabled(enabled = true): boolean {
  const focused = useAppWindowFocused();
  return focused && enabled;
}

/** Process polling — only while the GSM window is focused. */
export function usePollingEnabled(enabled = true): boolean {
  const focused = useAppWindowFocused();
  return focused && enabled;
}
