import { describe, expect, it, beforeEach } from "vitest";
import {
  filterParamsByPanel,
  filterParamsByMode,
  isRecommendedParam,
  panelForParameter,
  panelFromHash,
  readStoredPanel,
  writeStoredPanel,
} from "./editorPanels";
import type { GameParameter } from "../core/types";

function param(
  overrides: Partial<GameParameter> & Pick<GameParameter, "key">,
): GameParameter {
  return {
    section: "SystemSettings",
    file: "Engine.ini",
    category: "Rendering",
    value: "1",
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

describe("panelForParameter", () => {
  it("GUS sg.* goes to basic", () => {
    expect(
      panelForParameter(
        param({ key: "sg.ShadowQuality", file: "GameUserSettings.ini", category: "Scalability" }),
      ),
    ).toBe("basic");
  });

  it("standard GUS display fields go to basic", () => {
    expect(
      panelForParameter(
        param({
          key: "bUseVSync",
          file: "GameUserSettings.ini",
          section: "/Script/Engine.GameUserSettings",
          category: "Display",
        }),
      ),
    ).toBe("basic");
  });

  it("game-specific GUS sections go to basic", () => {
    expect(
      panelForParameter(
        param({
          key: "DLSSMode",
          file: "GameUserSettings.ini",
          section: "/Script/Game.UserSettings",
          category: "Rendering",
        }),
      ),
    ).toBe("basic");
  });

  it("Engine.ini r.* goes to advanced", () => {
    expect(panelForParameter(param({ key: "r.ViewDistanceScale" }))).toBe("advanced");
  });

  it("Scalability.ini r.* goes to advanced", () => {
    expect(
      panelForParameter(
        param({ key: "r.ShadowQuality", file: "Scalability.ini", category: "Shadows" }),
      ),
    ).toBe("advanced");
  });

  it("Scalability.ini sg.* stays advanced", () => {
    expect(
      panelForParameter(
        param({ key: "sg.ShadowQuality", file: "Scalability.ini", category: "Scalability" }),
      ),
    ).toBe("advanced");
  });
});

describe("filterParamsByPanel", () => {
  it("splits basic GUS and advanced engine params", () => {
    const items = [
      param({ key: "sg.ShadowQuality", file: "GameUserSettings.ini", category: "Scalability" }),
      param({ key: "r.Fog", category: "Rendering" }),
    ];
    expect(filterParamsByPanel(items, "basic")).toHaveLength(1);
    expect(filterParamsByPanel(items, "advanced")).toHaveLength(1);
  });
});

describe("isRecommendedParam", () => {
  it("recommends all sg keys in basic", () => {
    expect(
      isRecommendedParam(
        param({
          key: "sg.ExperimentalQuality",
          file: "GameUserSettings.ini",
          category: "Scalability",
          known: false,
          catalog_recommended: false,
        }),
        "basic",
      ),
    ).toBe(true);
  });

  it("advanced recommends only explicit catalog flags", () => {
    expect(isRecommendedParam(param({ key: "r.Custom" }), "advanced")).toBe(false);
    expect(
      isRecommendedParam(param({ key: "r.Custom", catalog_recommended: true }), "advanced"),
    ).toBe(true);
  });
});

describe("filterParamsByMode (recommended)", () => {
  it("basic recommended mode keeps sg and GUS display", () => {
    const items = [
      param({ key: "sg.ShadowQuality", file: "GameUserSettings.ini", category: "Scalability" }),
      param({ key: "bUseVSync", file: "GameUserSettings.ini", category: "Display" }),
      param({
        key: "DLSSMode",
        file: "GameUserSettings.ini",
        category: "Rendering",
        present_in_ini: true,
        known: false,
      }),
    ];
    const filtered = filterParamsByMode(items, "recommended", "basic", "");
    expect(filtered.map((p) => p.key)).toEqual(["sg.ShadowQuality", "bUseVSync"]);
  });

  it("ini_only mode keeps present keys only", () => {
    const items = [
      param({ key: "r.Obscure", known: false, present_in_ini: true }),
      param({ key: "r.Absent", known: false, present_in_ini: false }),
    ];
    const filtered = filterParamsByMode(items, "ini_only", "advanced", "");
    expect(filtered).toHaveLength(1);
    expect(filtered[0].key).toBe("r.Obscure");
  });

  it("full catalog mode keeps catalog recommended and all injected keys", () => {
    const items = [
      param({
        key: "r.Curated",
        present_in_ini: false,
        catalog_recommended: true,
      }),
      param({
        key: "r.Obscure",
        known: false,
        present_in_ini: false,
        catalog_recommended: false,
      }),
    ];
    const filtered = filterParamsByMode(items, "full", "advanced", "");
    expect(filtered.map((p) => p.key)).toEqual(["r.Curated", "r.Obscure"]);
  });

  it("advanced panel not empty when only catalog-injected engine keys exist", () => {
    const items = [
      param({
        key: "r.ViewDistanceScale",
        present_in_ini: false,
        catalog_recommended: true,
      }),
      param({
        key: "fx.AmbientOcclusion.Enable",
        present_in_ini: false,
        catalog_recommended: true,
      }),
    ];
    const panel = filterParamsByPanel(items, "advanced");
    const visible = filterParamsByMode(panel, "full", "advanced", "");
    expect(visible.length).toBeGreaterThanOrEqual(2);
    expect(visible.every((p) => !p.present_in_ini)).toBe(true);
  });

  it("basic recommended mode shows catalog sg without present_in_ini", () => {
    const items = [
      param({
        key: "sg.ViewDistanceQuality",
        file: "GameUserSettings.ini",
        category: "Scalability",
        section: "ScalabilityGroups",
        present_in_ini: false,
        catalog_recommended: true,
      }),
      param({
        key: "bUseVSync",
        file: "GameUserSettings.ini",
        category: "Display",
        section: "/Script/Engine.GameUserSettings",
        present_in_ini: false,
        catalog_recommended: true,
      }),
    ];
    const filtered = filterParamsByMode(items, "recommended", "basic", "");
    expect(filtered.map((p) => p.key)).toEqual([
      "sg.ViewDistanceQuality",
      "bUseVSync",
    ]);
  });

  it("full catalog mode shows reference-only keys not in ini", () => {
    const items = [
      param({
        key: "r.Shadow.MaxResolution",
        present_in_ini: false,
        catalog_recommended: false,
        known: true,
      }),
      param({
        key: "r.Obscure",
        present_in_ini: false,
        catalog_recommended: false,
        known: true,
      }),
    ];
    const recommended = filterParamsByMode(items, "recommended", "advanced", "");
    expect(recommended).toHaveLength(0);
    const full = filterParamsByMode(items, "full", "advanced", "");
    expect(full).toHaveLength(2);
  });

  it("advanced full catalog panel not empty with catalog-only engine keys", () => {
    const items = Array.from({ length: 350 }, (_, i) =>
      param({
        key: `r.TestParam${i}`,
        present_in_ini: false,
        catalog_recommended: i % 3 === 0,
        known: true,
      }),
    );
    const panel = filterParamsByPanel(items, "advanced");
    const full = filterParamsByMode(panel, "full", "advanced", "");
    expect(full.length).toBeGreaterThan(300);
  });
});

describe("panel migration", () => {
  beforeEach(() => {
    sessionStorage.clear();
    window.history.replaceState(null, "", "/");
  });

  it("migrates legacy storage scalability to basic", () => {
    sessionStorage.setItem("gsm-advanced-panel:game-1", "scalability");
    expect(readStoredPanel("game-1")).toBe("basic");
  });

  it("migrates legacy hash engine to advanced", () => {
    window.history.replaceState(null, "", "/#engine");
    expect(panelFromHash()).toBe("advanced");
  });

  it("writes new storage ids", () => {
    writeStoredPanel("game-1", "advanced");
    expect(sessionStorage.getItem("gsm-editor-panel:game-1")).toBe("advanced");
  });
});
