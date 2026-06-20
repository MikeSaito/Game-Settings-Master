import { invoke } from "@tauri-apps/api/core";
import type {
  ApplyResult,
  BackupInfo,
  ConfigResetResult,
  GameConfig,
  GameOverride,
  GameParameter,
  GameProfile,
  GpuCapabilities,
  LaunchResult,
  ScreenResolution,
  ScalabilityLimits,
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

export function setBackendLanguage(lang: string): Promise<void> {
  return invoke("set_language_cmd", { lang });
}

export function setAppBackgroundMode(background: boolean): Promise<void> {
  return invoke("set_app_background_mode_cmd", { background });
}

export function closeGame(exeName: string): Promise<void> {
  return invoke("close_game_cmd", { exeName });
}

export function scanGames(): Promise<GameProfile[]> {
  return invoke("scan_games");
}

export function getGameConfig(
  configDir: string,
  gameId?: string,
  engineFamily?: string,
): Promise<GameConfig> {
  return invoke("get_game_config", {
    configDir,
    gameId: gameId ?? null,
    engineFamily: engineFamily ?? null,
  });
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
  gameId?: string,
): Promise<ScalabilityLimits> {
  return invoke("get_scalability_limits_cmd", {
    configDir,
    installDir: installDir ?? null,
    gameId: gameId ?? null,
  });
}

export function applyCustom(
  configDir: string,
  files: Record<string, Record<string, Record<string, string>>>,
  exeName?: string,
  removals?: Record<string, Record<string, string[]>>,
  gameId?: string,
  engineFamily?: string,
): Promise<ApplyResult> {
  return invoke("apply_custom_cmd", {
    configDir,
    changes: { files, removals: removals ?? {} },
    exeName: exeName ?? null,
    gameId: gameId ?? null,
    engineFamily: engineFamily ?? null,
  });
}

export function listBackups(
  configDir: string,
  gameId?: string,
): Promise<BackupInfo[]> {
  return invoke("list_backups_cmd", {
    configDir,
    gameId: gameId ?? null,
  });
}

export function restoreBackup(
  configDir: string,
  backupId: string,
  exeName?: string,
  gameId?: string,
  engineFamily?: string,
  installDir?: string,
): Promise<string[]> {
  return invoke("restore_backup_cmd", {
    configDir,
    backupId,
    exeName: exeName ?? null,
    gameId: gameId ?? null,
    engineFamily: engineFamily ?? null,
    installDir: installDir ?? null,
  });
}

export function resetConfigToUser(
  configDir: string,
  exeName?: string,
  gameId?: string,
  engineFamily?: string,
): Promise<ConfigResetResult> {
  return invoke("reset_config_to_user_cmd", {
    configDir,
    exeName: exeName ?? null,
    gameId: gameId ?? null,
    engineFamily: engineFamily ?? null,
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

export function openConfigFolder(
  configDir: string,
  gameId?: string,
): Promise<void> {
  return invoke("open_config_folder", {
    configDir,
    gameId: gameId ?? null,
  });
}

export function launchGame(profile: GameProfile): Promise<LaunchResult> {
  return invoke("launch_game_cmd", { profile });
}
