import type { GameParameter, GameProfile, GpuCapabilities, ScalabilityLimits } from "@/lib/core/types";
import type { EditorPanel } from "@/lib/routing";
import { initialEngineEnabledKeys } from "../engineParams";
import { EMPTY_INI_SNAPSHOT, iniSnapshotKeyFromParts } from "../iniSnapshot";
import { buildCustomChanges } from "../buildCustomChanges";
import {
  analyzeSgEngineConflictGroups,
  collectPendingKeys,
} from "../sgEngineConflicts";
import { evaluateComboRules } from "./comboRules";
import type { ValidationIssue } from "./types";
import { keyAppliesToGame } from "./ueVersion";

/** Matches `UE_DEFAULT_SCALABILITY_MAX` in src-tauri/src/scalability/constants.rs */
export const UE_DEFAULT_SCALABILITY_MAX = 4;

export interface ValidateApplyContext {
  game: Pick<GameProfile, "engine_family" | "engine_version">;
  panel: EditorPanel;
  params: GameParameter[];
  parameters: GameParameter[];
  gpu: GpuCapabilities | undefined;
  engineEnabled: Set<string>;
  limits: ScalabilityLimits | undefined;
  /** True while scalability limits query has not resolved yet. */
  limitsPending?: boolean;
  /** True while GPU capabilities query has not resolved yet. */
  gpuPending?: boolean;
  /** True when GPU query finished without data (e.g. invoke failure). */
  gpuUnavailable?: boolean;
  editableCategories: Set<string>;
  /** Keys present in ini on first load of this game session — not removable. */
  shippedIniKeys?: ReadonlySet<string>;
}

function pendingEntries(
  files: Record<string, Record<string, Record<string, string>>>,
): Array<{ key: string; value: string; file: string }> {
  const rows: Array<{ key: string; value: string; file: string }> = [];
  for (const [file, sections] of Object.entries(files)) {
    for (const entries of Object.values(sections)) {
      for (const [key, value] of Object.entries(entries)) {
        rows.push({ key, value, file });
      }
    }
  }
  return rows;
}

function paramForKey(
  params: GameParameter[],
  key: string,
): GameParameter | undefined {
  const lower = key.toLowerCase();
  return params.find((p) => p.key.toLowerCase() === lower);
}

function checkValueRange(param: GameParameter | undefined, value: string): ValidationIssue | null {
  if (!param?.min && !param?.max) return null;
  const num = Number(value.trim());
  if (!Number.isFinite(num)) return null;
  const min = param.min != null ? Number(param.min) : NaN;
  const max = param.max != null ? Number(param.max) : NaN;
  if (Number.isFinite(min) && num < min) {
    return {
      code: "value_out_of_range",
      severity: "warning",
      key: param.key,
      i18nKey: "validation.valueBelowMin",
      i18nParams: { key: param.key, value, min: param.min ?? "" },
    };
  }
  if (Number.isFinite(max) && num > max) {
    return {
      code: "value_out_of_range",
      severity: "warning",
      key: param.key,
      i18nKey: "validation.valueAboveMax",
      i18nParams: { key: param.key, value, max: param.max ?? "" },
    };
  }
  return null;
}

function isSgQualityKey(key: string): boolean {
  const lower = key.toLowerCase();
  return lower.startsWith("sg.") && lower !== "sg.resolutionquality";
}

/** Mirrors `ScalabilityLimits.max_for` in src-tauri/src/scalability/types.rs */
export function sgMaxForKey(
  limits: ScalabilityLimits | undefined,
  key: string,
): number {
  if (!limits) return UE_DEFAULT_SCALABILITY_MAX;
  const groupMax = limits.groups[key];
  if (groupMax != null) return groupMax;
  return limits.global_max;
}

function checkSgLimit(
  ctx: Pick<ValidateApplyContext, "limits" | "limitsPending">,
  key: string,
  value: string,
  file: string,
): ValidationIssue | null {
  if (file !== "GameUserSettings.ini" || !isSgQualityKey(key)) return null;

  if (ctx.limitsPending) {
    return {
      code: "sg_limits_pending",
      severity: "error",
      key,
      i18nKey: "validation.limitsPending",
    };
  }

  const max = sgMaxForKey(ctx.limits, key);
  const n = Number(value.trim());
  if (Number.isFinite(n) && n > max) {
    return {
      code: "sg_exceeds_limit",
      severity: "error",
      key,
      i18nKey: "validation.sgExceedsLimit",
      i18nParams: { key, value, max },
    };
  }
  return null;
}

export function validationIssueSignature(issues: ValidationIssue[]): string {
  return issues
    .map(
      (issue) =>
        `${issue.code}|${issue.severity}|${issue.key ?? ""}|${issue.i18nKey}|${JSON.stringify(issue.i18nParams ?? {})}`,
    )
    .join("\n");
}

const ENGINE_WRITE_FILES = new Set(["Engine.ini", "Scalability.ini", "Game.ini"]);

function inferPanelFromFiles(
  files: Record<string, Record<string, Record<string, string>>>,
): EditorPanel {
  return Object.keys(files).some((file) => ENGINE_WRITE_FILES.has(file))
    ? "advanced"
    : "basic";
}

export interface ValidateIniChangesContext {
  game: Pick<GameProfile, "engine_family" | "engine_version">;
  panel: EditorPanel;
  params: GameParameter[];
  gpu: GpuCapabilities | undefined;
  limits: ScalabilityLimits | undefined;
  limitsPending?: boolean;
  gpuPending?: boolean;
  gpuUnavailable?: boolean;
  engineEnabled: Set<string>;
  /** Keys present in ini on first load of this game session — not removable. */
  shippedIniKeys?: ReadonlySet<string>;
}

function isProtectedShippedRemoval(file: string, key: string): boolean {
  if (file === "GameUserSettings.ini") return true;
  if (
    key.toLowerCase().startsWith("r.") &&
    ENGINE_WRITE_FILES.has(file)
  ) {
    return false;
  }
  return true;
}

function checkShippedRemovals(
  removals: Record<string, Record<string, string[]>> | undefined,
  shippedIniKeys: ReadonlySet<string>,
): ValidationIssue[] {
  if (!removals) return [];
  const issues: ValidationIssue[] = [];
  for (const [file, sections] of Object.entries(removals)) {
    for (const [section, keys] of Object.entries(sections)) {
      for (const key of keys) {
        const snap = iniSnapshotKeyFromParts(file, section, key);
        if (!shippedIniKeys.has(snap)) continue;
        if (!isProtectedShippedRemoval(file, key)) continue;
        issues.push({
          code: "shipped_key_removal",
          severity: "error",
          key,
          i18nKey: "validation.shippedKeyRemoval",
          i18nParams: { key },
        });
      }
    }
  }
  return issues;
}

export function validateIniChanges(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>> | undefined,
  ctx: ValidateIniChangesContext,
): ValidationIssue[] {
  const shippedIniKeys = ctx.shippedIniKeys ?? EMPTY_INI_SNAPSHOT;
  const pendingKeySet = collectPendingKeys(files, removals ?? {});
  const conflictGroups = analyzeSgEngineConflictGroups(
    ctx.params,
    pendingKeySet,
    ctx.engineEnabled,
    shippedIniKeys,
  );
  const hasPending =
    Object.keys(files).length > 0 || Object.keys(removals ?? {}).length > 0;

  if (!hasPending && conflictGroups.length === 0) {
    return [];
  }

  const issues: ValidationIssue[] = [];
  if (hasPending) {
    issues.push(...checkShippedRemovals(removals, shippedIniKeys));
  }

  for (const group of conflictGroups) {
    issues.push({
      code: "sg_r_conflict",
      severity: "warning",
      key: group.sgKey,
      i18nKey: "validation.sgRConflict",
      i18nParams: { sg: group.sgParam.key },
    });
  }

  if (!hasPending) {
    return dedupeIssues(issues);
  }

  for (const { key, value, file } of pendingEntries(files)) {
    if (!keyAppliesToGame(key, ctx.game.engine_family, ctx.game.engine_version)) {
      const meta = ctx.game.engine_version ?? ctx.game.engine_family ?? "?";
      issues.push({
        code: "version_mismatch",
        severity: "error",
        key,
        i18nKey: "validation.versionMismatch",
        i18nParams: { key, version: meta },
      });
      continue;
    }

    const param = paramForKey(ctx.params, key);
    const rangeIssue = checkValueRange(param, value);
    if (rangeIssue) issues.push(rangeIssue);

    const sgIssue = checkSgLimit(ctx, key, value, file);
    if (sgIssue) issues.push(sgIssue);
  }

  issues.push(
    ...evaluateComboRules({
      panel: ctx.panel,
      params: ctx.params,
      gpu: ctx.gpu,
      gpuPending: ctx.gpuPending,
      gpuUnavailable: ctx.gpuUnavailable,
      engineEnabled: ctx.engineEnabled,
      shippedIniKeys: ctx.shippedIniKeys,
      files,
    }),
  );

  return dedupeIssues(issues);
}

export type ValidateOverrideContext = Omit<
  ValidateIniChangesContext,
  "panel" | "engineEnabled"
> & {
  engineEnabled?: Set<string>;
};

export function validateOverridePlan(
  files: Record<string, Record<string, Record<string, string>>>,
  removals: Record<string, Record<string, string[]>> | undefined,
  ctx: ValidateOverrideContext,
): ValidationIssue[] {
  return validateIniChanges(files, removals, {
    ...ctx,
    panel: inferPanelFromFiles(files),
    engineEnabled: ctx.engineEnabled ?? initialEngineEnabledKeys(ctx.params),
  });
}

export type ValidateApplyPlanContext = Omit<ValidateApplyContext, "panel">;

export function mergePanelValidationIssues(ctx: ValidateApplyPlanContext): ValidationIssue[] {
  const basic = validateApplyPlan({ ...ctx, panel: "basic" });
  const advanced = validateApplyPlan({ ...ctx, panel: "advanced" });
  return dedupeIssues([...basic, ...advanced]);
}

export function validateApplyPlan(ctx: ValidateApplyContext): ValidationIssue[] {
  const shippedIniKeys = ctx.shippedIniKeys ?? EMPTY_INI_SNAPSHOT;
  const { files, removals } = buildCustomChanges(
    ctx.params,
    ctx.parameters,
    ctx.gpu,
    ctx.engineEnabled,
    ctx.editableCategories,
    ctx.panel,
    shippedIniKeys,
  );

  return validateIniChanges(files, removals, {
    game: ctx.game,
    panel: ctx.panel,
    params: ctx.params,
    gpu: ctx.gpu,
    limits: ctx.limits,
    limitsPending: ctx.limitsPending,
    gpuPending: ctx.gpuPending,
    gpuUnavailable: ctx.gpuUnavailable,
    engineEnabled: ctx.engineEnabled,
    shippedIniKeys,
  });
}

function dedupeIssues(issues: ValidationIssue[]): ValidationIssue[] {
  const seen = new Set<string>();
  const out: ValidationIssue[] = [];
  for (const issue of issues) {
    const id = `${issue.code}|${issue.key ?? ""}|${issue.i18nKey}`;
    if (seen.has(id)) continue;
    seen.add(id);
    out.push(issue);
  }
  return out;
}

export function hasBlockingErrors(issues: ValidationIssue[]): boolean {
  return issues.some((i) => i.severity === "error");
}

export function canApplyPlan(
  issues: ValidationIssue[],
  warningsAcknowledged: boolean,
): { allowed: boolean; blockingErrors: boolean; needsWarningAck: boolean } {
  const blockingErrors = hasBlockingErrors(issues);
  const hasWarnings = issues.some((i) => i.severity === "warning");
  const needsWarningAck = hasWarnings && !blockingErrors && !warningsAcknowledged;
  return {
    allowed: !blockingErrors && !needsWarningAck,
    blockingErrors,
    needsWarningAck,
  };
}

export function assertApplyPlanAllowed(
  issues: ValidationIssue[],
  warningsAcknowledged: boolean,
  t: (key: string, options?: Record<string, unknown>) => string,
): void {
  const gate = canApplyPlan(issues, warningsAcknowledged);
  if (gate.blockingErrors) {
    throw new Error(t("validation.applyBlocked"));
  }
  if (gate.needsWarningAck) {
    throw new Error(t("validation.confirmWarnings"));
  }
}

/** Preset import/save: block only hard errors; warnings are checked on apply. */
export function assertPresetStorable(
  issues: ValidationIssue[],
  t: (key: string, options?: Record<string, unknown>) => string,
): void {
  if (hasBlockingErrors(issues)) {
    throw new Error(t("validation.applyBlocked"));
  }
}

export function issuesByKey(issues: ValidationIssue[]): Map<string, ValidationIssue[]> {
  const map = new Map<string, ValidationIssue[]>();
  for (const issue of issues) {
    if (!issue.key) continue;
    const lower = issue.key.toLowerCase();
    const bucket = map.get(lower);
    if (bucket) bucket.push(issue);
    else map.set(lower, [issue]);
  }
  return map;
}

export function formatValidationIssue(
  t: (key: string, options?: Record<string, unknown>) => string,
  issue: ValidationIssue,
): string {
  return t(issue.i18nKey, issue.i18nParams ?? {});
}
