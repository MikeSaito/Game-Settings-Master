import type { AppInvokeError } from "@/lib/api/bindings";

/** Kept in sync with `app_error::RUNNING_GAME_ERROR_MARKER` in Rust. */
export const RUNNING_GAME_ERROR_MARKER = "GSM_ERR_GAME_RUNNING:";

function rawInvokeMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  if (err && typeof err === "object" && "message" in err) {
    const msg = (err as { message: unknown }).message;
    if (typeof msg === "string") return msg;
  }
  return String(err);
}

function tryParseStructuredError(raw: string): AppInvokeError | null {
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) return null;
  try {
    const parsed = JSON.parse(trimmed) as unknown;
    if (
      parsed &&
      typeof parsed === "object" &&
      "code" in parsed &&
      "message" in parsed &&
      typeof (parsed as AppInvokeError).code === "string" &&
      typeof (parsed as AppInvokeError).message === "string"
    ) {
      return parsed as AppInvokeError;
    }
  } catch {
    // not JSON
  }
  return null;
}

/** Structured invoke error from Tauri (`AppInvokeError`), if present. */
export function parseInvokeError(err: unknown): AppInvokeError | null {
  if (err && typeof err === "object" && "code" in err && "message" in err) {
    const candidate = err as AppInvokeError;
    if (typeof candidate.code === "string" && typeof candidate.message === "string") {
      return candidate;
    }
  }
  return tryParseStructuredError(rawInvokeMessage(err));
}

export function isRunningGameError(err: unknown): boolean {
  const structured = parseInvokeError(err);
  if (structured?.code === "game_running") return true;
  return rawInvokeMessage(err).includes(RUNNING_GAME_ERROR_MARKER);
}

/** Error text from invoke(Tauri) or fetch */
export function formatInvokeError(err: unknown): string {
  const structured = parseInvokeError(err);
  if (structured) return structured.message;

  const raw = rawInvokeMessage(err);
  if (raw.includes(RUNNING_GAME_ERROR_MARKER)) {
    return raw.replace(RUNNING_GAME_ERROR_MARKER, "");
  }
  return raw;
}
