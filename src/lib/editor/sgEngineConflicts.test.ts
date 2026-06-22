import { describe, expect, it } from "vitest";
import type { GameParameter } from "../core/types";
import {
  collectPendingKeys,
  detectSgEngineConflicts,
  extractRelatedRCvars,
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
