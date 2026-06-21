import { describe, expect, it } from "vitest";
import { humanizeCvarKey } from "./cvarHumanize";

const t = ((key: string, options?: { defaultValue?: string }) => {
  const map: Record<string, string> = {
    "humanize.view": "обзор",
    "humanize.distance": "дистанция",
    "humanize.scale": "масштаб",
    "humanize.shadow": "тени",
    "humanize.temporal": "временное",
    "humanize.aa": "AA",
    "humanize.upsampling": "апскейлинг",
    "humanize.upscaling": "апскейлинг",
    "humanize.final": "финальный",
    "humanize.gather": "сбор",
    "humanize.quality": "качество",
  };
  return map[key] ?? options?.defaultValue ?? key;
}) as Parameters<typeof humanizeCvarKey>[1];

describe("humanizeCvarKey", () => {
  it("splits camelCase keys into readable Russian tokens", () => {
    expect(humanizeCvarKey("r.ViewDistanceScale", t)).toBe("обзор · дистанция · масштаб");
  });

  it("strips sg prefix", () => {
    expect(humanizeCvarKey("sg.ShadowQuality", t)).toBe("тени · качество");
  });

  it("keeps acronyms and splits Unreal camel-case keys", () => {
    expect(humanizeCvarKey("r.TemporalAA.Upsampling", t)).toBe(
      "временное · AA · апскейлинг",
    );
    expect(humanizeCvarKey("r.Lumen.FinalGatherQuality", t)).toBe(
      "Lumen · финальный · сбор · качество",
    );
  });
});
