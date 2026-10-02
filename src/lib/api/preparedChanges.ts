import { invoke } from "@tauri-apps/api/core";
import i18n from "@/i18n";
import type { ApplyResult, PreparedChanges, PrepareRequest } from "./bindings";

export interface PreviewRequest {
  plan: PreparedChanges;
  request: PrepareRequest;
  reprepare?: () => Promise<PreparedChanges>;
  resolve: (result: ApplyResult) => void;
  reject: (error: Error) => void;
}
let receiver: ((request: PreviewRequest) => void) | undefined;
export function subscribePreview(listener: (request: PreviewRequest) => void) {
  receiver = listener;
  return () => {
    if (receiver === listener) receiver = undefined;
  };
}
export function prepareChanges(
  request: PrepareRequest,
): Promise<PreparedChanges> {
  return invoke("prepare_changes", { request });
}
export function discardPrepared(planId: string): Promise<void> {
  return invoke("discard_prepared_changes", { planId });
}
export function showPrepared(
  plan: PreparedChanges,
  request: PrepareRequest,
  reprepare?: () => Promise<PreparedChanges>,
): Promise<ApplyResult> {
  if (!receiver) {
    void discardPrepared(plan.id);
    return Promise.reject(
      new Error(i18n.t("improvements:preview.unavailable")),
    );
  }
  return new Promise((resolve, reject) =>
    receiver!({ plan, request, reprepare, resolve, reject }),
  );
}
export type ChangeRequest = Pick<PrepareRequest, "game_id" | "config_dir"> &
  Partial<PrepareRequest>;
export async function previewAndApply(
  input: ChangeRequest,
): Promise<ApplyResult> {
  const request: PrepareRequest = {
    changes: { files: {}, removals: {} },
    backup_id: null,
    restore_origin_backup_id: null,
    preset_metadata: null,
    input_revision: null,
    restore_files: [],
    input_updates: [],
    ...input,
  };
  const plan = await prepareChanges(request);
  if (plan.operations.length === 0) {
    await discardPrepared(plan.id);
    return {
      backup_id: "",
      changed_files: [],
      diff: [],
      effective_config_dir: plan.config_dir,
    };
  }
  if (!receiver) {
    await discardPrepared(plan.id);
    throw new Error(i18n.t("improvements:preview.unavailable"));
  }
  return showPrepared(plan, request);
}
