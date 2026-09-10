import { describe, expect, it } from "vitest";
import { formatBackupDate } from "./formatBackupDate";

describe("formatBackupDate", () => {
  it("formats legacy and unique backup ids", () => {
    expect(formatBackupDate("20260910_163045")).toBe("10.09.2026 · 16:30:45");
    expect(formatBackupDate("20260910_163045_aabbccdd")).toBe("10.09.2026 · 16:30:45");
  });
});
