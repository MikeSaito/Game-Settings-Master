import { useAppWindowFocused } from "../context/AppWindowFocusProvider";

/** UI-запросы не блокируем по фокусу — Rust уже снижает фоновую нагрузку через setAppBackgroundMode. */
export function useBackgroundSafeEnabled(enabled = true): boolean {
  return enabled;
}

/** Опрос процессов — только когда окно GSM в фокусе. */
export function usePollingEnabled(enabled = true): boolean {
  const focused = useAppWindowFocused();
  return focused && enabled;
}
