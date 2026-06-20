import { describe, expect, it } from "vitest";
import {
  buildCategoryList,
  filterParamsByCategoryAndSearch,
  paramRowKey,
} from "./advancedEditorFilters";
import type { GameParameter } from "./types";

function param(
  overrides: Partial<GameParameter> & Pick<GameParameter, "key" | "category">,
): GameParameter {
  return {
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "2",
    title: overrides.key,
    description: "",
    impact: "",
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

describe("filterParamsByCategoryAndSearch", () => {
  it("returns items for active category", () => {
    const items = [
      param({ key: "sg.ShadowQuality", category: "Scalability" }),
      param({ key: "r.ViewDistance", category: "Rendering", file: "Engine.ini" }),
    ];
    const filtered = filterParamsByCategoryAndSearch(items, "Scalability", "", new Set());
    expect(filtered).toHaveLength(1);
    expect(filtered[0].key).toBe("sg.ShadowQuality");
  });

  it("filters by search query", () => {
    const items = [
      param({ key: "sg.ShadowQuality", category: "Scalability", title: "Shadows" }),
      param({ key: "sg.TextureQuality", category: "Scalability", title: "Textures" }),
    ];
    const filtered = filterParamsByCategoryAndSearch(items, "Scalability", "shadow", new Set());
    expect(filtered).toHaveLength(1);
    expect(filtered[0].key).toBe("sg.ShadowQuality");
  });
});

describe("buildCategoryList", () => {
  it("orders known categories first", () => {
    const list = buildCategoryList([
      param({ key: "a", category: "Other" }),
      param({ key: "b", category: "Scalability" }),
    ]);
    expect(list[0].cat).toBe("Scalability");
    expect(list.some((c) => c.cat === "Other")).toBe(true);
  });
});

describe("paramRowKey", () => {
  it("is stable per file/section/key", () => {
    const p = param({ key: "sg.ShadowQuality", category: "Scalability" });
    expect(paramRowKey(p)).toBe("GameUserSettings.ini-ScalabilityGroups-sg.ShadowQuality");
  });
});
