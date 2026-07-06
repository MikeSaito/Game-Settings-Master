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

const sgViewDistanceParam: GameParameter = {
  ...basicParameters.find((p) => p.key === "sg.TextureQuality")!,
  key: "sg.ViewDistanceQuality",
  title: "View Distance Quality",
  value: "2",
  min: "0",
  max: "10",
  ui_control: "slider",
};

const shadowConflictParams: GameParameter[] = [
  {
    ...basicParameters.find((p) => p.key === "sg.ShadowQuality")!,
    section: "ScalabilityGroups",
    tier_hint: "Low (0): r.ShadowQuality=0 | High (2): r.ShadowQuality=2",
  },
  sgViewDistanceParam,
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

function mergeByKey(...groups: GameParameter[][]): GameParameter[] {
  const byKey = new Map<string, GameParameter>();
  for (const group of groups) {
    for (const param of group) {
      byKey.set(param.key, { ...param });
    }
  }
  return [...byKey.values()];
}

/** Default E2E catalog: screenshot basic GUS params only. */
export function createBasicE2eParameters(): GameParameter[] {
  return basicParameters.map((param) => ({ ...param }));
}

/** E2E catalog with sg limit fixture (slider max above scalability cap). */
export function createSgLimitE2eParameters(): GameParameter[] {
  return mergeByKey(createBasicE2eParameters(), [sgViewDistanceParam]);
}

/** E2E catalog with sg/r shadow conflict plus sg limit fixture. */
export function createE2eConflictParameters(): GameParameter[] {
  return mergeByKey(createBasicE2eParameters(), shadowConflictParams);
}

/** @deprecated Use createE2eConflictParameters() in new tests. */
export function createE2eParameters(): GameParameter[] {
  return createE2eConflictParameters();
}

export type E2eFixtureMode = "basic" | "sg-limit" | "conflict";

export function createE2eParametersForMode(mode: E2eFixtureMode): GameParameter[] {
  switch (mode) {
    case "sg-limit":
      return createSgLimitE2eParameters();
    case "conflict":
      return createE2eConflictParameters();
    default:
      return createBasicE2eParameters();
  }
}

export function readE2eFixtureMode(): E2eFixtureMode {
  if (typeof window === "undefined") return "basic";
  try {
    const raw = sessionStorage.getItem("gsm-e2e-fixtures");
    if (raw === "sg-limit" || raw === "conflict") return raw;
  } catch {
    /* ignore */
  }
  return "basic";
}
