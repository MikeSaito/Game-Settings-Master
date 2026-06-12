import type { GameParameter, GpuCapabilities } from "./types";

export type ParamPatch = Pick<GameParameter, "key" | "section" | "file" | "value">;

const DLSS_MODE_TO_NUM: Record<string, string> = {
  Off: "0",
  Performance: "1",
  Balanced: "2",
  Quality: "3",
  UltraQuality: "4",
  DLAA: "5",
};

const DLSS_NUM_TO_MODE: Record<string, string> = {
  "0": "Off",
  "1": "Performance",
  "2": "Balanced",
  "3": "Quality",
  "4": "UltraQuality",
  "5": "DLAA",
};

const DLSS_MODE_TO_SCALE: Record<string, string> = {
  Off: "1.0",
  Performance: "0.5",
  Balanced: "0.58",
  Quality: "0.66",
  UltraQuality: "0.77",
  DLAA: "1.0",
};

function findInFile(
  params: GameParameter[],
  file: string,
  key: string,
): GameParameter | undefined {
  return params.find((p) => p.file === file && p.key === key);
}

function setInFile(
  params: GameParameter[],
  file: string,
  key: string,
  value: string,
): GameParameter[] {
  return params.map((p) =>
    p.file === file && p.key === key ? { ...p, value } : p,
  );
}

const GUS = "GameUserSettings.ini";
const ENGINE = "Engine.ini";

function normalizeDlssMode(value: string): string {
  const v = value.trim();
  if (DLSS_MODE_TO_NUM[v] != null) return v;
  const lower = v.toLowerCase();
  for (const mode of Object.keys(DLSS_MODE_TO_NUM)) {
    if (mode.toLowerCase() === lower) return mode;
  }
  return v;
}

function dlssIsOff(mode: string): boolean {
  const m = normalizeDlssMode(mode);
  return m === "Off" || m === "0";
}

/** Подпись для UI: параметр подстраивается под другой. */
export function getDependencyLabel(key: string): string | null {
  const labels: Record<string, string> = {
    DLSSQualityMode: "Связан с DLSSMode",
    ResolutionScaleDLSS: "Связан с режимом DLSS",
    UpscalingMethod: "Связан с DLSS / TSR / FSR",
    TSRQualityMode: "Связан с методом upscaling",
    AntiAliasingType: "Связан с DLSS / TSR",
    UpscalingFrameGeneration: "Только при включённом DLSS (RTX 40+)",
    ResolutionScaleMin: "Не больше ResolutionScaleMax",
    ResolutionScaleMax: "Не меньше ResolutionScaleMin",
    "r.SSR.Quality": "Работает при включённых отражениях на воде/полу",
    "r.MotionBlurQuality": "Работает при включённом motion blur",
  };
  return labels[key] ?? null;
}

/**
 * Применяет каскад зависимостей после изменения одного параметра.
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

/** Полная согласованность перед записью в ini. */
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

function syncFromDlssMode(
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

function syncFromDlssQualityNum(
  params: GameParameter[],
  num: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  const mode = DLSS_NUM_TO_MODE[num.trim()] ?? "Off";
  let next = setInFile(params, GUS, "DLSSMode", mode);
  return syncFromDlssMode(next, mode, gpu);
}

function syncFromUpscalingMethod(
  params: GameParameter[],
  method: string,
  gpu?: GpuCapabilities,
): GameParameter[] {
  let next = params;
  const m = method.trim();

  if (m === "U_DLSS" || m.includes("DLSS")) {
    const mode = findInFile(next, GUS, "DLSSMode");
    next = setInFile(next, GUS, "TSRQualityMode", "0");
    // Игра без отдельного DLSSMode — синхронизировать каскадом нечего.
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

function syncFromTsrQuality(
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

function syncFromAntiAliasing(
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

function syncResolutionScaleBounds(params: GameParameter[]): GameParameter[] {
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

function syncSsrQuality(params: GameParameter[], ssrValue: string): GameParameter[] {
  const off = ssrValue === "0" || ssrValue === "False";
  if (!off) return params;
  const q = findInFile(params, ENGINE, "r.SSR.Quality");
  if (q && q.value !== "0") {
    return setInFile(params, ENGINE, "r.SSR.Quality", "0");
  }
  return params;
}

function syncMotionBlurQuality(params: GameParameter[], mbValue: string): GameParameter[] {
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
