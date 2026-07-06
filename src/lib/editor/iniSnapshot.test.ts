import { describe, expect, it } from "vitest";
import type { GameParameter } from "@/lib/core";
import { buildIniSnapshot, iniSnapshotKey, iniSnapshotKeyFromParts, isIniShippedKey } from "./iniSnapshot";
import { isIniMembershipToggleable } from "./engineParams";

function param(overrides: Partial<GameParameter>): GameParameter {
  return {
    key: "sg.ShadowQuality",
    section: "ScalabilityGroups",
    file: "GameUserSettings.ini",
    value: "2",
    title: "Shadows",
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
    catalog_recommended: false,
    tier_hint: null,
    ...overrides,
  };
}

describe("iniSnapshot", () => {
  it("captures only present_in_ini keys", () => {
    const items = [
      param({ key: "sg.ShadowQuality", present_in_ini: true }),
      param({ key: "bUseVSync", section: "/Script/Engine.GameUserSettings", present_in_ini: false }),
    ];
    const snapshot = buildIniSnapshot(items);
    expect(snapshot.size).toBe(1);
    expect(isIniShippedKey(items[0], snapshot)).toBe(true);
    expect(isIniShippedKey(items[1], snapshot)).toBe(false);
  });

  it("blocks remove toggle for shipped GUS keys", () => {
    const p = param({ key: "sg.ShadowQuality", present_in_ini: true });
    const snapshot = buildIniSnapshot([p]);
    expect(isIniMembershipToggleable(p, snapshot)).toBe(false);
    expect(isIniMembershipToggleable(
      param({ key: "bUseVSync", section: "/Script/Engine.GameUserSettings", present_in_ini: false }),
      snapshot,
    )).toBe(true);
  });

  it("uses stable snapshot keys", () => {
    const p = param({ key: "sg.ShadowQuality" });
    expect(iniSnapshotKey(p)).toBe("gameusersettings.ini|scalabilitygroups|sg.shadowquality");
  });

  it("normalizes bracketed ini sections in snapshot keys", () => {
    expect(
      iniSnapshotKeyFromParts(
        "GameUserSettings.ini",
        "[ScalabilityGroups]",
        "sg.ShadowQuality",
      ),
    ).toBe("gameusersettings.ini|scalabilitygroups|sg.shadowquality");
  });
});
