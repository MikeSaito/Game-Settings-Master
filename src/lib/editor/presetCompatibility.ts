import i18n from "@/i18n";
import type { GameOverride, GameProfile, GpuCapabilities } from "@/lib/core";

export function presetCompatibility(
  preset: GameOverride,
  game: GameProfile,
  gpu?: GpuCapabilities,
): string[] {
  const meta = preset.metadata;
  const warnings: string[] = [];
  const sourceId = meta?.source_game_id ?? preset.game_id;
  if (sourceId && sourceId !== game.id)
    warnings.push(
      i18n.t("improvements:presets.otherGame", {
        name: meta?.source_game_name ?? sourceId,
      }),
    );
  if (!meta) warnings.push(i18n.t("improvements:presets.legacy"));
  if (meta?.engine_family && meta.engine_family !== game.engine_family)
    warnings.push(i18n.t("improvements:presets.engineMismatch"));
  if (meta?.engine_version && meta.engine_version !== game.engine_version)
    warnings.push(i18n.t("improvements:presets.engineVersionMismatch"));
  if (meta?.game_build && meta.game_build !== game.build_id)
    warnings.push(i18n.t("improvements:presets.buildMismatch"));
  if (meta?.gpu_vendor && meta.gpu_vendor !== gpu?.vendor)
    warnings.push(i18n.t("improvements:presets.gpuMismatch"));
  if (
    meta?.requires_ray_tracing &&
    (!gpu || gpu.ray_tracing_status === "unknown" || !gpu.supports_ray_tracing)
  )
    warnings.push(i18n.t("improvements:presets.rtMismatch"));
  if (
    meta?.min_dedicated_memory_mb &&
    (gpu?.dedicated_memory_mb == null ||
      gpu.dedicated_memory_mb < meta.min_dedicated_memory_mb)
  )
    warnings.push(
      i18n.t("improvements:presets.memoryMismatch", {
        memory: meta.min_dedicated_memory_mb,
      }),
    );
  return warnings;
}

export function parsePreset(raw: string, gameId: string): GameOverride {
  if (new TextEncoder().encode(raw).length > 512 * 1024)
    throw new Error(i18n.t("improvements:presets.invalid"));
  const data = JSON.parse(raw) as GameOverride;
  if (
    !data ||
    typeof data.name !== "string" ||
    !data.name.trim() ||
    !data.files ||
    typeof data.files !== "object" ||
    Array.isArray(data.files)
  )
    throw new Error(i18n.t("improvements:presets.invalid"));
  if (data.metadata && data.metadata.format_version !== 2)
    throw new Error(i18n.t("improvements:presets.invalid"));
  // Local ownership changes; original source identity remains explicit.
  return {
    ...data,
    game_id: gameId,
    name: data.name.trim(),
    metadata: data.metadata ?? {
      format_version: 2,
      mode: "changes",
      description: "",
      source_game_id: data.game_id ?? "",
      source_game_name: data.game_id ?? "",
      game_build: null,
      engine_family: null,
      engine_version: null,
      gpu_vendor: null,
      min_dedicated_memory_mb: null,
      requires_ray_tracing: false,
    },
  };
}
