export interface LaunchResult {
  launcher: string;
  detail: string;
  warning?: string;
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
  ui_control: string | null;
  step: string | null;
  options: ParameterOption[] | null;
  recommended: string | null;
}

export interface ParameterOption {
  value: string;
  label: string;
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

export type AppTab = "library" | "wizard" | "advanced" | "backups" | "reshade";

export type ReShadeApiId = "dx9" | "dx11" | "dx12" | "opengl" | "vulkan" | string;

export interface ReShadeBehaviorSettings {
  performance_mode?: boolean;
  key_overlay?: string | null;
  key_toggle_effects?: string | null;
}

export interface ReShadePresetOverrides {
  techniques?: string[] | null;
  parameters?: Record<string, Record<string, string>>;
  behavior?: ReShadeBehaviorSettings | null;
}

export interface ReShadePerGameSettings {
  enabled: boolean;
  api?: ReShadeApiId | null;
  api_remembered?: boolean;
  preset?: string | null;
  preset_overrides?: ReShadePresetOverrides | null;
}

export interface GraphicsApiInfo {
  id: ReShadeApiId;
  name: string;
  description: string;
  files: string[];
}

export interface ReShadeSettings {
  global_enabled: boolean;
  default_preset: string;
  warnings_acknowledged: boolean;
  install_warning_acknowledged: boolean;
  launch_warning_acknowledged: boolean;
  per_game: Record<string, ReShadePerGameSettings>;
}

export interface ReShadePresetInfo {
  id: string;
  name: string;
  description: string;
  author?: boolean;
}

export interface ReShadeSettingsResponse {
  settings: ReShadeSettings;
  presets: ReShadePresetInfo[];
  apis: GraphicsApiInfo[];
  suggested_api?: string | null;
  gpu_adapt_reason?: string | null;
  gpu_name?: string | null;
  requested_preset?: string | null;
  effective_preset?: string | null;
}

export interface ReShadeWorkspace {
  settings: ReShadeSettingsResponse;
  status: ReShadeGameStatus;
}

export interface ReShadeGameStatus {
  game_id: string;
  install_dir: string;
  target_dir: string;
  saved_api: string | null;
  api_remembered: boolean;
  configured_api: ReShadeApiId | null;
  installed_api: string | null;
  installed_files: string[];
  installed: boolean;
  active_preset: string | null;
  api_matches_install: boolean;
  reshade_ini_present: boolean;
  bundle_ready: boolean;
  bundled_binaries_valid: boolean;
  shaders_present: boolean;
  shaders_in_bundle: boolean;
  installed_proxy_valid: boolean;
  broken_install: boolean;
  exe_path: string | null;
  suggested_api?: string | null;
  gpu_name?: string | null;
  gpu_adapt_reason?: string | null;
  requested_preset?: string | null;
  effective_preset?: string | null;
}

export interface ReShadeInstallResult {
  target_dir: string;
  preset_id: string;
  graphics_api: ReShadeApiId;
  installed_files: string[];
  warnings?: string[];
}

export interface ReShadePresetParameter {
  effect: string;
  key: string;
  value: string;
}

export interface ReShadePresetDetails {
  id: string;
  techniques: string[];
  parameters: ReShadePresetParameter[];
  shaders_available: boolean;
}

export interface ReShadeRemoveResult {
  target_dir: string;
  restored_files: string[];
  removed_files: string[];
  warnings?: string[];
}

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
