import type { GameParameter, GpuCapabilities } from "@/lib/core/types";

import { GUS } from "./constants";
import { dlssIsOff, findInFile, normalizeDlssMode, setInFile } from "./patchUtils";
import { syncFromDlssMode } from "./dlssSync";

export function syncFromUpscalingMethod(
  params: GameParameter[],
  method: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = params;
  const m = method.trim();

  if (m === "U_DLSS" || m.includes("DLSS")) {
    const mode = findInFile(next, GUS, "DLSSMode");
    next = setInFile(next, GUS, "TSRQualityMode", "0");
    if (!mode) {
      return next;
    }
    if (dlssIsOff(mode.value)) {
      next = setInFile(next, GUS, "DLSSMode", "Quality");
    }
    const current = findInFile(next, GUS, "DLSSMode")?.value ?? "Quality";
    next = syncFromDlssMode(next, normalizeDlssMode(current), gpu);
    return next;
  }

  if (m === "U_TSR" || m.includes("TSR")) {
    next = setInFile(next, GUS, "DLSSMode", "Off");
    next = setInFile(next, GUS, "DLSSQualityMode", "0");
    next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
    const tsr = findInFile(next, GUS, "TSRQualityMode");
    if (tsr && (tsr.value === "0" || tsr.value === "-1")) {
      next = setInFile(next, GUS, "TSRQualityMode", "2");
    }
    const aaParam = findInFile(next, GUS, "AntiAliasingType");
    if (aaParam) {
      if (
        aaParam.value === "AAM_DLAA" ||
        aaParam.value === "AAM_None" ||
        aaParam.value === "AAM_FXAA"
      ) {
        next = setInFile(next, GUS, "AntiAliasingType", "AAM_TSR");
      }
    }
    return next;
  }

  if (m === "U_FSR" || m.includes("FSR")) {
    next = setInFile(next, GUS, "DLSSMode", "Off");
    next = setInFile(next, GUS, "DLSSQualityMode", "0");
    next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
    next = setInFile(next, GUS, "TSRQualityMode", "0");
    return next;
  }

  if (m === "U_None" || m === "None") {
    next = setInFile(next, GUS, "DLSSMode", "Off");
    next = setInFile(next, GUS, "DLSSQualityMode", "0");
    next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
    next = setInFile(next, GUS, "TSRQualityMode", "0");
  }

  return next;
}

export function syncFromTsrQuality(
  params: GameParameter[],
  value: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  const n = Number(value);
  if (!Number.isFinite(n) || n <= 0) {
    return params;
  }
  let next = setInFile(params, GUS, "UpscalingMethod", "U_TSR");
  next = setInFile(next, GUS, "DLSSMode", "Off");
  next = setInFile(next, GUS, "DLSSQualityMode", "0");
  next = setInFile(next, GUS, "UpscalingFrameGeneration", "0");
  const aaParam = findInFile(next, GUS, "AntiAliasingType");
  if (aaParam && aaParam.value !== "AAM_TSR" && aaParam.value !== "AAM_TemporalAA") {
    next = setInFile(next, GUS, "AntiAliasingType", "AAM_TSR");
  }
  return syncFromUpscalingMethod(next, "U_TSR", gpu);
}

export function syncFromAntiAliasing(
  params: GameParameter[],
  value: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = params;
  if (value.includes("DLAA")) {
    next = setInFile(next, GUS, "DLSSMode", "DLAA");
    return syncFromDlssMode(next, "DLAA", gpu);
  }
  if (value.includes("TSR")) {
    next = setInFile(next, GUS, "UpscalingMethod", "U_TSR");
    const tsr = findInFile(next, GUS, "TSRQualityMode");
    if (tsr && Number(tsr.value) <= 0) {
      next = setInFile(next, GUS, "TSRQualityMode", "2");
    }
    return syncFromUpscalingMethod(next, "U_TSR", gpu);
  }
  return next;
}
