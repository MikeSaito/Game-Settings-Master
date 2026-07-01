import { describe, expect, it } from "vitest";
import {
  CONFIG_INI_FILES,
  GAME_USER_SETTINGS_INI,
  isUserOnlyConfig,
  OVERRIDE_INI_FILES,
} from "./configFiles";

describe("configFiles", () => {
  it("override list excludes GameUserSettings only", () => {
    expect(OVERRIDE_INI_FILES).not.toContain(GAME_USER_SETTINGS_INI);
    expect(new Set(OVERRIDE_INI_FILES)).toEqual(
      new Set(CONFIG_INI_FILES.filter((f) => f !== GAME_USER_SETTINGS_INI)),
    );
  });

  it("includes DeviceProfiles in override detection", () => {
    expect(
      isUserOnlyConfig({
        [GAME_USER_SETTINGS_INI]: {},
        "DeviceProfiles.ini": {},
      }),
    ).toBe(false);
  });

  it("is user-only when only GameUserSettings exists", () => {
    expect(isUserOnlyConfig({ [GAME_USER_SETTINGS_INI]: {} })).toBe(true);
    expect(isUserOnlyConfig(undefined)).toBe(false);
  });
});
