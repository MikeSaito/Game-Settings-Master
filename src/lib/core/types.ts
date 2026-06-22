/**
 * IPC DTOs — generated from Rust (`npm run types:gen`).
 * Frontend-only types and serde-default augmentations live below.
 */
import type { GameParameter as GeneratedGameParameter } from "../api/bindings";

export type {
  ApplyResult,
  BackupInfo,
  ConfigDiffEntry,
  ConfigResetResult,
  GameConfig,
  GameOverride,
  GameProfile,
  GpuCapabilities,
  GpuVendor,
  IniFileData,
  LaunchResult,
  ParameterOption,
  ScalabilityLimits,
  ScreenResolution,
} from "../api/bindings";

/** Rust defaults `editable` / `present_in_ini` to true when omitted in JSON. */
export type GameParameter = GeneratedGameParameter & {
  editable: boolean;
  present_in_ini: boolean;
};

export type GameTabRoute = "advanced" | "backups";
export type AppTab = "library" | GameTabRoute;
