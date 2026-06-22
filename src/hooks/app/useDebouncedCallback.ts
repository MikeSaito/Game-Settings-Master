import { useCallback, useEffect, useRef } from "react";

export function useDebouncedCallback<T extends (...args: never[]) => void>(
  callback: T,
  delayMs: number,
): [T, () => void] {
  const callbackRef = useRef(callback);
  callbackRef.current = callback;
  const timerRef = useRef<number | undefined>(undefined);
  const pendingArgsRef = useRef<Parameters<T> | null>(null);

  const flush = useCallback(() => {
    if (timerRef.current !== undefined) {
      window.clearTimeout(timerRef.current);
      timerRef.current = undefined;
    }
    if (pendingArgsRef.current) {
      const args = pendingArgsRef.current;
      pendingArgsRef.current = null;
      callbackRef.current(...args);
    }
  }, []);

  useEffect(
    () => () => {
      if (timerRef.current !== undefined) {
        window.clearTimeout(timerRef.current);
      }
    },
    [],
  );

  const debounced = useCallback(
    (...args: Parameters<T>) => {
      pendingArgsRef.current = args;
      if (timerRef.current !== undefined) {
        window.clearTimeout(timerRef.current);
      }
      timerRef.current = window.setTimeout(() => {
        timerRef.current = undefined;
        pendingArgsRef.current = null;
        callbackRef.current(...args);
      }, delayMs);
    },
    [delayMs],
  ) as T;

  return [debounced, flush];
}
