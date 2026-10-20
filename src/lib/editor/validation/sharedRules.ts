import policy from "@shared/validation-rules.json";
import type { ValidationIssueCode } from "./types";

export interface SharedContext {
  values: Record<string, string>;
  changedKeys: string[];
  rtStatus: string;
}
export interface SharedIssue { code: ValidationIssueCode; severity: "warning"; key: string | null }
const decimal = new RegExp(policy.decimalPattern);
export function finiteDecimal(raw: string): number | null {
  const value = raw.trim();
  if (!decimal.test(value)) return null;
  const number = Number(value);
  return Number.isFinite(number) ? number : null;
}
export function evaluateSharedRules(ctx: SharedContext): SharedIssue[] {
  const changed = new Set(ctx.changedKeys.map((key) => key.toLowerCase()));
  return policy.rules.filter((rule) => rule.changedAny.some((key) => changed.has(key)) && rule.all.every((clause) => {
    if ("anyTruthy" in clause && clause.anyTruthy) return clause.anyTruthy.some((key) => policy.truthy.includes((ctx.values[key] ?? "").trim().toLowerCase()));
    if ("rtStatus" in clause) return ctx.rtStatus === clause.rtStatus;
    if (!("key" in clause) || !clause.key) return false;
    const value = finiteDecimal(ctx.values[clause.key] ?? "");
    return value !== null && (!("lte" in clause) || clause.lte == null || value <= clause.lte) && (!("gt" in clause) || clause.gt == null || value > clause.gt);
  })).map((rule) => ({ code: rule.code as ValidationIssueCode, severity: "warning", key: rule.key }));
}
