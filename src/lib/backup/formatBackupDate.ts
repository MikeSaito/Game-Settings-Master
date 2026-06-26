/** `YYYYMMDD_HHMMSS` backup folder id → display label. */
export function formatBackupDate(id: string): string {
  const match = /^(\d{4})(\d{2})(\d{2})_(\d{2})(\d{2})(\d{2})$/.exec(id);
  if (!match) return id;
  const [, y, mo, d, h, mi, s] = match;
  return `${d}.${mo}.${y} · ${h}:${mi}:${s}`;
}
