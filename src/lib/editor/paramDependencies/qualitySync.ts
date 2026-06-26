import type { GameParameter } from "@/lib/core/types";

import { ENGINE, GUS } from "./constants";
import { findInFile, setInFile } from "./patchUtils";

export function syncResolutionScaleBounds(params: GameParameter[]): GameParameter[] {
  const minP = findInFile(params, GUS, "ResolutionScaleMin");
  const maxP = findInFile(params, GUS, "ResolutionScaleMax");
  if (!minP || !maxP) return params;

  let min = Number(minP.value);
  let max = Number(maxP.value);
  if (!Number.isFinite(min) || !Number.isFinite(max)) return params;

  let next = params;
  if (min > max) {
    next = setInFile(next, GUS, "ResolutionScaleMin", String(max));
  }
  return next;
}

export function syncSsrQuality(params: GameParameter[], ssrValue: string): GameParameter[] {
  const off = ssrValue === "0" || ssrValue === "False";
  if (!off) return params;
  const q = findInFile(params, ENGINE, "r.SSR.Quality");
  if (q && q.value !== "0") {
    return setInFile(params, ENGINE, "r.SSR.Quality", "0");
  }
  return params;
}

export function syncMotionBlurQuality(params: GameParameter[], mbValue: string): GameParameter[] {
  const off = mbValue === "False" || mbValue === "0" || mbValue === "Off";
  if (!off) return params;
  const q = findInFile(params, ENGINE, "r.MotionBlurQuality");
  if (q && q.value !== "0") {
    return setInFile(params, ENGINE, "r.MotionBlurQuality", "0");
  }
  const scale = findInFile(params, ENGINE, "r.MotionBlur.Scale");
  if (scale && scale.value !== "0") {
    return setInFile(params, ENGINE, "r.MotionBlur.Scale", "0");
  }
  return params;
}
