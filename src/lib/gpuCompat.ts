import i18n from "../i18n";
import type { GpuCapabilities, GameParameter } from "./types";

/** Parameters fully hidden without DLSS/RTX support. */
const NVIDIA_ONLY_KEYS = new Set([
  "DLSSMode",
  "DLSSQualityMode",
  "ResolutionScaleDLSS",
  "UpscalingFrameGeneration",
  "r.RayTracing",
  "r.RayTracing.Shadows",
]);

export function isParamVisible(param: GameParameter, gpu: GpuCapabilities | undefined): boolean {
  if (!gpu) return true;
  if (NVIDIA_ONLY_KEYS.has(param.key)) {
    if (param.key === "UpscalingFrameGeneration") {
      return gpu.supports_dlss_fg;
    }
    if (param.key === "r.RayTracing" || param.key === "r.RayTracing.Shadows") {
      return gpu.supports_ray_tracing;
    }
    return gpu.supports_dlss;
  }
  return true;
}

export function filterSelectOptions(
  param: GameParameter,
  gpu: GpuCapabilities | undefined,
): string[] | null {
  if (!gpu?.supports_dlss && param.key === "AntiAliasingType") {
    return ["AAM_None", "AAM_FXAA", "AAM_TemporalAA", "AAM_TSR"];
  }
  if (!gpu?.supports_dlss && param.key === "UpscalingMethod") {
    return ["U_None", "U_FSR", "U_TSR"];
  }
  return null;
}

export function gpuSummaryLabel(gpu: GpuCapabilities): string {
  const parts: string[] = [gpu.name];
  if (gpu.vendor === "nvidia") {
    if (gpu.supports_dlss_fg) parts.push(i18n.t("common:gpuDlssFg"));
    else if (gpu.supports_dlss) parts.push(i18n.t("common:gpuDlss"));
    else parts.push(i18n.t("common:gpuNoRtx"));
  } else if (gpu.vendor === "amd") {
    parts.push(i18n.t("common:gpuFsrTsr"));
  }
  return parts.join(" · ");
}

export function gpuFilterHint(gpu: GpuCapabilities): string | null {
  if (gpu.vendor === "amd") {
    return i18n.t("common:gpuFilterAmd", { name: gpu.name });
  }
  if (gpu.vendor === "intel") {
    return i18n.t("common:gpuFilterIntel", { name: gpu.name });
  }
  if (gpu.vendor === "nvidia" && !gpu.supports_dlss) {
    return i18n.t("common:gpuFilterNvidiaNoDlss", { name: gpu.name });
  }
  if (gpu.vendor === "nvidia" && gpu.supports_dlss && !gpu.supports_dlss_fg) {
    return i18n.t("common:gpuFilterNvidiaDlssOnly", { name: gpu.name });
  }
  return null;
}
