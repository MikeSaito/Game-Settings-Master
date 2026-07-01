import type {
  ApplyResult,
  BackupInfo,
  ConfigDiffEntry,
  ConfigResetResult,
  GameConfig,
  GameParameter,
  GpuCapabilities,
  ScalabilityLimits,
} from "@/lib/core";
import { OVERRIDE_INI_FILES } from "@/lib/ini/configFiles";
import { testGame } from "@/test/fixtures/gameProfile";
import { createE2eParameters } from "@/e2e/parameters";

const e2eGpu: GpuCapabilities = {
  vendor: "nvidia",
  name: "E2E Test GPU",
  supports_dlss: true,
  supports_dlss_fg: false,
  supports_ray_tracing: true,
};

const scalabilityLimits: ScalabilityLimits = {
  groups: {},
  global_max: 4,
  sources: [],
};

function cloneParams(source: GameParameter[]): GameParameter[] {
  return source.map((param) => ({ ...param }));
}

let parameters = cloneParams(createE2eParameters());
let backups: BackupInfo[] = [];
const snapshots = new Map<string, Map<string, string>>();

function paramKey(param: GameParameter): string {
  return `${param.file}::${param.section}::${param.key}`;
}

function nextBackupId(): string {
  const now = new Date();
  const pad = (value: number) => String(value).padStart(2, "0");
  return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}_${pad(now.getHours())}-${pad(now.getMinutes())}-${pad(now.getSeconds())}`;
}

function snapshotCurrentValues(): Map<string, string> {
  const snapshot = new Map<string, string>();
  for (const param of parameters) {
    snapshot.set(paramKey(param), param.value);
  }
  return snapshot;
}

function buildGameConfig(): GameConfig {
  const files: GameConfig["files"] = {};
  for (const file of ["GameUserSettings.ini", ...OVERRIDE_INI_FILES]) {
    const fileParams = parameters.filter(
      (param) => param.file === file && param.present_in_ini && param.value.trim() !== "",
    );
    if (fileParams.length === 0) continue;
    const sections: Record<string, Record<string, string>> = {};
    for (const param of fileParams) {
      if (!sections[param.section]) sections[param.section] = {};
      sections[param.section][param.key] = param.value;
    }
    files[file] = { sections };
  }
  return {
    config_dir: testGame.config_dir ?? "",
    files,
  };
}

function applyChanges(
  files: Record<string, Record<string, Record<string, string>>>,
): ConfigDiffEntry[] {
  const diff: ConfigDiffEntry[] = [];
  for (const [file, sections] of Object.entries(files)) {
    for (const [section, keys] of Object.entries(sections)) {
      for (const [key, newValue] of Object.entries(keys)) {
        const param = parameters.find(
          (row) => row.file === file && row.section === section && row.key === key,
        );
        if (!param) continue;
        const oldValue = param.value;
        if (oldValue === newValue) continue;
        diff.push({
          file,
          section,
          key,
          old_value: oldValue,
          new_value: newValue,
        });
        param.value = newValue;
        param.present_in_ini = true;
      }
    }
  }
  return diff;
}

function applyRemovals(
  removals: Record<string, Record<string, string[]>>,
): ConfigDiffEntry[] {
  const diff: ConfigDiffEntry[] = [];
  for (const [file, sections] of Object.entries(removals)) {
    for (const [section, keys] of Object.entries(sections)) {
      for (const key of keys) {
        const param = parameters.find(
          (row) => row.file === file && row.section === section && row.key === key,
        );
        if (!param || !param.present_in_ini) continue;
        diff.push({
          file,
          section,
          key,
          old_value: param.value,
          new_value: "",
        });
        param.value = "";
        param.present_in_ini = false;
      }
    }
  }
  return diff;
}

function createBackup(changedFiles: string[]): string {
  const backupId = nextBackupId();
  snapshots.set(backupId, snapshotCurrentValues());
  backups.unshift({
    id: backupId,
    created_at: new Date().toISOString(),
    files: changedFiles.length > 0 ? changedFiles : ["GameUserSettings.ini"],
  });
  return backupId;
}

export function resetE2eMockState(): void {
  parameters = cloneParams(createE2eParameters());
  backups = [];
  snapshots.clear();
}

export function handleE2eInvoke(cmd: string, args?: Record<string, unknown>): unknown {
  switch (cmd) {
    case "scan_games":
      return [testGame];
    case "get_gpu_info_cmd":
      return e2eGpu;
    case "get_desktop_resolution_cmd":
      return { width: 2560, height: 1440 };
    case "is_game_running_cmd":
      return false;
    case "set_language_cmd":
    case "set_app_background_mode_cmd":
      return null;
    case "get_game_parameters_cmd":
      return cloneParams(parameters);
    case "get_scalability_limits_cmd":
      return scalabilityLimits;
    case "get_game_overrides":
      return [];
    case "get_game_config":
      return buildGameConfig();
    case "apply_custom_cmd": {
      const changes = args?.changes as
        | {
            files?: Record<string, Record<string, Record<string, string>>>;
            removals?: Record<string, Record<string, string[]>>;
          }
        | undefined;
      const files = changes?.files ?? {};
      const removals = changes?.removals ?? {};
      createBackup([]);
      const diff = [...applyChanges(files), ...applyRemovals(removals)];
      const changedFiles = [...new Set(diff.map((entry) => entry.file))];
      if (backups[0]) {
        backups[0].files = changedFiles.length > 0 ? changedFiles : ["GameUserSettings.ini"];
      }
      const result: ApplyResult = {
        backup_id: backups[0]?.id ?? nextBackupId(),
        changed_files: changedFiles,
        diff,
        effective_config_dir: testGame.config_dir ?? null,
      };
      return result;
    }
    case "list_backups_cmd":
      return [...backups];
    case "restore_backup_cmd": {
      const backupId = String(args?.backupId ?? "");
      const snapshot = snapshots.get(backupId);
      if (!snapshot) {
        throw new Error(`Backup not found: ${backupId}`);
      }
      for (const param of parameters) {
        const saved = snapshot.get(paramKey(param));
        if (saved != null) {
          param.value = saved;
        }
      }
      return ["GameUserSettings.ini"];
    }
    case "reset_config_to_user_cmd": {
      const backupId = createBackup([]);
      const deletedFiles: string[] = [];
      for (const file of OVERRIDE_INI_FILES) {
        const hadOverride = parameters.some(
          (param) => param.file === file && param.present_in_ini,
        );
        if (!hadOverride) continue;
        deletedFiles.push(file);
        for (const param of parameters) {
          if (param.file === file) {
            param.present_in_ini = false;
            param.value = "";
          }
        }
      }
      const result: ConfigResetResult = {
        backup_id: backupId,
        deleted_files: deletedFiles,
      };
      return result;
    }
    case "submit_crash_report_cmd":
    case "list_crash_reports_cmd":
      return cmd === "list_crash_reports_cmd"
        ? []
        : {
            id: "e2e",
            created_at: "",
            kind: "uncaught",
            message: "",
            stack: null,
            component_stack: null,
            url: null,
            app_version: "1.0.4",
          };
    case "clear_crash_reports_cmd":
      return null;
    default:
      return null;
  }
}
