import { describe, expect, it } from "vitest";
import { buildCustomChanges } from "./buildCustomChanges";
import type { GameParameter, GpuCapabilities } from "@/lib/core/types";

function param(
  overrides: Partial<GameParameter> & Pick<GameParameter, "key" | "value">,
): GameParameter {
  return {
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    title: overrides.key,
    description: "",
    impact: "",
    category: "Scalability",
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

describe("buildCustomChanges", () => {
  it("includes only changed editable parameters", () => {
    const baseline = [
      param({ key: "sg.ShadowQuality", value: "2" }),
      param({ key: "sg.TextureQuality", value: "3" }),
    ];
    const edited = [
      param({ key: "sg.ShadowQuality", value: "4" }),
      param({ key: "sg.TextureQuality", value: "3" }),
    ];
    const { files, removals } = buildCustomChanges(
      edited,
      baseline,
      undefined,
      new Set(),
      new Set(["Scalability"]),
    );
    const gus = files["GameUserSettings.ini"];
    expect(gus).toBeDefined();
    const section =
      gus["[ScalabilityGroups]"] ??
      gus.ScabilityGroups ??
      Object.values(gus)[0];
    expect(section?.["sg.ShadowQuality"]).toBe("4");
    expect(section?.["sg.TextureQuality"]).toBeUndefined();
    expect(Object.keys(removals)).toHaveLength(0);
  });

  it("skips values equal to baseline", () => {
    const baseline = [param({ key: "sg.ShadowQuality", value: "2" })];
    const edited = [param({ key: "sg.ShadowQuality", value: "2.0" })];
    const { files } = buildCustomChanges(
      edited,
      baseline,
      undefined,
      new Set(),
      new Set(["Scalability"]),
    );
    expect(Object.keys(files)).toHaveLength(0);
  });

  it("includes editable unknown GameUserSettings parameters", () => {
    const baseline = [
      param({
        key: "DLSSMode",
        value: "Off",
        category: "GameSpecific",
        known: false,
      }),
    ];
    const edited = [
      param({
        key: "DLSSMode",
        value: "Quality",
        category: "GameSpecific",
        known: false,
      }),
    ];

    const { files } = buildCustomChanges(
      edited,
      baseline,
      undefined,
      new Set(),
      new Set(["GameSpecific"]),
    );

    const section = Object.values(files["GameUserSettings.ini"] ?? {})[0];
    expect(section?.DLSSMode).toBe("Quality");
  });

  it("writes hidden GPU dependency changes produced by reconciliation", () => {
    const noFrameGenerationGpu: GpuCapabilities = {
      name: "RTX 3060",
      vendor: "nvidia",
      supports_dlss: true,
      supports_dlss_fg: false,
      supports_ray_tracing: true,
    };
    const baseline = [
      param({ key: "UpscalingMethod", value: "U_DLSS", category: "GameSpecific" }),
      param({ key: "DLSSMode", value: "Quality", category: "GameSpecific" }),
      param({ key: "UpscalingFrameGeneration", value: "1", category: "GameSpecific" }),
    ];
    const edited = [
      param({ key: "UpscalingMethod", value: "U_FSR", category: "GameSpecific" }),
      param({ key: "DLSSMode", value: "Quality", category: "GameSpecific" }),
      param({ key: "UpscalingFrameGeneration", value: "1", category: "GameSpecific" }),
    ];

    const { files } = buildCustomChanges(
      edited,
      baseline,
      noFrameGenerationGpu,
      new Set(),
      new Set(["GameSpecific"]),
    );

    const section = Object.values(files["GameUserSettings.ini"] ?? {})[0];
    expect(section?.UpscalingMethod).toBe("U_FSR");
    expect(section?.UpscalingFrameGeneration).toBe("0");
  });
});
