import { describe, expect, it } from "vitest";
import { updateTextDraft } from "./textDraft";
describe("textarea source endings", () => {
  it("preserves mixed endings outside the edited range", () => {
    const source = "; a\r\n[X]\nKey=1\r\nOther=2\n";
    expect(updateTextDraft(source, "; a\n[X]\nKey=3\nOther=2\n")).toBe(
      "; a\r\n[X]\nKey=3\r\nOther=2\n",
    );
    expect(updateTextDraft(source, source.replace(/\r\n/g, "\n"))).toBe(source);
  });
  it("keeps CRLF for pasted lines and edits with unicode", () => {
    expect(updateTextDraft("[X]\r\nKey=Я🎮\r\n", "[X]\nKey=Я🎮!\nNew=\n")).toBe(
      "[X]\r\nKey=Я🎮!\r\nNew=\r\n",
    );
    expect(updateTextDraft("a\r\nb\r\nc", "a\nc")).toBe("a\r\nc");
    expect(updateTextDraft("a\nb", "a\nb\nc")).toBe("a\nb\nc");
  });
});
