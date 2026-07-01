import { describe, expect, it } from "vitest";
import type { GameParameter } from "@/lib/core/types";
import {
  collectPendingKeys,
  detectSgEngineConflicts,
  extractRelatedRCvars,
  parseTierCvarsForSgValue,
  analyzeSgEngineConflictGroups,
  resolveConflictKeepSg,
  sgQualityToRPrefix,
  matchesSgRPrefixFamily,
} from "./sgEngineConflicts";

function param(overrides: Partial<GameParameter> & Pick<GameParameter, "key">): GameParameter {
  return {
    section: "SystemSettings",
    file: "Engine.ini",
    value: "1",
    title: overrides.key,
    description: "",
    impact: "",
    category: "Rendering",
    min: null,
    max: null,
    in_game_label: null,
    value_hint: null,
    value_type: "int",
    known: true,
    editable: true,
    present_in_ini: true,
    default_value: null,
    ui_control: null,
    step: null,
    options: null,
    recommended: null,
    catalog_recommended: false,
    tier_hint: null,
    ...overrides,
  };
}

describe("extractRelatedRCvars", () => {
  it("parses r.* from tier hint", () => {
    const hint =
      "Low (0): r.ShadowQuality=0 · r.Shadow.MaxResolution=512 | High (2): r.ShadowQuality=2";
    expect(extractRelatedRCvars(hint)).toEqual(
      expect.arrayContaining(["r.shadowquality", "r.shadow.maxresolution"]),
    );
  });
});

describe("parseTierCvarsForSgValue", () => {
  it("returns CVars for matching sg quality index", () => {
    const hint =
      "Low (0): r.ShadowQuality=0 · r.Shadow.MaxResolution=512 | High (2): r.ShadowQuality=2 · r.Shadow.MaxResolution=1024";
    const preview = parseTierCvarsForSgValue(hint, "2");
    expect(preview?.tierLabel).toBe("High (2)");
    expect(preview?.cvars).toEqual([
      { key: "r.ShadowQuality", value: "2" },
      { key: "r.Shadow.MaxResolution", value: "1024" },
    ]);
  });

  it("returns null when tier hint has no CVars for index", () => {
    expect(parseTierCvarsForSgValue("Low (0) | High (2)", "2")).toBeNull();
  });
});

describe("sgQualityToRPrefix", () => {
  it("derives r.shadow from sg.ShadowQuality", () => {
    expect(sgQualityToRPrefix("sg.ShadowQuality")).toBe("r.shadow");
  });

  it("derives r.viewdistance from sg.ViewDistanceQuality", () => {
    expect(sgQualityToRPrefix("sg.ViewDistanceQuality")).toBe("r.viewdistance");
  });
});

describe("matchesSgRPrefixFamily", () => {
  it("matches r.ShadowQuality and r.Shadow.* but not unrelated keys", () => {
    expect(matchesSgRPrefixFamily("r.ShadowQuality", "r.shadow")).toBe(true);
    expect(matchesSgRPrefixFamily("r.Shadow.MaxResolution", "r.shadow")).toBe(true);
    expect(matchesSgRPrefixFamily("r.ShadowMap", "r.shadow")).toBe(false);
  });

  it("matches r.TextureQuality but not r.TextureStreamingPoolSize", () => {
    expect(matchesSgRPrefixFamily("r.TextureQuality", "r.texture")).toBe(true);
    expect(matchesSgRPrefixFamily("r.TextureStreamingPoolSize", "r.texture")).toBe(false);
  });
});

describe("analyzeSgEngineConflictGroups", () => {
  it("does not flag unrelated r.TextureStreamingPoolSize for sg.TextureQuality", () => {
    const params = [
      param({
        key: "sg.TextureQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.TextureQuality=0 | High (2): r.TextureQuality=2",
        present_in_ini: true,
        value: "2",
      }),
      param({
        key: "r.TextureStreamingPoolSize",
        file: "Engine.ini",
        present_in_ini: true,
        value: "4096",
      }),
    ];
    const enabled = new Set(["Engine.ini::r.TextureStreamingPoolSize"]);
    expect(analyzeSgEngineConflictGroups(params, new Set(), enabled)).toHaveLength(0);
  });

  it("detects r.Shadow.MaxResolution via derived prefix beyond tier_hint", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.ShadowQuality=0 | High (2): r.ShadowQuality=2",
        present_in_ini: true,
        value: "2",
      }),
      param({
        key: "r.Shadow.MaxResolution",
        file: "Engine.ini",
        present_in_ini: true,
        value: "4096",
      }),
    ];
    const enabled = new Set(["Engine.ini::r.Shadow.MaxResolution"]);
    const groups = analyzeSgEngineConflictGroups(params, new Set(), enabled);
    expect(groups).toHaveLength(1);
    expect(groups[0]?.conflictingRParams.map((p) => p.key)).toContain("r.Shadow.MaxResolution");
  });

  it("builds actionable group with tier preview", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.ShadowQuality=0 | High (2): r.ShadowQuality=2",
        present_in_ini: true,
        value: "2",
      }),
      param({
        key: "r.ShadowQuality",
        file: "Engine.ini",
        present_in_ini: true,
        value: "5",
      }),
    ];
    const groups = analyzeSgEngineConflictGroups(params, new Set(), new Set(["Engine.ini::r.ShadowQuality"]));
    expect(groups).toHaveLength(1);
    expect(groups[0]?.sgKey).toBe("sg.shadowquality");
    expect(groups[0]?.tierPreview?.cvars).toEqual([{ key: "r.ShadowQuality", value: "2" }]);
    expect(groups[0]?.conflictingRParams.map((p) => p.key)).toEqual(["r.ShadowQuality"]);
  });
});

describe("resolveConflictKeepSg", () => {
  it("disables engine toggle and reverts r.* draft values", () => {
    const baseline = param({
      key: "r.ShadowQuality",
      file: "Engine.ini",
      present_in_ini: true,
      value: "5",
    });
    const draft = { ...baseline, value: "3" };
    const group = {
      sgKey: "sg.shadowquality",
      sgParam: param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        value: "2",
      }),
      sgValue: "2",
      conflictingRParams: [draft],
      tierPreview: null,
    };
    const enabled = new Set(["Engine.ini::r.ShadowQuality"]);
    const { params: nextParams, engineEnabled } = resolveConflictKeepSg(
      group,
      [draft],
      [baseline],
      enabled,
    );
    expect(engineEnabled.has("Engine.ini::r.ShadowQuality")).toBe(false);
    expect(nextParams[0]?.value).toBe("5");
  });

  it("clears Scalability.ini r.* overrides via ini toggle", () => {
    const baseline = param({
      key: "r.ShadowQuality",
      file: "Scalability.ini",
      section: "[ShadowQuality@3]",
      present_in_ini: true,
      value: "5",
    });
    const draft = { ...baseline };
    const group = {
      sgKey: "sg.shadowquality",
      sgParam: param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        value: "2",
      }),
      sgValue: "2",
      conflictingRParams: [draft],
      tierPreview: null,
    };
    const enabled = new Set(["Scalability.ini::r.ShadowQuality"]);
    const { params: nextParams, engineEnabled } = resolveConflictKeepSg(
      group,
      [draft],
      [baseline],
      enabled,
    );
    expect(engineEnabled.has("Scalability.ini::r.ShadowQuality")).toBe(false);
    expect(nextParams[0]?.value).toBe("5");
  });

  it("pending removal does not count as active override", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.ShadowQuality=0",
        present_in_ini: true,
        value: "2",
      }),
      param({
        key: "r.ShadowQuality",
        file: "Scalability.ini",
        section: "[ShadowQuality@3]",
        present_in_ini: true,
        value: "5",
      }),
    ];
    const pending = collectPendingKeys(
      {},
      { "Scalability.ini": { "[ShadowQuality@3]": ["r.ShadowQuality"] } },
    );
    const conflicts = detectSgEngineConflicts(params, pending, new Set());
    expect(conflicts.size).toBe(0);
  });
});

describe("detectSgEngineConflicts", () => {
  it("flags sg.ShadowQuality and r.Shadow* when both active", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.ShadowQuality=0",
        present_in_ini: true,
        value: "3",
      }),
      param({
        key: "r.ShadowQuality",
        file: "Engine.ini",
        present_in_ini: true,
        value: "5",
      }),
    ];
    const conflicts = detectSgEngineConflicts(params, new Set(), new Set(["Engine.ini::r.ShadowQuality"]));
    expect(conflicts.has("sg.shadowquality")).toBe(true);
    expect(conflicts.has("r.shadowquality")).toBe(true);
  });

  it("flags r.shadow prefix overrides for shadow sg", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        present_in_ini: true,
        value: "2",
      }),
      param({
        key: "r.Shadow.MaxResolution",
        file: "Engine.ini",
        present_in_ini: true,
        value: "4096",
      }),
    ];
    const enabled = new Set(["Engine.ini::r.Shadow.MaxResolution"]);
    const conflicts = detectSgEngineConflicts(params, new Set(), enabled);
    expect(conflicts.has("r.shadow.maxresolution")).toBe(true);
  });

  it("detects pending r.* without sg in pending", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        tier_hint: "Low (0): r.ShadowQuality=0",
        present_in_ini: true,
        value: "3",
      }),
      param({
        key: "r.ShadowQuality",
        file: "Engine.ini",
        present_in_ini: false,
        value: "",
      }),
    ];
    const pending = collectPendingKeys(
      { "Engine.ini": { SystemSettings: { "r.ShadowQuality": "5" } } },
      {},
    );
    const conflicts = detectSgEngineConflicts(params, pending, new Set());
    expect(conflicts.has("r.shadowquality")).toBe(true);
  });

  it("no conflict when sg inactive", () => {
    const params = [
      param({
        key: "sg.ShadowQuality",
        file: "GameUserSettings.ini",
        section: "ScalabilityGroups",
        present_in_ini: false,
        value: "",
      }),
      param({
        key: "r.ShadowQuality",
        file: "Engine.ini",
        present_in_ini: true,
        value: "5",
      }),
    ];
    expect(detectSgEngineConflicts(params, new Set(), new Set()).size).toBe(0);
  });
});
