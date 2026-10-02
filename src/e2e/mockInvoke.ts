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
import type {
  ChangeOperation,
  PrepareRequest,
  PreparedChanges,
  InputDocument,
  GameOverride,
} from "@/lib/api/bindings";
import { OVERRIDE_INI_FILES } from "@/lib/ini/configFiles";
import { iniSnapshotKeyFromParts } from "@/lib/editor/iniSnapshot";
import { testGame } from "@/test/fixtures/gameProfile";
import {
  createE2eParametersForMode,
  readE2eFixtureMode,
} from "@/e2e/parameters";

const e2eGpu: GpuCapabilities = {
  adapter_id: "test-nvidia",
  dedicated_memory_mb: 8192,
  shared_memory_mb: 16384,
  ray_tracing_status: "supported",
  vendor: "nvidia",
  name: "E2E Test GPU",
  supports_dlss: true,
  supports_dlss_fg: false,
  supports_ray_tracing: true,
};
const e2eAmd: GpuCapabilities = {
  ...e2eGpu,
  adapter_id: "test-amd",
  name: "E2E AMD GPU",
  vendor: "amd",
  supports_dlss: false,
  dedicated_memory_mb: 4096,
};
let selectedGpu: string | null = null;
const inputFixture = (): InputDocument => ({
  revision: crypto.randomUUID(),
  entries: [0, 1].map((index) => ({
    id: `input-${index}`,
    line: index + 2,
    key: "+ActionMappings",
    value: `(ActionName=Jump,Key=${index ? "J" : "SpaceBar"},bShift=False)`,
    fields: {
      ActionName: "Jump",
      Key: index ? "J" : "SpaceBar",
      bShift: "False",
    },
    axis_properties: null,
    editable: true,
    reason: null,
  })),
});
let inputDocument = inputFixture();
let overrides: GameOverride[] = [];

const scalabilityLimits: ScalabilityLimits = {
  groups: {},
  global_max: 4,
  sources: [],
};

function cloneParams(source: GameParameter[]): GameParameter[] {
  return source.map((param) => ({ ...param }));
}

let parameters = cloneParams(createE2eParametersForMode(readE2eFixtureMode()));
let backups: BackupInfo[] = [];
const snapshots = new Map<string, Map<string, string>>();
const prepared = new Map<string, PreparedChanges>();

function paramKey(param: GameParameter): string {
  return `${param.file}::${param.section}::${param.key}`;
}

function paramIniKey(param: GameParameter): string {
  return iniSnapshotKeyFromParts(param.file, param.section, param.key);
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
      (param) =>
        param.file === file &&
        param.present_in_ini &&
        param.value.trim() !== "",
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
          (row) =>
            paramIniKey(row) === iniSnapshotKeyFromParts(file, section, key),
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
          (row) =>
            paramIniKey(row) === iniSnapshotKeyFromParts(file, section, key),
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
  const backupId = `${nextBackupId()}_${crypto.randomUUID()}`;
  snapshots.set(backupId, snapshotCurrentValues());
  backups.unshift({
    id: backupId,
    created_at: new Date().toISOString(),
    files: changedFiles.length > 0 ? changedFiles : ["GameUserSettings.ini"],
  });
  return backupId;
}

export function resetE2eMockState(): void {
  parameters = cloneParams(createE2eParametersForMode(readE2eFixtureMode()));
  backups = [];
  snapshots.clear();
  prepared.clear();
  selectedGpu = null;
  inputDocument = inputFixture();
  overrides = [];
}

export function handleE2eInvoke(
  cmd: string,
  args?: Record<string, unknown>,
): unknown {
  switch (cmd) {
    case "scan_games":
      return [{ ...testGame, gpu_adapter_id: selectedGpu }];
    case "get_gpu_info_cmd":
    case "get_game_gpu":
      return selectedGpu === "test-amd" ? e2eAmd : e2eGpu;
    case "list_gpu_adapters":
      return [e2eGpu, e2eAmd];
    case "set_game_gpu":
      selectedGpu = args?.adapterId as string | null;
      return null;
    case "get_discovery_warnings":
      return [];
    case "get_diagnostic_report":
      return null;
    case "get_input_document":
      return structuredClone(inputDocument);
    case "create_snapshot": {
      const id = createBackup([]);
      backups[0].name = String(args?.name);
      return id;
    }
    case "rename_snapshot": {
      const backup = backups.find((backup) => backup.id === args?.backupId);
      if (backup) backup.name = String(args?.name);
      return null;
    }
    case "compare_snapshots":
      return [];
    case "prepare_changes": {
      const request = args?.request as PrepareRequest;
      const changes = request.changes ?? { files: {}, removals: {} };
      const restoreFiles = request.restore_files ?? [];
      const operations: ChangeOperation[] = [];
      const add = (param: GameParameter, after: string | null) => {
        const before = param.present_in_ini ? param.value : null;
        if (before === after) return;
        const id = operations.length.toString();
        operations.push({
          id,
          group: id,
          file: param.file,
          section: param.section,
          key: param.key,
          before,
          after,
          kind: "scalar",
        });
      };
      if (request.backup_id) {
        const snapshot = snapshots.get(request.backup_id);
        if (!snapshot) throw new Error("Snapshot missing");
        for (const param of parameters) {
          if (!restoreFiles.length || restoreFiles.includes(param.file))
            add(param, snapshot.get(paramKey(param)) ?? null);
        }
      } else {
        for (const [file, sections] of Object.entries(changes.files))
          for (const [section, entries] of Object.entries(sections))
            for (const [key, value] of Object.entries(entries)) {
              const param = parameters.find(
                (param) =>
                  paramIniKey(param) ===
                  iniSnapshotKeyFromParts(file, section, key),
              );
              if (param) add(param, value);
            }
        for (const [file, sections] of Object.entries(changes.removals ?? {}))
          for (const [section, keys] of Object.entries(sections))
            for (const key of keys) {
              const param = parameters.find(
                (param) =>
                  paramIniKey(param) ===
                  iniSnapshotKeyFromParts(file, section, key),
              );
              if (param) add(param, null);
            }
        for (const update of request.input_updates ?? []) {
          const entry = inputDocument.entries.find(
            (entry) => entry.id === update.id || entry.line === update.line,
          );
          if (entry && entry.value !== update.value) {
            const id = String(operations.length);
            operations.push({
              id,
              group: id,
              file: "Input.ini",
              section: "/Script/Engine.InputSettings",
              key: entry.key,
              before: entry.value,
              after: update.value,
              kind: `input:${entry.line}`,
            });
          }
        }
      }
      const plan: PreparedChanges = {
        id: crypto.randomUUID(),
        game_id: request.game_id,
        config_dir: request.config_dir,
        operations,
        issues: [],
        revisions: {},
      };
      prepared.set(plan.id, plan);
      return plan;
    }
    case "discard_prepared_changes":
      prepared.delete(String(args?.planId));
      return null;
    case "validate_prepared_changes":
      return [];
    case "apply_prepared_changes": {
      const plan = prepared.get(String(args?.planId));
      if (!plan) throw new Error("Preview expired");
      const ids = new Set(args?.selectedIds as string[]);
      const operations = plan.operations.filter((op) => ids.has(op.id));
      const backupId = operations.length ? createBackup([]) : "";
      for (const op of operations) {
        if (op.kind.startsWith("input:")) {
          const entry = inputDocument.entries.find(
            (entry) => entry.line === Number(op.kind.slice(6)),
          );
          if (entry && op.after) {
            entry.value = op.after;
            entry.fields = Object.fromEntries(
              op.after
                .slice(1, -1)
                .split(",")
                .map((pair) => pair.split("=")),
            );
          }
        }
        const param = parameters.find(
          (param) =>
            paramIniKey(param) ===
            iniSnapshotKeyFromParts(op.file, op.section, op.key),
        );
        if (param) {
          param.value = op.after ?? "";
          param.present_in_ini = op.after !== null;
        }
      }
      prepared.delete(plan.id);
      inputDocument.revision = crypto.randomUUID();
      return {
        applied_input_lines: operations
          .filter((op) => op.kind.startsWith("input:"))
          .map((op) => Number(op.kind.slice(6))),
        backup_id: backupId,
        changed_files: [...new Set(operations.map((op) => op.file))],
        diff: operations.map((op) => ({
          file: op.file,
          section: op.section,
          key: op.key,
          old_value: op.before,
          new_value: op.after ?? "",
        })),
        effective_config_dir: plan.config_dir,
      } satisfies ApplyResult;
    }
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
      return structuredClone(overrides);
    case "save_game_override":
      overrides.push(structuredClone(args?.overrideDef as GameOverride));
      return null;
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
        backups[0].files =
          changedFiles.length > 0 ? changedFiles : ["GameUserSettings.ini"];
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
            app_version: "1.0.5",
          };
    case "clear_crash_reports_cmd":
      return null;
    default:
      return null;
  }
}
