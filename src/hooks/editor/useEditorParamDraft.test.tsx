import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useEditorParamDraft } from "./useEditorFilteredParams";
import { buildCustomChanges, buildIniSnapshot, engineParamId } from "@/lib/editor";
import { testParameters } from "@/test/fixtures/testParameters";

const basic = { ...testParameters[0], file: "GameUserSettings.ini", key: "sg.ShadowQuality", value: "2" };
const advanced = { ...testParameters[0], file: "Engine.ini", key: "r.Fog", value: "1" };
const baseline = [basic, advanced];
const shipped = buildIniSnapshot(baseline);

describe("editor draft refresh", () => {
  it("preserves the other panel and edits made while apply is pending", () => {
    const dirty = { current: false };
    const { result, rerender } = renderHook(
      ({ disk }) => useEditorParamDraft(disk, dirty, shipped, "game"),
      { initialProps: { disk: baseline } },
    );
    act(() => {
      dirty.current = true;
      result.current.setParams([{ ...basic, value: "3" }, { ...advanced, value: "0" }]);
    });
    // Apply submitted basic=3. The user edits basic again before the reply.
    act(() => result.current.setParams([{ ...basic, value: "4" }, { ...advanced, value: "0" }]));
    const refreshed = [{ ...basic, value: "3" }, advanced];
    rerender({ disk: refreshed });
    expect(result.current.params.map((p) => p.value)).toEqual(["4", "0"]);
    expect(dirty.current).toBe(true);
    const pending = buildCustomChanges(result.current.params, refreshed, undefined, result.current.engineEnabled, new Set(["Scalability"]), "advanced", shipped);
    expect(pending.files["Engine.ini"]?.["[/Script/Engine.RendererSettings]"]?.["r.Fog"]).toBe("0");
  });

  it("accepts applied rows while retaining other changes and refreshed metadata", () => {
    const dirty = { current: false };
    const { result, rerender } = renderHook(({ disk }) => useEditorParamDraft(disk, dirty, shipped), { initialProps: { disk: baseline } });
    act(() => {
      dirty.current = true;
      result.current.setParams([{ ...basic, value: "3" }, { ...advanced, value: "0" }]);
    });
    rerender({ disk: [{ ...basic, value: "3", title: "Updated title" }, advanced] });
    expect(result.current.params[0].title).toBe("Updated title");
    expect(result.current.params.map((p) => p.value)).toEqual(["3", "0"]);
    rerender({ disk: [{ ...basic, value: "3" }, { ...advanced, value: "0" }] });
    expect(dirty.current).toBe(false);
  });

  it("preserves toggle edits across refresh and clears them after apply", () => {
    const dirty = { current: false };
    const empty = new Set<string>();
    const { result, rerender } = renderHook(({ disk }) => useEditorParamDraft(disk, dirty, empty), { initialProps: { disk: [advanced] } });
    act(() => {
      dirty.current = true;
      result.current.setEngineEnabled(new Set());
    });
    rerender({ disk: [{ ...advanced, title: "Refreshed" }] });
    expect(result.current.engineEnabled.has(engineParamId(advanced))).toBe(false);
    expect(dirty.current).toBe(true);
    rerender({ disk: [{ ...advanced, present_in_ini: false }] });
    expect(dirty.current).toBe(false);
  });

  it("resets the draft when the selected game changes", () => {
    const dirty = { current: false };
    const { result, rerender } = renderHook(({ scope }) => useEditorParamDraft(baseline, dirty, shipped, scope), { initialProps: { scope: "A" } });
    act(() => {
      dirty.current = true;
      result.current.setParams([{ ...basic, value: "4" }, advanced]);
    });
    rerender({ scope: "B" });
    expect(result.current.params).toEqual(baseline);
    expect(dirty.current).toBe(false);
  });

  it("retains a staged conflict removal while the basic panel refreshes", () => {
    const dirty = { current: false };
    const { result, rerender } = renderHook(({ disk }) => useEditorParamDraft(disk, dirty, shipped), { initialProps: { disk: baseline } });
    act(() => {
      dirty.current = true;
      result.current.setParams([basic, { ...advanced, value: "", present_in_ini: false }]);
    });
    const refreshed = [{ ...basic, value: "3" }, advanced];
    rerender({ disk: refreshed });
    const pending = buildCustomChanges(result.current.params, refreshed, undefined, result.current.engineEnabled, new Set(["Scalability"]), "advanced", shipped);
    expect(pending.removals["Engine.ini"]?.["[/Script/Engine.RendererSettings]"]).toEqual(["r.Fog"]);
  });

  it("preserves a value reverted to the old baseline while apply is pending", () => {
    const dirty = { current: false };
    const { result, rerender } = renderHook(({ disk }) => useEditorParamDraft(disk, dirty, shipped), { initialProps: { disk: baseline } });
    const submitted = [{ ...basic, value: "3" }, advanced];
    const changes = buildCustomChanges(submitted, baseline, undefined, new Set(), new Set(["Scalability"]), "basic", shipped);
    act(() => {
      dirty.current = true;
      result.current.setParams(baseline);
      result.current.recordAppliedDraft({ params: submitted, engineEnabled: new Set(), changes });
    });
    rerender({ disk: submitted });
    expect(result.current.params[0].value).toBe("2");
    expect(dirty.current).toBe(true);
  });
});
