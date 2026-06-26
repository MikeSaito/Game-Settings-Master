import { describe, expect, it } from "vitest";
import type { GameParameter } from "@/lib/core/types";
import { applyParamDependencies, reconcileAllParams } from "./paramDependencies";

function gusParam(key: string, value: string): GameParameter {
  return {
    key,
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value,
    title: key,
    description: "",
    impact: "",
    category: "Scalability",
    min: null,
    max: null,
    in_game_label: null,
    value_hint: null,
    value_type: "string",
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
  };
}

const rtxGpu = {
  name: "RTX 4070",
  vendor: "nvidia" as const,
  supports_dlss: true,
  supports_dlss_fg: true,
  supports_ray_tracing: true,
};

describe("applyParamDependencies", () => {
  it("syncs DLSSQualityMode when DLSSMode changes", () => {
    const params = [
      gusParam("DLSSMode", "Off"),
      gusParam("DLSSQualityMode", "0"),
      gusParam("UpscalingMethod", "U_None"),
    ];
    const next = applyParamDependencies(
      params,
      {
        key: "DLSSMode",
        section: "ScalabilityGroups",
        file: "GameUserSettings.ini",
        value: "Quality",
      },
      rtxGpu,
    );
    expect(next.find((p) => p.key === "DLSSQualityMode")?.value).toBe("3");
    expect(next.find((p) => p.key === "UpscalingMethod")?.value).toBe("U_DLSS");
  });

  it("turns off frame generation when DLSS is off", () => {
    const params = [
      gusParam("DLSSMode", "Quality"),
      gusParam("UpscalingFrameGeneration", "1"),
      gusParam("DLSSQualityMode", "3"),
      gusParam("UpscalingMethod", "U_DLSS"),
    ];
    const next = applyParamDependencies(
      params,
      {
        key: "DLSSMode",
        section: "ScalabilityGroups",
        file: "GameUserSettings.ini",
        value: "Off",
      },
      rtxGpu,
    );
    expect(next.find((p) => p.key === "UpscalingFrameGeneration")?.value).toBe("0");
  });
});

describe("reconcileAllParams", () => {
  it("clamps ResolutionScaleMin when above max", () => {
    const params = [
      gusParam("ResolutionScaleMin", "120"),
      gusParam("ResolutionScaleMax", "100"),
    ];
    const next = reconcileAllParams(params);
    expect(next.find((p) => p.key === "ResolutionScaleMin")?.value).toBe("100");
  });
});
