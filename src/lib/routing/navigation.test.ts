import { describe, expect, it, vi } from "vitest";
import { goToLibrary } from "./navigation";

describe("goToLibrary", () => {
  it("navigates to /library and clears hash", () => {
    const navigate = vi.fn();
    goToLibrary(navigate, {
      pathname: "/game/foo/advanced",
      search: "",
      hash: "#backups",
      state: null,
      key: "abc",
    });
    expect(navigate).toHaveBeenCalledWith({ pathname: "/library", search: "", hash: "" });
  });

  it("scrolls to top when already on library", () => {
    const navigate = vi.fn();
    const scrollTo = vi.fn();
    vi.stubGlobal("scrollTo", scrollTo);
    goToLibrary(navigate, {
      pathname: "/library",
      search: "",
      hash: "",
      state: null,
      key: "abc",
    });
    expect(navigate).not.toHaveBeenCalled();
    expect(scrollTo).toHaveBeenCalled();
    vi.unstubAllGlobals();
  });
});
