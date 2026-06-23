import i18n from "@/i18n";
import type { GameParameter, GpuCapabilities } from "@/lib/core/types";

import { ENGINE } from "./constants";
import { normalizeDlssMode, syncFromDlssMode, syncFromDlssQualityNum } from "./dlssSync";
import { dlssIsOff, setInFile } from "./patchUtils";
import {
  syncMotionBlurQuality,
  syncResolutionScaleBounds,
  syncSsrQuality,
} from "./qualitySync";
import { reconcileAllParams } from "./reconcile";
import {
  syncFromAntiAliasing,
  syncFromTsrQuality,
  syncFromUpscalingMethod,
} from "./upscalingSync";
import type { ParamPatch } from "./types";

/** UI hint: parameter is synced from another. */
export function getDependencyLabel(key: string): string | null {
  const translated = i18n.t(`errors:paramSync.${key}`, { defaultValue: "" });
  return translated || null;
}

/**
 * Applies dependency cascade after a single parameter change.
 */
export function applyParamDependencies(
  params: GameParameter[],
  changed: ParamPatch,
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = params.map((p) =>
    p.key === changed.key && p.section === changed.section && p.file === changed.file
      ? { ...p, value: changed.value }
      : p,
  );

  const changedKey = changed.key;

  if (changedKey === "DLSSMode") {
    next = syncFromDlssMode(next, normalizeDlssMode(changed.value), gpu);
  } else if (changedKey === "DLSSQualityMode") {
    next = syncFromDlssQualityNum(next, changed.value, gpu);
  } else if (changedKey === "UpscalingMethod") {
    next = syncFromUpscalingMethod(next, changed.value, gpu);
  } else if (changedKey === "TSRQualityMode") {
    next = syncFromTsrQuality(next, changed.value, gpu);
  } else if (changedKey === "AntiAliasingType") {
    next = syncFromAntiAliasing(next, changed.value, gpu);
  } else if (changedKey === "ResolutionScaleMin" || changedKey === "ResolutionScaleMax") {
    next = syncResolutionScaleBounds(next);
  } else if (changedKey === "r.ScreenSpaceReflections") {
    next = syncSsrQuality(next, changed.value);
  } else if (changedKey === "r.DefaultFeature.MotionBlur") {
    next = syncMotionBlurQuality(next, changed.value);
  } else if (changedKey === "EnableMotionBlur") {
    if (changed.value === "Off") {
      next = setInFile(next, ENGINE, "r.DefaultFeature.MotionBlur", "False");
      next = syncMotionBlurQuality(next, "False");
    }
  }

  return reconcileAllParams(next, gpu);
}
