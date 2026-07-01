/**
 * UE config ini filenames — keep in sync with
 * `src-tauri/src/fs_util/path_safety.rs` (ALLOWED_CONFIG_INI_FILES, OVERRIDE_INI_FILES).
 */
export const GAME_USER_SETTINGS_INI = "GameUserSettings.ini";

export const CONFIG_INI_FILES = [
  GAME_USER_SETTINGS_INI,
  "Engine.ini",
  "Game.ini",
  "Scalability.ini",
  "Input.ini",
  "DeviceProfiles.ini",
] as const;

/** Removed on backup reset; GameUserSettings.ini is kept. */
export const OVERRIDE_INI_FILES = [
  "Engine.ini",
  "Game.ini",
  "Scalability.ini",
  "Input.ini",
  "DeviceProfiles.ini",
] as const;

export type ConfigIniFile = (typeof CONFIG_INI_FILES)[number];
export type OverrideIniFile = (typeof OVERRIDE_INI_FILES)[number];

/** Human-readable list for UI copy (no GameUserSettings.ini). */
export const OVERRIDE_INI_FILES_LABEL = OVERRIDE_INI_FILES.join(", ");

/** True when no engine override ini files are present in config snapshot. */
export function isUserOnlyConfig(
  files: Record<string, unknown> | undefined,
): boolean {
  if (!files) return false;
  return !OVERRIDE_INI_FILES.some((file) => file in files);
}
