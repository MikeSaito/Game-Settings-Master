import { isTauriRuntime } from "./tauriRuntime";

type OpenDialogOptions = {
  directory?: boolean;
  multiple?: boolean;
  title?: string;
  filters?: { name: string; extensions: string[] }[];
};

/** Native file/folder picker in Tauri; null in browser or when cancelled. */
export async function openPathDialog(
  options: OpenDialogOptions,
): Promise<string | null> {
  if (!isTauriRuntime()) return null;
  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open(options);
  if (!selected) return null;
  return typeof selected === "string" ? selected : selected;
}
