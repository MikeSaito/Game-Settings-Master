import { handleE2eInvoke } from "./mockInvoke";

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return handleE2eInvoke(cmd, args) as T;
}

export function convertFileSrc(path: string): string {
  return path;
}
