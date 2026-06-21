import type { GameParameter } from "./types";
import { filterSelectOptions } from "./gpuCompat";

const DLSS_MODES = ["Off", "Performance", "Balanced", "Quality", "UltraQuality", "DLAA"] as const;

const UPSCALING_METHODS = ["U_None", "U_DLSS", "U_FSR", "U_TSR"] as const;

const ANTI_ALIASING_TYPES = [
  "AAM_None",
  "AAM_FXAA",
  "AAM_TemporalAA",
  "AAM_TSR",
  "AAM_DLAA",
] as const;

const KNOWN_SELECT_KEYS: Record<string, readonly string[]> = {
  DLSSMode: DLSS_MODES,
  UpscalingMethod: UPSCALING_METHODS,
  AntiAliasingType: ANTI_ALIASING_TYPES,
};

/** Known enum lists for GUS keys; GPU filter may restrict further. */
export function getParamSelectOptions(
  param: GameParameter,
  gpu: Parameters<typeof filterSelectOptions>[1],
): string[] | undefined {
  const gpuFiltered = filterSelectOptions(param, gpu);
  if (gpuFiltered) return gpuFiltered;

  const base = KNOWN_SELECT_KEYS[param.key];
  if (!base) return undefined;

  const current = param.value.trim();
  if (current && !base.some((option) => option === current)) {
    return [...base, current];
  }
  return [...base];
}
