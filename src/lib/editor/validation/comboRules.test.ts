import { describe, expect, it } from "vitest";
import { engineParamId } from "../engineParams";
import { buildIniSnapshot } from "../iniSnapshot";
import { evaluateComboRules } from "./comboRules";
import type { GameParameter } from "@/lib/core";

function param(overrides: Partial<GameParameter>): GameParameter {
  return {
    key: "sg.ShadowQuality",
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "0",
    title: "Shadows",
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

describe("evaluateComboRules", () => {
  it("warns when same key is pending in Engine and Scalability", () => {
    const issues = evaluateComboRules({
      panel: "advanced",
      params: [],
      gpu: undefined,
      engineEnabled: new Set(),
      files: {
        "Engine.ini": { "[SystemSettings]": { "r.ShadowQuality": "3" } },
        "Scalability.ini": { "[ShadowQuality@3]": { "r.ShadowQuality": "2" } },
      },
    });
    expect(issues.some((i) => i.code === "combo_engine_scalability_dup")).toBe(true);
  });

  it("warns on RT with low shadow quality", () => {
    const rtParam = param({
      key: "r.RayTracing",
      file: "Engine.ini",
      section: "SystemSettings",
      value: "1",
      category: "Rendering",
      value_type: "bool",
    });
    const sgParam = param({ key: "sg.ShadowQuality", value: "0" });
    const shipped = buildIniSnapshot([sgParam]);
    const issues = evaluateComboRules({
      panel: "advanced",
      params: [sgParam, rtParam],
      gpu: {
        name: "RTX",
        vendor: "nvidia",
        supports_dlss: true,
        supports_dlss_fg: true,
        supports_ray_tracing: true,
      },
      engineEnabled: new Set([engineParamId(rtParam)]),
      shippedIniKeys: shipped,
      files: {
        "Engine.ini": { "[SystemSettings]": { "r.RayTracing": "1" } },
      },
    });
    expect(issues.some((i) => i.code === "combo_rt_shadows")).toBe(true);
  });

  it("ignores RT draft values when engine toggle is off", () => {
    const rtParam = param({
      key: "r.RayTracing",
      file: "Engine.ini",
      section: "SystemSettings",
      value: "1",
      category: "Rendering",
      value_type: "bool",
      present_in_ini: false,
    });
    const issues = evaluateComboRules({
      panel: "advanced",
      params: [param({ key: "sg.ShadowQuality", value: "0" }), rtParam],
      gpu: undefined,
      engineEnabled: new Set(),
      files: {},
    });
    expect(issues.some((i) => i.code.startsWith("combo_rt"))).toBe(false);
  });

  it("does not block basic apply when RT exists only in advanced draft", () => {
    const rtParam = param({
      key: "r.RayTracing",
      file: "Engine.ini",
      section: "SystemSettings",
      value: "1",
      category: "Rendering",
      value_type: "bool",
      present_in_ini: true,
    });
    const sgParam = param({ key: "sg.ViewDistanceQuality", value: "3" });
    const shipped = buildIniSnapshot([sgParam]);
    const issues = evaluateComboRules({
      panel: "basic",
      params: [sgParam, rtParam],
      gpu: undefined,
      gpuPending: true,
      engineEnabled: new Set([engineParamId(rtParam)]),
      shippedIniKeys: shipped,
      files: {
        "GameUserSettings.ini": {
          "[ScalabilityGroups]": { "sg.ViewDistanceQuality": "3" },
        },
      },
    });
    expect(issues.some((i) => i.code === "combo_gpu_pending")).toBe(false);
  });

  it("blocks RT apply while GPU info is loading", () => {
    const rtParam = param({
      key: "r.RayTracing",
      file: "Engine.ini",
      section: "SystemSettings",
      value: "1",
      category: "Rendering",
      value_type: "bool",
    });
    const issues = evaluateComboRules({
      panel: "advanced",
      params: [rtParam],
      gpu: undefined,
      gpuPending: true,
      engineEnabled: new Set([engineParamId(rtParam)]),
      files: {
        "Engine.ini": { "[SystemSettings]": { "r.RayTracing": "1" } },
      },
    });
    expect(issues.some((i) => i.code === "combo_gpu_pending" && i.severity === "error")).toBe(
      true,
    );
  });

  it("warns when GPU query failed but RT is in pending apply", () => {
    const rtParam = param({
      key: "r.RayTracing",
      file: "Engine.ini",
      section: "SystemSettings",
      value: "1",
      category: "Rendering",
      value_type: "bool",
    });
    const issues = evaluateComboRules({
      panel: "advanced",
      params: [rtParam],
      gpu: undefined,
      gpuUnavailable: true,
      engineEnabled: new Set([engineParamId(rtParam)]),
      files: {
        "Engine.ini": { "[SystemSettings]": { "r.RayTracing": "1" } },
      },
    });
    expect(issues.some((i) => i.code === "combo_rt_gpu_unknown")).toBe(true);
  });
});
