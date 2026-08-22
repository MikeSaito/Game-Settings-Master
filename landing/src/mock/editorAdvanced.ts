import type { LocaleStrings, MockParam } from "../i18n/types";
import type { MockScreen } from "./appWindow";
import { buildApplyBar, buildContextBar, buildModeBar, buildStatsRow } from "./chrome";
import { badge, mockButton, mockSwitch } from "./ds";

function buildRow(t: LocaleStrings, p: MockParam): HTMLElement {
  const row = document.createElement("div");
  row.className = "mock__row mock__row--engine";

  const toggleCell = document.createElement("div");
  toggleCell.className = "mock__row-toggle";
  toggleCell.append(mockButton("ghost", t.mock.removeFromIni));

  const label = document.createElement("div");
  label.className = "mock__row-label";
  const titleRow = document.createElement("div");
  titleRow.className = "mock__row-title-row";
  const title = document.createElement("span");
  title.className = "mock__row-title";
  title.textContent = p.title;
  titleRow.append(title);
  if (p.tier) titleRow.append(badge("accent", p.tier));
  if (p.warning) titleRow.append(badge("warning", p.warning));
  if (p.hint) titleRow.append(badge("info", p.hint));
  titleRow.append(badge("success", t.mock.inIni));
  const key = document.createElement("div");
  key.className = "mock__row-key";
  const code = document.createElement("code");
  code.textContent = p.key;
  key.append(code, document.createTextNode("Engine.ini"));
  label.append(titleRow, key);

  const control = document.createElement("div");
  control.className = "mock__control";
  if (p.kind === "toggle") {
    const on = p.value === "1" || p.value.toLowerCase() === "true";
    const wrap = document.createElement("div");
    wrap.className = "mock__switch-value";
    wrap.append(mockSwitch(on, p.title));
    const value = document.createElement("span");
    value.className = "mock__switch-num";
    value.textContent = p.value;
    wrap.append(value);
    control.append(wrap);
  } else {
    const box = document.createElement("code");
    box.className = "mock__cvar";
    box.textContent = p.value;
    control.append(box);
  }

  row.append(toggleCell, label, control);
  return row;
}

function buildDetailPane(t: LocaleStrings): HTMLElement {
  const pane = document.createElement("aside");
  pane.className = "mock__detail";

  const title = document.createElement("div");
  title.className = "mock__detail-title";
  title.textContent = t.mock.detail.title;

  const key = document.createElement("code");
  key.className = "mock__detail-key";
  key.textContent = t.mock.detail.key;

  const rows = document.createElement("dl");
  rows.className = "mock__detail-rows";
  for (const [label, value] of [t.mock.detail.current, t.mock.detail.type, t.mock.detail.range]) {
    const dt = document.createElement("dt");
    dt.textContent = label;
    const dd = document.createElement("dd");
    dd.textContent = value;
    rows.append(dt, dd);
  }

  const tierRow = document.createElement("div");
  tierRow.className = "mock__detail-tier";
  tierRow.append(badge("accent", t.mock.detail.tier));

  const desc = document.createElement("p");
  desc.className = "mock__detail-desc";
  desc.textContent = t.mock.detail.desc;

  const compat = document.createElement("div");
  compat.className = "mock__detail-compat";
  const compatLabel = document.createElement("span");
  compatLabel.textContent = t.mock.detail.compat[0];
  const compatValue = document.createElement("b");
  compatValue.textContent = t.mock.detail.compat[1];
  compat.append(compatLabel, compatValue);

  pane.append(title, key, rows, tierRow, desc, compat);
  return pane;
}

export function buildAdvancedScreen(
  t: LocaleStrings,
  onPanel: (screen: MockScreen) => void,
): { el: HTMLElement; cleanup: () => void } {
  const el = document.createElement("div");
  el.className = "mock__editor";

  const game = t.mock.games[0];
  if (!game) return { el, cleanup: () => {} };

  const body = document.createElement("div");
  body.className = "mock__editor-body";

  const paramsCol = document.createElement("div");
  paramsCol.className = "mock__params-col";
  const params = document.createElement("div");
  params.className = "mock__params";
  for (const p of t.mock.advancedParams) {
    params.append(buildRow(t, p));
  }
  const applyBar = buildApplyBar(t, "advanced");
  paramsCol.append(params, applyBar.el);

  body.append(paramsCol, buildDetailPane(t));

  el.append(
    buildContextBar(t, game),
    buildModeBar(t, "advanced", onPanel),
    buildStatsRow(t, "advanced"),
    body,
  );
  return { el, cleanup: () => {} };
}
