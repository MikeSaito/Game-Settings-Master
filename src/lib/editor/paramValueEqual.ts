/** Ini value comparison: exact, case, or equivalent floats (same as backend). */
export function paramValuesEqual(a: string, b: string): boolean {
  const ta = a.trim();
  const tb = b.trim();
  if (ta === tb) return true;
  if (ta.toLowerCase() === tb.toLowerCase()) return true;
  const fa = Number(ta);
  const fb = Number(tb);
  if (Number.isFinite(fa) && Number.isFinite(fb)) {
    return Math.abs(fa - fb) < 1e-4;
  }
  return false;
}
