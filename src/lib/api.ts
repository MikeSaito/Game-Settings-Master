import { invoke } from "@tauri-apps/api/core";
import type {
  ApplyResult,
  BackupInfo,
  ConfigDiffEntry,
  ConfigResetResult,
  GameConfig,
  GameOverride,
  GameParameter,
  GameProfile,
  GpuCapabilities,
  LaunchResult,
  ScreenResolution,
  PresetInfo,
  PresetServerConfig,
  RemotePresetStatus,
  ScalabilityLimits,
  SyncPresetsReport,
} from "./types";

export function getGpuInfo(): Promise<GpuCapabilities> {
  return invoke("get_gpu_info_cmd");
}

export function getDesktopResolution(): Promise<ScreenResolution> {
  return invoke("get_desktop_resolution_cmd");
}

export function isGameRunning(exeName: string): Promise<boolean> {
  return invoke("is_game_running_cmd", { exeName });
}

export function closeGame(exeName: string): Promise<void> {
  return invoke("close_game_cmd", { exeName });
}

export function scanGames(): Promise<GameProfile[]> {
  return invoke("scan_games");
}

export function getGameConfig(configDir: string): Promise<GameConfig> {
  return invoke("get_game_config", { configDir });
}

export function getGameParameters(
  configDir: string,
  gameId?: string,
  installDir?: string,
  engineFamily?: string,
): Promise<GameParameter[]> {
  return invoke("get_game_parameters_cmd", {
    configDir,
    gameId: gameId ?? null,
    installDir: installDir ?? null,
    engineFamily: engineFamily ?? null,
  });
}

export function getScalabilityLimits(
  configDir: string,
  installDir?: string,
): Promise<ScalabilityLimits> {
  return invoke("get_scalability_limits_cmd", {
    configDir,
    installDir: installDir ?? null,
  });
}

export function listPresets(
  engineFamily?: string,
  gameId?: string,
): Promise<PresetInfo[]> {
  return invoke("list_presets_cmd", {
    engineFamily: engineFamily ?? null,
    gameId: gameId ?? null,
  });
}

export function previewPreset(
  configDir: string,
  presetId: string,
  gameId?: string,
  installDir?: string,
  engineFamily?: string,
): Promise<ConfigDiffEntry[]> {
  return invoke("preview_preset_cmd", {
    configDir,
    presetId,
    gameId: gameId ?? null,
    installDir: installDir ?? null,
    engineFamily: engineFamily ?? null,
  });
}

export function applyPreset(
  configDir: string,
  presetId: string,
  gameId?: string,
  installDir?: string,
  exeName?: string,
  engineFamily?: string,
): Promise<ApplyResult> {
  return invoke("apply_preset_cmd", {
    configDir,
    presetId,
    gameId: gameId ?? null,
    installDir: installDir ?? null,
    exeName: exeName ?? null,
    engineFamily: engineFamily ?? null,
  });
}

export function applyCustom(
  configDir: string,
  files: Record<string, Record<string, Record<string, string>>>,
  exeName?: string,
  removals?: Record<string, Record<string, string[]>>,
): Promise<ApplyResult> {
  return invoke("apply_custom_cmd", {
    configDir,
    changes: { files, removals: removals ?? {} },
    exeName: exeName ?? null,
  });
}

export function listBackups(configDir: string): Promise<BackupInfo[]> {
  return invoke("list_backups_cmd", { configDir });
}

export function restoreBackup(
  configDir: string,
  backupId: string,
  exeName?: string,
): Promise<string[]> {
  return invoke("restore_backup_cmd", {
    configDir,
    backupId,
    exeName: exeName ?? null,
  });
}

export function resetConfigToUser(
  configDir: string,
  exeName?: string,
): Promise<ConfigResetResult> {
  return invoke("reset_config_to_user_cmd", {
    configDir,
    exeName: exeName ?? null,
  });
}

export function addManualGame(
  name: string,
  installDir: string,
): Promise<GameProfile> {
  return invoke("add_manual_game", { name, installDir });
}

export function setGameConfigDir(
  gameId: string,
  configDir: string,
): Promise<GameProfile> {
  return invoke("set_game_config_dir", { gameId, configDir });
}

export function resolveConfigFromPath(
  installDir: string,
): Promise<string | null> {
  return invoke("resolve_config_from_path", { installDir });
}

export function saveGameProfile(profile: GameProfile): Promise<void> {
  return invoke("save_game_profile", { profile });
}

export function removeGameProfile(id: string): Promise<void> {
  return invoke("remove_game_profile", { id });
}

export function importGameCover(gameId: string, imagePath: string): Promise<GameProfile> {
  return invoke("import_game_cover_cmd", { gameId, imagePath });
}

export function removeGameCover(gameId: string): Promise<GameProfile> {
  return invoke("remove_game_cover_cmd", { gameId });
}

export function saveGameOverride(override: GameOverride): Promise<void> {
  return invoke("save_game_override", { overrideDef: override });
}

export function getGameOverrides(gameId: string): Promise<GameOverride[]> {
  return invoke("get_game_overrides", { gameId });
}

export function deleteGameOverride(
  gameId: string,
  name: string,
): Promise<void> {
  return invoke("delete_game_override", { gameId, name });
}

export function applyGameOverride(
  configDir: string,
  override: GameOverride,
  exeName?: string,
): Promise<ApplyResult> {
  return invoke("apply_game_override", {
    configDir,
    overrideDef: override,
    exeName: exeName ?? null,
  });
}

export function openConfigFolder(configDir: string): Promise<void> {
  return invoke("open_config_folder", { configDir });
}

export function launchGame(profile: GameProfile): Promise<LaunchResult> {
  return invoke("launch_game_cmd", { profile });
}

export function getPresetServerStatus(): Promise<RemotePresetStatus> {
  return invoke("get_preset_server_status_cmd");
}

export function setPresetServerUrl(
  baseUrl: string | null,
): Promise<PresetServerConfig> {
  return invoke("set_preset_server_url_cmd", { baseUrl });
}

export function syncPresets(force = false): Promise<SyncPresetsReport> {
  return invoke("sync_presets_cmd", { force });
}
