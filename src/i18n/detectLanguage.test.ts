import { afterEach, describe, expect, it, vi } from "vitest";
import { detectSystemLanguage, languageFromTag } from "./detectLanguage";

describe("detectLanguage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("maps Russian tags to ru", () => {
    expect(languageFromTag("ru-RU")).toBe("ru");
    expect(languageFromTag("ru")).toBe("ru");
  });

  it("maps non-Russian tags to en", () => {
    expect(languageFromTag("en-US")).toBe("en");
    expect(languageFromTag("de-DE")).toBe("en");
    expect(languageFromTag("ja")).toBe("en");
  });

  it("detects Russian from navigator.languages", () => {
    vi.stubGlobal("navigator", { language: "en-US", languages: ["ru-RU", "en-US"] });
    expect(detectSystemLanguage()).toBe("ru");
  });

  it("detects English from navigator.language", () => {
    vi.stubGlobal("navigator", { language: "en-GB", languages: ["en-GB"] });
    expect(detectSystemLanguage()).toBe("en");
  });

  it("defaults to English for unsupported locales", () => {
    vi.stubGlobal("navigator", { language: "de-DE", languages: ["de-DE", "fr-FR"] });
    expect(detectSystemLanguage()).toBe("en");
  });
});
