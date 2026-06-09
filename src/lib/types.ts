export interface LaunchResult {
  launcher: string;
  detail: string;
}

export interface ScreenResolution {
  width: number;
  height: number;
}

export interface GameProfile {
  id: string;
  name: string;
  source: "steam" | "epic" | "manual" | string;
  install_dir: string;
  config_dir: string | null;
  exe_name: string | null;
  is_ue: boolean;
  is_unity?: boolean;
  is_author_curated?: boolean;
  possible_unity?: boolean;
  possible_ue?: boolean;
  cover_url?: string | null;
  custom_cover?: string | null;
  build_id?: string | null;
  engine_family?: "ue4" | "ue5" | "unity" | "unknown" | string;
  engine_version?: string | null;
}

export interface IniFileData {
  sections: Record<string, Record<string, string>>;
}

export interface GameConfig {
  config_dir: string;
  files: Record<string, IniFileData>;
}

export interface ScalabilityLimits {
  groups: Record<string, number>;
  global_max: number;
  sources: string[];
}

export interface GameParameter {
  key: string;
  section: string;
  file: string;
  value: string;
  title: string;
  description: string;
  impact: string;
  category: string;
  min: string | null;
  max: string | null;
  in_game_label: string | null;
  value_hint: string | null;
  value_type: string;
  known: boolean;
  editable: boolean;
  present_in_ini: boolean;
  default_value: string | null;
}

export interface PresetInfo {
  id: string;
  name: string;
  description: string;
}

export interface ConfigDiffEntry {
  file: string;
  section: string;
  key: string;
  old_value: string | null;
  new_value: string;
}

export interface ApplyResult {
  backup_id: string;
  changed_files: string[];
  diff: ConfigDiffEntry[];
  effective_config_dir?: string | null;
}

export interface BackupInfo {
  id: string;
  created_at: string;
  files: string[];
}

export interface ConfigResetResult {
  backup_id: string;
  deleted_files: string[];
}

export interface GameOverride {
  game_id: string;
  name: string;
  files: Record<string, Record<string, Record<string, string>>>;
  /** Ключи Engine.ini для удаления при применении пресета. */
  removals?: Record<string, Record<string, string[]>>;
}

export type AppTab = "library" | "wizard" | "advanced" | "backups";

export interface PresetDefinition {
  id: string;
  name: string;
  description: string;
  files: Record<string, Record<string, Record<string, string>>>;
}

export interface GpuCapabilities {
  name: string;
  vendor: string;
  supports_dlss: boolean;
  supports_dlss_fg: boolean;
  supports_ray_tracing: boolean;
}

export interface PresetServerConfig {
  base_url?: string | null;
  auto_sync?: boolean;
  last_sync_at?: string | null;
  last_sync_ok?: boolean;
  last_sync_error?: string | null;
  catalog_version?: string | null;
}

export interface RemotePresetStatus {
  configured: boolean;
  base_url?: string | null;
  auto_sync: boolean;
  last_sync_at?: string | null;
  last_sync_ok: boolean;
  last_sync_error?: string | null;
  catalog_version?: string | null;
  cached_packs: string[];
}

export interface SyncPresetsReport {
  ok: boolean;
  message: string;
  packs_synced: number;
  catalog_version?: string | null;
}
