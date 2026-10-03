import { beforeEach, describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { testGame } from "@/test/fixtures/gameProfile";
import { parsePreset, presetCompatibility } from "./presetCompatibility";
import type { GpuCapabilities } from "@/lib/core";

describe("preset compatibility and provenance", () => {
  beforeEach(async () => {
    await i18n.changeLanguage("en");
  });
  it("reads legacy edits and preserves the original game after local import", () => {
    const preset = parsePreset(
      JSON.stringify({
        game_id: "old-game",
        name: " Legacy ",
        files: { "Game.ini": { A: { key: "" } } },
      }),
      testGame.id,
    );
    expect(preset.game_id).toBe(testGame.id);
    expect(preset.name).toBe("Legacy");
    expect(preset.metadata?.source_game_id).toBe("old-game");
    expect(preset.metadata?.mode).toBe("changes");
    expect(preset.files["Game.ini"].A.key).toBe("");
    expect(presetCompatibility(preset, testGame)).toContain(
      "Preset was created for another game: old-game",
    );
  });
  it("keeps v2 metadata and per-occurrence input updates through import", () => {
    const metadata = {
      format_version: 2,
      mode: "profile",
      description: "Controls",
      source_game_id: "source",
      source_game_name: "Source",
      game_build: "42",
      engine_family: "ue4",
      engine_version: "4.27",
      gpu_vendor: "amd",
      min_dedicated_memory_mb: 8192,
      requires_ray_tracing: true,
    };
    const input_updates = [{ line: 4, expected: "(Key=W)", value: "(Key=Up)" }];
    const preset = parsePreset(
      JSON.stringify({
        game_id: "source",
        name: "Profile",
        files: {},
        input_updates,
        metadata,
      }),
      testGame.id,
    );
    expect(preset.metadata).toEqual(metadata);
    expect(preset.input_updates).toEqual(input_updates);
    const gpu: GpuCapabilities = {
      vendor: "intel",
      name: "Intel",
      supports_dlss: false,
      supports_dlss_fg: false,
      supports_ray_tracing: false,
      ray_tracing_status: "unknown",
      dedicated_memory_mb: 4096,
    };
    expect(presetCompatibility(preset, testGame, gpu)).toHaveLength(7);
  });
  it("rejects unsupported versions and malformed preset data", () => {
    expect(() =>
      parsePreset(
        '{"name":"x","files":[],"metadata":{"format_version":2}}',
        testGame.id,
      ),
    ).toThrow();
    expect(() =>
      parsePreset(
        '{"name":"x","files":{},"metadata":{"format_version":99}}',
        testGame.id,
      ),
    ).toThrow();
  });
});
