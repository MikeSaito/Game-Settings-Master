import type { GameParameter } from "@/lib/core";
import { engineParamId, isEngineEnabled, isIniMembershipToggleable } from "./engineParams";
import { iniSnapshotKey } from "./iniSnapshot";
import { paramValuesEqual } from "./paramValueEqual";
import type { buildCustomChanges } from "./buildCustomChanges";

export interface AppliedDraft {
  params: GameParameter[];
  engineEnabled: Set<string>;
  changes: ReturnType<typeof buildCustomChanges>;
}

/** Compare later edits to the submitted draft for exactly the rows applied. */
export function baselineAfterApply(
  baseline: GameParameter[],
  submitted: AppliedDraft,
  shippedIniKeys: ReadonlySet<string>,
): GameParameter[] {
  const touched = new Set<string>();
  for (const [file, sections] of Object.entries(submitted.changes.files)) {
    for (const [section, entries] of Object.entries(sections)) {
      for (const key of Object.keys(entries)) touched.add(iniSnapshotKey({ file, section, key }));
    }
  }
  for (const [file, sections] of Object.entries(submitted.changes.removals)) {
    for (const [section, keys] of Object.entries(sections)) {
      for (const key of keys) touched.add(iniSnapshotKey({ file, section, key }));
    }
  }
  const drafts = new Map(submitted.params.map((p) => [iniSnapshotKey(p), p]));
  return baseline.map((p) => {
    const id = iniSnapshotKey(p);
    const draft = drafts.get(id);
    if (!touched.has(id) || !draft) return p;
    const present = isIniMembershipToggleable(draft, shippedIniKeys)
      ? isEngineEnabled(draft, submitted.engineEnabled, shippedIniKeys)
      : draft.present_in_ini;
    return { ...p, value: draft.value, present_in_ini: present };
  });
}

/** Merge refreshed disk data without discarding edits in any panel. */
export function rebaseParameterDraft(
  draft: GameParameter[],
  previousBaseline: GameParameter[],
  baseline: GameParameter[],
  enabled: Set<string>,
  shippedIniKeys: ReadonlySet<string>,
) {
  const previous = new Map(previousBaseline.map((p) => [iniSnapshotKey(p), p]));
  const current = new Map(draft.map((p) => [iniSnapshotKey(p), p]));
  const engineEnabled = new Set<string>();
  let dirty = false;
  const params = baseline.map((fresh) => {
    const id = iniSnapshotKey(fresh);
    const old = previous.get(id);
    const edited = current.get(id);
    const valueChanged = old && edited && !paramValuesEqual(old.value, edited.value);
    const removed = old && edited && old.present_in_ini &&
      !edited.present_in_ini && edited.value.trim() === "";
    const row = valueChanged
      ? { ...fresh, value: edited.value, present_in_ini: removed ? false : fresh.present_in_ini }
      : fresh;
    if (!paramValuesEqual(row.value, fresh.value)) dirty = true;
    if (removed && fresh.present_in_ini) dirty = true;
    if (isIniMembershipToggleable(fresh, shippedIniKeys)) {
      const membershipChanged = old && edited &&
        isEngineEnabled(edited, enabled, shippedIniKeys) !== old.present_in_ini;
      const on = membershipChanged
        ? isEngineEnabled(edited!, enabled, shippedIniKeys)
        : fresh.present_in_ini;
      if (on) engineEnabled.add(engineParamId(fresh));
      if (on !== fresh.present_in_ini) dirty = true;
    }
    return row;
  });
  return { params, engineEnabled, dirty };
}
