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

export function isRunningGameError(err: unknown): boolean {
  return rawInvokeMessage(err).includes(RUNNING_GAME_ERROR_MARKER);
}

/** Error text from invoke(Tauri) or fetch */
export function formatInvokeError(err: unknown): string {
  const raw = rawInvokeMessage(err);
  if (raw.includes(RUNNING_GAME_ERROR_MARKER)) {
    return raw.replace(RUNNING_GAME_ERROR_MARKER, "");
  }
  return raw;
}
