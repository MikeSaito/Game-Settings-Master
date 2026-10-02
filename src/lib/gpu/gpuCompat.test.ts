import { describe, expect, it } from "vitest";
import type { GpuCapabilities } from "@/lib/core/types";
import { filterSelectOptions, isParamVisible } from "@/lib/gpu/gpuCompat";

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
  it("shows ray tracing on AMD and Intel with confirmed hardware support", () => {
    for (const vendor of ["amd", "intel"] as const) {
      expect(
        isParamVisible(
          { key: "r.RayTracing" } as Parameters<typeof isParamVisible>[0],
          {
            ...amdGpu,
            vendor,
            supports_ray_tracing: true,
            ray_tracing_status: "supported",
          },
        ),
      ).toBe(true);
    }
  });
  it("keeps unknown capabilities visible without claiming support", () => {
    expect(
      isParamVisible(
        { key: "r.RayTracing" } as Parameters<typeof isParamVisible>[0],
        { ...amdGpu, ray_tracing_status: "unknown" },
      ),
    ).toBe(true);
    expect(
      isParamVisible(
        { key: "DLSSMode" } as Parameters<typeof isParamVisible>[0],
        { ...amdGpu, vendor: "unknown" },
      ),
    ).toBe(true);
  });
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

  it("keeps generic frame generation visible without DLSS FG", () => {
    const noFg: GpuCapabilities = { ...rtxGpu, supports_dlss_fg: false };
    expect(
      isParamVisible(
        { key: "UpscalingFrameGeneration" } as Parameters<
          typeof isParamVisible
        >[0],
        noFg,
      ),
    ).toBe(true);
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
        { key: "AntiAliasingType" } as Parameters<
          typeof filterSelectOptions
        >[0],
        rtxGpu,
      ),
    ).toBeNull();
  });
});
