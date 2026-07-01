import type { GameParameter } from "@/lib/core";
import { basicParameters } from "@/screenshot/fixtures";

function engineParam(
  overrides: Partial<GameParameter> & Pick<GameParameter, "key" | "value">,
): GameParameter {
  return {
    section: "SystemSettings",
    file: "Engine.ini",
    title: overrides.key,
    description: "",
    impact: "",
    category: "Shadows",
    min: null,
    max: null,
    in_game_label: null,
    value_hint: null,
    value_type: "int",
    known: true,
    editable: true,
    present_in_ini: true,
    default_value: null,
    ui_control: null,
    step: null,
    options: null,
    recommended: null,
    catalog_recommended: false,
    tier_hint: null,
    ...overrides,
  };
}

const shadowConflictParams: GameParameter[] = [
  {
    ...basicParameters.find((p) => p.key === "sg.ShadowQuality")!,
    section: "ScalabilityGroups",
    tier_hint: "Low (0): r.ShadowQuality=0 | High (2): r.ShadowQuality=2",
  },
  engineParam({
    key: "r.ShadowQuality",
    value: "5",
    title: "Shadow Quality",
  }),
  engineParam({
    key: "r.Shadow.MaxResolution",
    value: "4096",
    title: "Shadow Max Resolution",
  }),
];

/** E2E catalog: basic GUS params plus sg/r shadow conflict fixtures. */
export function createE2eParameters(): GameParameter[] {
  const byKey = new Map(basicParameters.map((p) => [p.key, { ...p }]));
  for (const param of shadowConflictParams) {
    byKey.set(param.key, { ...param });
  }
  return [...byKey.values()];
}
