import { describe, expect, it } from "vitest";
import { paramValuesEqual } from "./paramValueEqual";

describe("paramValuesEqual", () => {
  it("matches exact strings", () => {
    expect(paramValuesEqual("1.0", "1.0")).toBe(true);
  });

  it("matches case-insensitively", () => {
    expect(paramValuesEqual("True", "true")).toBe(true);
  });

  it("matches equivalent floats", () => {
    expect(paramValuesEqual("1.0", "1.0000")).toBe(true);
    expect(paramValuesEqual("0.5", "0.50001")).toBe(true);
  });

  it("rejects different values", () => {
    expect(paramValuesEqual("1", "2")).toBe(false);
    expect(paramValuesEqual("on", "off")).toBe(false);
  });
});
