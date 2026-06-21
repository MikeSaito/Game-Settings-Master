import { describe, expect, it } from "vitest";
import {
  buildCategoryList,
  filterParamsByCategoryAndSearch,
  normalizeParameterCategory,
  paramRowKey,
} from "./advancedEditorFilters";
import {
  filterParamsByPanel,
  filterParamsByRecommendedMode,
} from "./editorPanels";
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
    catalog_recommended: false,
    tier_hint: null,
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
    expect(list[0].cat).toBe("All");
    expect(list[1].cat).toBe("Scalability");
    expect(list.some((c) => c.cat === "Other")).toBe(true);
  });
});

describe("normalizeParameterCategory", () => {
  it("moves game-specific frame generation into rendering", () => {
    const normalized = normalizeParameterCategory(
      param({
        key: "UpscalingFrameGeneration",
        category: "Subnautica2",
        section: "/Script/Subnautica2.SN2SettingsLocal",
      }),
    );

    expect(normalized.category).toBe("Rendering");
  });

  it("moves legacy author-curated category into game", () => {
    const normalized = normalizeParameterCategory(
      param({ key: "GammaValue", category: "AuthorCurated" }),
    );

    expect(normalized.category).toBe("GameSpecific");
  });
});

describe("panel + recommended integration", () => {
  it("basic panel recommended filter keeps sg keys", () => {
    const items = [
      param({ key: "sg.ShadowQuality", category: "Scalability" }),
      param({ key: "r.Fog", category: "Rendering", file: "Engine.ini", known: false, present_in_ini: true }),
    ];
    const panelItems = filterParamsByPanel(items, "basic");
    const filtered = filterParamsByRecommendedMode(panelItems, true, "basic", "");
    expect(filtered.some((p) => p.key === "sg.ShadowQuality")).toBe(true);
    expect(filtered.some((p) => p.key === "r.Fog")).toBe(false);
  });
});

describe("paramRowKey", () => {
  it("is stable per file/section/key", () => {
    const p = param({ key: "sg.ShadowQuality", category: "Scalability" });
    expect(paramRowKey(p)).toBe("GameUserSettings.ini-ScalabilityGroups-sg.ShadowQuality");
  });
});
