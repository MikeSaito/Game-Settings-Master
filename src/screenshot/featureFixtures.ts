import type {
  ChangeOperation,
  DiagnosticReport,
  PreparedChanges,
  PrepareRequest,
} from "@/lib/api/bindings";

export const featureOperations: ChangeOperation[] = [
  {
    id: "shadow",
    group: "shadow",
    file: "Engine.ini",
    section: "SystemSettings",
    key: "r.Shadow.MaxResolution",
    before: "2048",
    after: "1024",
    kind: "scalar",
  },
  {
    id: "reflections",
    group: "reflections",
    file: "Engine.ini",
    section: "SystemSettings",
    key: "r.Lumen.Reflections.Allow",
    before: "1",
    after: "0",
    kind: "scalar",
  },
];
export const featureConfigDir = "C:\\Games\\Example\\Saved\\Config\\Windows";
export const featurePlan: PreparedChanges = {
  id: "screenshot-preview",
  game_id: "game-1",
  config_dir: featureConfigDir,
  operations: featureOperations,
  issues: [],
  revisions: {},
};
export const featureRequest: PrepareRequest = {
  game_id: "game-1",
  config_dir: featureConfigDir,
  changes: { files: {}, removals: {} },
  backup_id: null,
  restore_origin_backup_id: null,
  preset_metadata: null,
  input_revision: null,
  input_updates: [],
  restore_files: [],
};
export const featureReport: DiagnosticReport = {
  game_id: "game-1",
  config_dir: featureConfigDir,
  expected_dir: featureConfigDir,
  status: "changed",
  changes: [{ ...featureOperations[0], before: "1024", after: "2048" }],
  checked_at: "2026-10-03T10:00:00Z",
  message: null,
};
