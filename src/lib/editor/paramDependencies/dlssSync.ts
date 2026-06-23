import type { GameParameter, GpuCapabilities } from "@/lib/core/types";

import {
  DLSS_MODE_TO_NUM,
  DLSS_MODE_TO_SCALE,
  DLSS_NUM_TO_MODE,
  GUS,
} from "./constants";
import { dlssIsOff, findInFile, normalizeDlssMode, setInFile } from "./patchUtils";

export function syncFromDlssMode(
  params: GameParameter[],
  mode: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = params;
  const num = DLSS_MODE_TO_NUM[mode] ?? "0";
  const scale = DLSS_MODE_TO_SCALE[mode] ?? "1.0";

  next = setInFile(next, GUS, "DLSSQualityMode", num);

  if (dlssIsOff(mode)) {
    next = setInFile(next, GUS, "UpscalingMethod", "U_None");
    next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
  } else {
    if (gpu?.supports_dlss !== false) {
      next = setInFile(next, GUS, "UpscalingMethod", "U_DLSS");
    }
    next = setInFile(next, GUS, "ResolutionScaleDLSS", scale);
    next = setInFile(next, GUS, "TSRQualityMode", "0");
    if (mode === "DLAA") {
      next = setInFile(next, GUS, "AntiAliasingType", "AAM_DLAA");
    } else if (findInFile(next, GUS, "AntiAliasingType")?.value === "AAM_DLAA") {
      next = setInFile(next, GUS, "AntiAliasingType", "AAM_TemporalAA");
    }
  }

  return next;
}

export function syncFromDlssQualityNum(
  params: GameParameter[],
  num: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  const mode = DLSS_NUM_TO_MODE[num.trim()] ?? "Off";
  let next = setInFile(params, GUS, "DLSSMode", mode);
  return syncFromDlssMode(next, mode, gpu);
}

export { normalizeDlssMode } from "./patchUtils";