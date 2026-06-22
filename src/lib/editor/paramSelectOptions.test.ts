import { describe, expect, it } from "vitest";
import { getParamSelectOptions } from "./paramSelectOptions";
import type { GameParameter } from "../core/types";

function param(key: string, value = ""): GameParameter {
  return { key, value } as GameParameter;
}

describe("getParamSelectOptions", () => {
  it("returns DLSS mode list", () => {
    expect(getParamSelectOptions(param("DLSSMode", "Quality"), undefined)).toEqual([
      "Off",
      "Performance",
      "Balanced",
      "Quality",
      "UltraQuality",
      "DLAA",
    ]);
  });

  it("appends unknown current DLSS value", () => {
    expect(getParamSelectOptions(param("DLSSMode", "CustomMode"), undefined)).toEqual([
      "Off",
      "Performance",
      "Balanced",
      "Quality",
      "UltraQuality",
      "DLAA",
      "CustomMode",
    ]);
  });

  it("filters upscaling options without DLSS on AMD", () => {
    const gpu = {
      name: "RX 6800",
      vendor: "amd",
      supports_dlss: false,
      supports_dlss_fg: false,
      supports_ray_tracing: false,
    } as const;
    expect(getParamSelectOptions(param("UpscalingMethod", "U_TSR"), gpu)).toEqual([
      "U_None",
      "U_FSR",
      "U_TSR",
    ]);
  });
});
