import { describe, expect, it } from "vitest";
import { buildCustomChanges } from "./buildCustomChanges";
import type { GameParameter } from "./types";

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
});
