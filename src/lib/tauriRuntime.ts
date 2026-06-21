import { convertFileSrc } from "@tauri-apps/api/core";

/** True when the UI runs inside a Tauri webview (not plain Vite in browser). */
export function isTauriRuntime(): boolean {
  return (
    typeof window !== "undefined" &&
    typeof (window as Window & { __TAURI_INTERNALS__?: unknown })
      .__TAURI_INTERNALS__ !== "undefined"
  );
}

/** convertFileSrc in Tauri; empty string in browser (cover fallback handles it). */
export function safeConvertFileSrc(filePath: string): string {
  if (!isTauriRuntime()) return "";
  return convertFileSrc(filePath);
}
