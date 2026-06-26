import { describe, expect, it } from "vitest";
import { clampParamValue } from "./paramValue";

describe("clampParamValue", () => {
  it("clamps float to min/max", () => {
    expect(
      clampParamValue("5", { min: "0", max: "4", value_type: "float" }),
    ).toBe("4");
    expect(
      clampParamValue("0.5", { min: "1", max: "4", value_type: "float" }),
    ).toBe("1");
  });

  it("clamps int and rounds", () => {
    expect(
      clampParamValue("2.7", { min: "0", max: "4", value_type: "int" }),
    ).toBe("3");
  });

  it("leaves sentinel values unchanged", () => {
    expect(
      clampParamValue("-1", { min: "0", max: "4", value_type: "int" }),
    ).toBe("-1");
  });

  it("passes through when min/max missing", () => {
    expect(clampParamValue("99", { value_type: "int" })).toBe("99");
  });
});
