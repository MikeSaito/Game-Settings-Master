import { describe, expect, it } from "vitest";
import { keyAppliesToGame, parseUeSemver } from "./ueVersion";

describe("parseUeSemver", () => {
  it("parses major.minor.patch", () => {
    expect(parseUeSemver("5.4.2")).toEqual({ major: 5, minor: 4, patch: 2 });
  });
});

describe("keyAppliesToGame", () => {
  it("rejects keys introduced after game version", () => {
    expect(keyAppliesToGame("r.Hair.Strands", "ue5", "5.0")).toBe(false);
  });

  it("allows keys present in game version", () => {
    expect(keyAppliesToGame("sg.ShadowQuality", "ue5", "5.4")).toBe(true);
  });
});
