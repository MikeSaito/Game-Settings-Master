import type { GameParameter, GpuCapabilities } from "@/lib/core/types";
import type { EditorPanel } from "@/lib/routing";
import { shouldIncludeInApply } from "../engineParams";
import { EMPTY_INI_SNAPSHOT } from "../iniSnapshot";
import type { ValidationIssue } from "./types";

const ENGINE_SCALABILITY = new Set(["Engine.ini", "Scalability.ini"]);

const RT_CVAR_KEYS = [
  "r.raytracing",
  "r.raytracing.enabled",
  "r.lumen.hardwareraytracing",
] as const;

function isTruthyCvar(value: string): boolean {
  const v = value.trim().toLowerCase();
  return v === "1" || v === "true" || v === "yes" || v === "on";
}

function isLowQuality(value: string, max = 4): boolean {
  const n = Number(value.trim());
  return Number.isFinite(n) && n <= Math.max(0, Math.floor(max / 2) - 1);
}

function collectPendingValues(
  files: Record<string, Record<string, Record<string, string>>>,
): Map<string, string> {
  const map = new Map<string, string>();
  for (const sections of Object.values(files)) {
    for (const entries of Object.values(sections)) {
      for (const [key, value] of Object.entries(entries)) {
        map.set(key.toLowerCase(), value);
      }
    }
  }
  return map;
}

function removedKeysInFile(
  removals: Record<string, Record<string, string[]>> | undefined,
  file: string,
): Set<string> {
  const keys = new Set<string>();
  const sections = removals?.[file];
  if (!sections) return keys;
  for (const entries of Object.values(sections)) {
    for (const key of entries) keys.add(key.toLowerCase());
  }
  return keys;
}

function effectiveKeysInFile(ctx: ComboRuleContext, file: string): Set<string> {
  const removed = removedKeysInFile(ctx.removals, file);
  const keys = new Set(
    ctx.params
      .filter((param) => param.file === file && param.present_in_ini)
      .map((param) => param.key.toLowerCase())
      .filter((key) => !removed.has(key)),
  );
  const sections = ctx.files[file];
  if (sections) {
    for (const entries of Object.values(sections)) {
      for (const key of Object.keys(entries)) keys.add(key.toLowerCase());
    }
  }
  return keys;
}

function pendingKeysInFile(ctx: ComboRuleContext, file: string): Set<string> {
  const sections = ctx.files[file];
  if (!sections) return new Set();
  return new Set(
    Object.values(sections).flatMap((entries) =>
      Object.keys(entries).map((key) => key.toLowerCase()),
    ),
  );
}

export interface ComboRuleContext {
  panel: EditorPanel;
  params: GameParameter[];
  gpu: GpuCapabilities | undefined;
  gpuPending?: boolean;
  gpuUnavailable?: boolean;
  engineEnabled: Set<string>;
  shippedIniKeys?: ReadonlySet<string>;
  files: Record<string, Record<string, Record<string, string>>>;
  removals?: Record<string, Record<string, string[]>>;
}

/** Value that will be active after apply — respects engine ini toggles. */
function activeApplyValue(
  ctx: ComboRuleContext,
  key: string,
  pendingValues: Map<string, string>,
): string | null {
  const lower = key.toLowerCase();
  const pending = pendingValues.get(lower);
  if (pending != null) return pending;

  for (const param of ctx.params) {
    if (param.key.toLowerCase() !== lower) continue;
    if (removedKeysInFile(ctx.removals, param.file).has(lower)) continue;
    if (!shouldIncludeInApply(param, ctx.engineEnabled, ctx.shippedIniKeys ?? EMPTY_INI_SNAPSHOT)) continue;
    const value = param.value.trim();
    if (!value) continue;
    return value;
  }
  return null;
}

function engineCvarApplyValue(
  ctx: ComboRuleContext,
  key: string,
  pendingValues: Map<string, string>,
): string | null {
  const lower = key.toLowerCase();
  const pending = pendingValues.get(lower);
  if (pending != null) return pending;
  return activeApplyValue(ctx, key, pendingValues);
}

function engineCvarActiveInApply(
  ctx: ComboRuleContext,
  key: string,
  pendingValues: Map<string, string>,
): boolean {
  const value = engineCvarApplyValue(ctx, key, pendingValues);
  return value != null && isTruthyCvar(value);
}

export function evaluateComboRules(ctx: ComboRuleContext): ValidationIssue[] {
  const issues: ValidationIssue[] = [];
  const pendingValues = collectPendingValues(ctx.files);
  const engineKeys = effectiveKeysInFile(ctx, "Engine.ini");
  const scalabilityKeys = effectiveKeysInFile(ctx, "Scalability.ini");
  const duplicateCandidates = new Set([
    ...pendingKeysInFile(ctx, "Engine.ini"),
    ...pendingKeysInFile(ctx, "Scalability.ini"),
  ]);

  for (const key of duplicateCandidates) {
    if (engineKeys.has(key) && scalabilityKeys.has(key)) {
      issues.push({
        code: "combo_engine_scalability_dup",
        severity: "warning",
        key,
        i18nKey: "validation.combo.engineScalabilityDup",
        i18nParams: { key },
      });
    }
  }

  const rtOn = RT_CVAR_KEYS.some((key) =>
    engineCvarActiveInApply(ctx, key, pendingValues),
  );
  const rtChanged = RT_CVAR_KEYS.some((key) => pendingValues.has(key));
  const shadowsChanged = pendingValues.has("sg.shadowquality");

  if (rtOn && (rtChanged || shadowsChanged)) {
    const shadowVal = activeApplyValue(ctx, "sg.ShadowQuality", pendingValues);
    if (shadowVal && isLowQuality(shadowVal, 4)) {
      issues.push({
        code: "combo_rt_shadows",
        severity: "warning",
        key: "sg.ShadowQuality",
        i18nKey: "validation.combo.rtLowShadows",
        i18nParams: { shadows: shadowVal },
      });
    }

    if (rtChanged && ctx.gpuPending) {
      issues.push({
        code: "combo_gpu_pending",
        severity: "error",
        i18nKey: "validation.gpuPending",
      });
    } else if (rtChanged && ctx.gpuUnavailable) {
      issues.push({
        code: "combo_rt_gpu_unknown",
        severity: "warning",
        i18nKey: "validation.combo.rtGpuUnknown",
      });
    } else if (rtChanged && ctx.gpu && !ctx.gpu.supports_ray_tracing) {
      issues.push({
        code: "combo_rt_no_hw",
        severity: "warning",
        i18nKey: "validation.combo.rtNoHardware",
      });
    }
  }

  const textureVal = activeApplyValue(ctx, "sg.TextureQuality", pendingValues);
  const poolVal = engineCvarApplyValue(ctx, "r.Streaming.PoolSize", pendingValues);
  if (
    (pendingValues.has("sg.texturequality") || pendingValues.has("r.streaming.poolsize")) &&
    textureVal &&
    isLowQuality(textureVal, 4) &&
    poolVal &&
    Number(poolVal) > 3000
  ) {
    issues.push({
      code: "combo_streaming_texture",
      severity: "warning",
      key: "r.Streaming.PoolSize",
      i18nKey: "validation.combo.streamingTexture",
      i18nParams: { pool: poolVal, texture: textureVal },
    });
  }

  return issues;
}

export function comboIssuesForKey(
  issues: ValidationIssue[],
  paramKey: string,
): ValidationIssue[] {
  const lower = paramKey.toLowerCase();
  return issues.filter((issue) => {
    if (issue.key?.toLowerCase() === lower) return true;
    if (issue.code === "combo_rt_shadows" && lower === "sg.shadowquality") return true;
    if (issue.code === "combo_rt_no_hw" && rtKeysForInline.has(lower)) return true;
    return false;
  });
}

const rtKeysForInline = new Set([
  ...RT_CVAR_KEYS,
  "sg.shadowquality",
]);

export { ENGINE_SCALABILITY };
