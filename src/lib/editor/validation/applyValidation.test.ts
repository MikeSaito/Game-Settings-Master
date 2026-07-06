import { describe, expect, it } from "vitest";
import { buildIniSnapshot } from "../iniSnapshot";
import { validateApplyPlan, validateOverridePlan, sgMaxForKey } from "./applyValidation";
import type { GameParameter, GameProfile } from "@/lib/core";

const game: Pick<GameProfile, "engine_family" | "engine_version"> = {
  engine_family: "ue5",
  engine_version: "5.4",
};

function baseParam(overrides: Partial<GameParameter>): GameParameter {
  return {
    key: "sg.ViewDistanceQuality",
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "2",
    title: "View distance",
    description: "",
    impact: "",
    category: "Scalability",
    min: "0",
    max: "4",
    in_game_label: null,
    value_hint: null,
    value_type: "int",
    known: true,
    editable: true,
    present_in_ini: true,
    ...overrides,
  };
}

function applyCtx(
  baseline: GameParameter,
  params: GameParameter[],
  extra: Record<string, unknown> = {},
) {
  const parameters = [baseline];
  return {
    game,
    panel: "basic" as const,
    params,
    parameters,
    gpu: undefined,
    limits: undefined,
    engineEnabled: new Set<string>(),
    shippedIniKeys: buildIniSnapshot(parameters),
    editableCategories: new Set(["Scalability"]),
    ...extra,
  };
}

describe("validateApplyPlan", () => {
  it("blocks sg above global_max on basic panel", () => {
    const baseline = baseParam({ value: "2" });
    const params = [baseParam({ value: "5" })];
    const issues = validateApplyPlan(
      applyCtx(baseline, params, {
        limits: { global_max: 4, groups: {}, sources: [] },
      }),
    );
    expect(issues.some((i) => i.code === "sg_exceeds_limit" && i.severity === "error")).toBe(
      true,
    );
  });

  it("blocks pending sg changes while limits are loading", () => {
    const baseline = baseParam({ value: "2" });
    const params = [baseParam({ value: "3" })];
    const issues = validateApplyPlan(
      applyCtx(baseline, params, { limits: undefined, limitsPending: true }),
    );
    expect(issues.some((i) => i.code === "sg_limits_pending" && i.severity === "error")).toBe(
      true,
    );
  });

  it("uses default sg max when limits query failed", () => {
    const baseline = baseParam({ value: "2" });
    const params = [baseParam({ value: "5" })];
    const issues = validateApplyPlan(
      applyCtx(baseline, params, { limits: undefined, limitsPending: false }),
    );
    expect(issues.some((i) => i.code === "sg_exceeds_limit" && i.severity === "error")).toBe(
      true,
    );
  });

  it("uses per-group sg max when group limit is lower than global", () => {
    const baseline = baseParam({ value: "2" });
    const params = [baseParam({ value: "3" })];
    const limits = {
      global_max: 4,
      groups: { "sg.ViewDistanceQuality": 2 },
      sources: [],
    };
    expect(sgMaxForKey(limits, "sg.ViewDistanceQuality")).toBe(2);
    const issues = validateApplyPlan(
      applyCtx(baseline, params, { limits }),
    );
    expect(issues.some((i) => i.code === "sg_exceeds_limit" && i.severity === "error")).toBe(
      true,
    );
  });
});

describe("validateOverridePlan", () => {
  it("blocks sg above limit in preset files", () => {
    const issues = validateOverridePlan(
      {
        "GameUserSettings.ini": {
          "[ScalabilityGroups]": { "sg.ViewDistanceQuality": "9" },
        },
      },
      {},
      {
        game,
        params: [],
        gpu: undefined,
        limits: { global_max: 4, groups: {}, sources: [] },
        limitsPending: false,
      },
    );
    expect(issues.some((i) => i.code === "sg_exceeds_limit")).toBe(true);
  });

  it("blocks removal of shipped ini keys in presets", () => {
    const param = baseParam({ present_in_ini: true });
    const shipped = buildIniSnapshot([param]);
    const issues = validateOverridePlan(
      {},
      {
        "GameUserSettings.ini": {
          "[ScalabilityGroups]": ["sg.ViewDistanceQuality"],
        },
      },
      {
        game,
        params: [param],
        gpu: undefined,
        limits: undefined,
        limitsPending: false,
        shippedIniKeys: shipped,
      },
    );
    expect(issues.some((i) => i.code === "shipped_key_removal" && i.severity === "error")).toBe(
      true,
    );
  });

  it("allows removing shipped r.* overrides from Engine.ini", () => {
    const param = {
      ...baseParam({
        key: "r.ShadowQuality",
        file: "Engine.ini",
        section: "SystemSettings",
        present_in_ini: true,
      }),
    };
    const shipped = buildIniSnapshot([param]);
    const issues = validateOverridePlan(
      {},
      {
        "Engine.ini": {
          SystemSettings: ["r.ShadowQuality"],
        },
      },
      {
        game,
        params: [param],
        gpu: undefined,
        limits: undefined,
        limitsPending: false,
        shippedIniKeys: shipped,
      },
    );
    expect(issues.some((i) => i.code === "shipped_key_removal")).toBe(false);
  });
});

describe("canApplyPlan", () => {
  it("blocks when errors present", async () => {
    const { canApplyPlan, assertApplyPlanAllowed } = await import("./applyValidation");
    const issues = [
      {
        code: "sg_exceeds_limit" as const,
        severity: "error" as const,
        key: "sg.ViewDistanceQuality",
        i18nKey: "validation.sgExceedsLimit",
      },
    ];
    expect(canApplyPlan(issues, false).allowed).toBe(false);
    expect(() => assertApplyPlanAllowed(issues, false, (k) => k)).toThrow();
  });
});

describe("validationIssueSignature", () => {
  it("is stable for equivalent issue sets", async () => {
    const { validationIssueSignature } = await import("./applyValidation");
    const a = validationIssueSignature([
      {
        code: "sg_r_conflict",
        severity: "warning",
        key: "sg.shadowquality",
        i18nKey: "validation.sgRConflict",
        i18nParams: { sg: "sg.ShadowQuality" },
      },
    ]);
    const b = validationIssueSignature([
      {
        code: "sg_r_conflict",
        severity: "warning",
        key: "sg.shadowquality",
        i18nKey: "validation.sgRConflict",
        i18nParams: { sg: "sg.ShadowQuality" },
      },
    ]);
    expect(a).toBe(b);
  });
});
