import { describe, expect, it } from "vitest";
import { resolveGameTabRoute } from "./gameEngine";
import { testGame } from "../../test/fixtures/gameProfile";

describe("resolveGameTabRoute", () => {
  it("returns advanced for UE game with config_dir", () => {
    expect(resolveGameTabRoute(testGame)).toBe("advanced");
  });

  it("returns null when game is not UE", () => {
    expect(resolveGameTabRoute({ ...testGame, is_ue: false })).toBeNull();
  });

  it("returns null when config_dir is missing", () => {
    expect(resolveGameTabRoute({ ...testGame, config_dir: null })).toBeNull();
  });
});
