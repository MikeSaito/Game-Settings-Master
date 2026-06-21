import { describe, expect, it } from "vitest";
import {
  gameTabPath,
  isLibraryRoute,
  libraryPath,
  parseGameRoute,
  parseLegacyGameRoute,
  tabFromPathname,
} from "./routes";

describe("libraryPath", () => {
  it("returns /library", () => {
    expect(libraryPath()).toBe("/library");
  });
});

describe("gameTabPath", () => {
  it("encodes game id and tab", () => {
    expect(gameTabPath("steam:123", "advanced")).toBe("/game/steam%3A123/advanced");
  });
});

describe("parseGameRoute", () => {
  it("parses valid game routes", () => {
    expect(parseGameRoute("/game/foo/advanced")).toEqual({
      gameId: "foo",
      tab: "advanced",
    });
    expect(parseGameRoute("/game/foo/backups")).toEqual({
      gameId: "foo",
      tab: "backups",
    });
  });

  it("returns null for library", () => {
    expect(parseGameRoute("/library")).toBeNull();
  });

  it("returns null for removed tabs", () => {
    expect(parseGameRoute("/game/foo/wizard")).toBeNull();
    expect(parseGameRoute("/game/foo/reshade")).toBeNull();
  });

  it("parses legacy wizard/reshade routes for redirect", () => {
    expect(parseLegacyGameRoute("/game/foo/wizard")).toEqual({ gameId: "foo" });
    expect(parseLegacyGameRoute("/game/foo/reshade")).toEqual({ gameId: "foo" });
    expect(parseLegacyGameRoute("/game/foo/advanced")).toBeNull();
  });

  it("returns null for unknown tab", () => {
    expect(parseGameRoute("/game/foo/unknown")).toBeNull();
  });
});

describe("tabFromPathname", () => {
  it("maps library paths", () => {
    expect(tabFromPathname("/")).toBe("library");
    expect(tabFromPathname("/library")).toBe("library");
    expect(isLibraryRoute("/library")).toBe(true);
  });

  it("maps game tab paths", () => {
    expect(tabFromPathname("/game/id/backups")).toBe("backups");
  });
});
