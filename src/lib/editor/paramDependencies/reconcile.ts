import type { GameParameter, GpuCapabilities } from "@/lib/core/types";

import { ENGINE, GUS } from "./constants";
import { normalizeDlssMode, syncFromDlssMode } from "./dlssSync";
import { dlssIsOff, findInFile, setInFile } from "./patchUtils";
import {
  syncMotionBlurQuality,
  syncResolutionScaleBounds,
  syncSsrQuality,
} from "./qualitySync";
import { syncFromTsrQuality, syncFromUpscalingMethod } from "./upscalingSync";

/** Full consistency pass before writing to ini. */
export function reconcileAllParams(
  params: GameParameter[],
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = [...params];

  const dlssMode = findInFile(next, GUS, "DLSSMode");
  const upscaling = findInFile(next, GUS, "UpscalingMethod");
  const tsr = findInFile(next, GUS, "TSRQualityMode");

  if (upscaling) {
    next = syncFromUpscalingMethod(next, upscaling.value, gpu);
  } else if (dlssMode && !dlssIsOff(dlssMode.value)) {
    next = syncFromDlssMode(next, normalizeDlssMode(dlssMode.value), gpu);
  } else if (tsr && tsr.value !== "0") {
    next = syncFromTsrQuality(next, tsr.value, gpu);
  } else if (dlssMode) {
    next = syncFromDlssMode(next, normalizeDlssMode(dlssMode.value), gpu);
  }

  next = syncResolutionScaleBounds(next);

  const ssr = findInFile(next, ENGINE, "r.ScreenSpaceReflections");
  if (ssr) next = syncSsrQuality(next, ssr.value);

  const mb = findInFile(next, ENGINE, "r.DefaultFeature.MotionBlur");
  if (mb) next = syncMotionBlurQuality(next, mb.value);

  const fg = findInFile(next, GUS, "UpscalingFrameGeneration");
  const mode = findInFile(next, GUS, "DLSSMode");
  if (fg && mode) {
    const fgOn = fg.value === "1" || fg.value === "True";
    const dlssOn = !dlssIsOff(mode.value);
    const fgAllowed = gpu?.supports_dlss_fg ?? true;
    if (fgOn && (!dlssOn || !fgAllowed)) {
      next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
    }
  }

  return next;
}
