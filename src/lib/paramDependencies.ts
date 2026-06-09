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

function findParam(
  params: GameParameter[],
  key: string,
): GameParameter | undefined {
  return params.find((p) => p.key === key);
}

function setValue(
  params: GameParameter[],
  key: string,
  value: string,
): GameParameter[] {
  return params.map((p) => (p.key === key ? { ...p, value } : p));
}

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
      next = setValue(next, "r.DefaultFeature.MotionBlur", "False");
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

  const dlssMode = findParam(next, "DLSSMode");
  const upscaling = findParam(next, "UpscalingMethod");
  const tsr = findParam(next, "TSRQualityMode");

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

  const ssr = findParam(next, "r.ScreenSpaceReflections");
  if (ssr) next = syncSsrQuality(next, ssr.value);

  const mb = findParam(next, "r.DefaultFeature.MotionBlur");
  if (mb) next = syncMotionBlurQuality(next, mb.value);

  const fg = findParam(next, "UpscalingFrameGeneration");
  const mode = findParam(next, "DLSSMode");
  if (fg && mode) {
    const fgOn = fg.value === "1" || fg.value === "True";
    const dlssOn = !dlssIsOff(mode.value);
    const fgAllowed = gpu?.supports_dlss_fg ?? true;
    if (fgOn && (!dlssOn || !fgAllowed)) {
      next = setValue(next, "UpscalingFrameGeneration", "0");
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

  next = setValue(next, "DLSSQualityMode", num);

  if (dlssIsOff(mode)) {
    next = setValue(next, "UpscalingMethod", "U_None");
    next = setValue(next, "UpscalingFrameGeneration", "0");
    if (findParam(next, "TSRQualityMode") && findParam(next, "TSRQualityMode")!.value === "0") {
      // оставить TSR как есть, если пользователь выбрал TSR отдельно
    }
  } else {
    if (gpu?.supports_dlss !== false) {
      next = setValue(next, "UpscalingMethod", "U_DLSS");
    }
    next = setValue(next, "ResolutionScaleDLSS", scale);
    next = setValue(next, "TSRQualityMode", "0");
    if (mode === "DLAA") {
      next = setValue(next, "AntiAliasingType", "AAM_DLAA");
    } else if (findParam(next, "AntiAliasingType")?.value === "AAM_DLAA") {
      next = setValue(next, "AntiAliasingType", "AAM_TemporalAA");
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
  let next = setValue(params, "DLSSMode", mode);
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
    const mode = findParam(next, "DLSSMode");
    if (!mode || dlssIsOff(mode.value)) {
      next = setValue(next, "DLSSMode", "Quality");
    }
    next = setValue(next, "TSRQualityMode", "0");
    const current = findParam(next, "DLSSMode")!.value;
    next = syncFromDlssMode(next, normalizeDlssMode(current), gpu);
    return next;
  }

  if (m === "U_TSR" || m.includes("TSR")) {
    next = setValue(next, "DLSSMode", "Off");
    next = setValue(next, "DLSSQualityMode", "0");
    next = setValue(next, "UpscalingFrameGeneration", "0");
    const tsr = findParam(next, "TSRQualityMode");
    if (tsr && (tsr.value === "0" || tsr.value === "-1")) {
      next = setValue(next, "TSRQualityMode", "2");
    }
    if (findParam(next, "AntiAliasingType")?.value === "AAM_DLAA") {
      next = setValue(next, "AntiAliasingType", "AAM_TSR");
    } else if (findParam(next, "AntiAliasingType")) {
      const aa = findParam(next, "AntiAliasingType")!.value;
      if (aa === "AAM_None" || aa === "AAM_FXAA") {
        next = setValue(next, "AntiAliasingType", "AAM_TSR");
      }
    }
    return next;
  }

  if (m === "U_FSR" || m.includes("FSR")) {
    next = setValue(next, "DLSSMode", "Off");
    next = setValue(next, "DLSSQualityMode", "0");
    next = setValue(next, "UpscalingFrameGeneration", "0");
    next = setValue(next, "TSRQualityMode", "0");
    return next;
  }

  if (m === "U_None" || m === "None") {
    next = setValue(next, "DLSSMode", "Off");
    next = setValue(next, "DLSSQualityMode", "0");
    next = setValue(next, "UpscalingFrameGeneration", "0");
    next = setValue(next, "TSRQualityMode", "0");
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
  let next = setValue(params, "UpscalingMethod", "U_TSR");
  next = setValue(next, "DLSSMode", "Off");
  next = setValue(next, "DLSSQualityMode", "0");
  next = setValue(next, "UpscalingFrameGeneration", "0");
  if (findParam(next, "AntiAliasingType")) {
    const aa = findParam(next, "AntiAliasingType")!.value;
    if (aa !== "AAM_TSR" && aa !== "AAM_TemporalAA") {
      next = setValue(next, "AntiAliasingType", "AAM_TSR");
    }
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
    next = setValue(next, "DLSSMode", "DLAA");
    return syncFromDlssMode(next, "DLAA", gpu);
  }
  if (value.includes("TSR")) {
    next = setValue(next, "UpscalingMethod", "U_TSR");
    const tsr = findParam(next, "TSRQualityMode");
    if (tsr && Number(tsr.value) <= 0) {
      next = setValue(next, "TSRQualityMode", "2");
    }
    return syncFromUpscalingMethod(next, "U_TSR", gpu);
  }
  return next;
}

function syncResolutionScaleBounds(params: GameParameter[]): GameParameter[] {
  const minP = findParam(params, "ResolutionScaleMin");
  const maxP = findParam(params, "ResolutionScaleMax");
  if (!minP || !maxP) return params;

  let min = Number(minP.value);
  let max = Number(maxP.value);
  if (!Number.isFinite(min) || !Number.isFinite(max)) return params;

  let next = params;
  if (min > max) {
    next = setValue(next, "ResolutionScaleMin", String(max));
  }
  return next;
}

function syncSsrQuality(params: GameParameter[], ssrValue: string): GameParameter[] {
  const off = ssrValue === "0" || ssrValue === "False";
  if (!off) return params;
  const q = findParam(params, "r.SSR.Quality");
  if (q && q.value !== "0") {
    return setValue(params, "r.SSR.Quality", "0");
  }
  return params;
}

function syncMotionBlurQuality(params: GameParameter[], mbValue: string): GameParameter[] {
  const off = mbValue === "False" || mbValue === "0" || mbValue === "Off";
  if (!off) return params;
  const q = findParam(params, "r.MotionBlurQuality");
  if (q && q.value !== "0") {
    return setValue(params, "r.MotionBlurQuality", "0");
  }
  const scale = findParam(params, "r.MotionBlur.Scale");
  if (scale && scale.value !== "0") {
    return setValue(params, "r.MotionBlur.Scale", "0");
  }
  return params;
}
