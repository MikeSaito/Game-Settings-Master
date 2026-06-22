import { describe, expect, it } from "vitest";
import type { GpuCapabilities } from "../core/types";
import { filterSelectOptions, isParamVisible } from "../gpu/gpuCompat";

const rtxGpu: GpuCapabilities = {
  name: "RTX 4070",
  vendor: "nvidia",
  supports_dlss: true,
  supports_dlss_fg: true,
  supports_ray_tracing: true,
};

const amdGpu: GpuCapabilities = {
  name: "RX 7800 XT",
  vendor: "amd",
  supports_dlss: false,
  supports_dlss_fg: false,
  supports_ray_tracing: false,
};

describe("isParamVisible", () => {
  it("hides DLSS keys on AMD", () => {
    expect(
      isParamVisible(
        { key: "DLSSMode" } as Parameters<typeof isParamVisible>[0],
        amdGpu,
      ),
    ).toBe(false);
  });

  it("shows DLSS keys on RTX", () => {
    expect(
      isParamVisible(
        { key: "DLSSMode" } as Parameters<typeof isParamVisible>[0],
        rtxGpu,
      ),
    ).toBe(true);
  });

  it("hides frame generation without DLSS FG", () => {
    const noFg: GpuCapabilities = { ...rtxGpu, supports_dlss_fg: false };
    expect(
      isParamVisible(
        { key: "UpscalingFrameGeneration" } as Parameters<typeof isParamVisible>[0],
        noFg,
      ),
    ).toBe(false);
  });
});

describe("filterSelectOptions", () => {
  it("removes DLSS upscaling options on AMD", () => {
    const opts = filterSelectOptions(
      { key: "UpscalingMethod" } as Parameters<typeof filterSelectOptions>[0],
      amdGpu,
    );
    expect(opts).toEqual(["U_None", "U_FSR", "U_TSR"]);
    expect(opts).not.toContain("U_DLSS");
  });

  it("returns null when no filtering needed", () => {
    expect(
      filterSelectOptions(
        { key: "AntiAliasingType" } as Parameters<typeof filterSelectOptions>[0],
        rtxGpu,
      ),
    ).toBeNull();
  });
});
