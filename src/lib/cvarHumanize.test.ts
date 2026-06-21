import { describe, expect, it } from "vitest";
import { humanizeCvarKey } from "./cvarHumanize";

const t = ((key: string, options?: { defaultValue?: string }) => {
  const map: Record<string, string> = {
    "humanize.view": "обзор",
    "humanize.distance": "дистанция",
    "humanize.scale": "масштаб",
  };
  return map[key] ?? options?.defaultValue ?? key;
}) as Parameters<typeof humanizeCvarKey>[1];

describe("humanizeCvarKey", () => {
  it("splits camelCase keys into readable Russian tokens", () => {
    expect(humanizeCvarKey("r.ViewDistanceScale", t)).toBe("обзор · дистанция · масштаб");
  });

  it("strips sg prefix", () => {
    expect(humanizeCvarKey("sg.ShadowQuality", t)).toBe("Shadow · Quality");
  });
});
