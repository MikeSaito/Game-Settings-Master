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
  it("encodes game id", () => {
    expect(gameTabPath("steam:123", "advanced")).toBe("/game/steam%3A123/advanced");
  });
});

describe("parseGameRoute", () => {
  it("parses advanced route", () => {
    expect(parseGameRoute("/game/foo/advanced")).toEqual({
      gameId: "foo",
      tab: "advanced",
    });
  });

  it("returns null for library", () => {
    expect(parseGameRoute("/library")).toBeNull();
  });

  it("returns null for legacy tabs", () => {
    expect(parseGameRoute("/game/foo/wizard")).toBeNull();
    expect(parseGameRoute("/game/foo/reshade")).toBeNull();
    expect(parseGameRoute("/game/foo/backups")).toBeNull();
  });

  it("parses legacy routes for redirect", () => {
    expect(parseLegacyGameRoute("/game/foo/wizard")).toEqual({ gameId: "foo" });
    expect(parseLegacyGameRoute("/game/foo/reshade")).toEqual({ gameId: "foo" });
    expect(parseLegacyGameRoute("/game/foo/backups")).toEqual({ gameId: "foo" });
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

  it("maps advanced game path", () => {
    expect(tabFromPathname("/game/id/advanced")).toBe("advanced");
  });

  it("treats legacy backups URL as library until redirect", () => {
    expect(tabFromPathname("/game/id/backups")).toBe("library");
  });
});
