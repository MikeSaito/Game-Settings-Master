import type { LocaleStrings, MockParam } from "../i18n/types";
import type { MockScreen } from "./appWindow";
import { buildApplyBar, buildContextBar, buildModeBar, buildStatsRow } from "./chrome";
import { mockSelect, mockSlider, mockSwitch } from "./ds";

function buildRow(p: MockParam): HTMLElement {
  const row = document.createElement("div");
  row.className = "mock__row";

  const dot = document.createElement("span");
  dot.className = "mock__row-dot";
  dot.setAttribute("aria-hidden", "true");

  const label = document.createElement("div");
  label.className = "mock__row-label";
  const title = document.createElement("div");
  title.className = "mock__row-title";
  title.textContent = p.title;
  const key = document.createElement("div");
  key.className = "mock__row-key";
  const code = document.createElement("code");
  code.textContent = p.key;
  key.append(code, document.createTextNode("GameUserSettings.ini"));
  label.append(title, key);

  const control = document.createElement("div");
  control.className = "mock__control";
  const demo = { demoTo: p.demoTo, demoAt: p.demoAt };
  if (p.kind === "slider") {
    control.append(mockSlider(p.min ?? 0, p.max ?? 100, p.value, p.title, demo));
  } else if (p.kind === "select") {
    control.append(mockSelect(p.options ?? [], p.value, p.title, demo));
  } else {
    control.append(mockSwitch(p.value.toLowerCase() === "true", p.title, demo));
  }

  row.append(dot, label, control);
  return row;
}

export function buildBasicScreen(
  t: LocaleStrings,
  onPanel: (screen: MockScreen) => void,
): { el: HTMLElement; cleanup: () => void } {
  const el = document.createElement("div");
  el.className = "mock__editor";

  const game = t.mock.games[0];
  if (!game) return { el, cleanup: () => {} };

  const params = document.createElement("div");
  params.className = "mock__params";
  for (const p of t.mock.basicParams) {
    params.append(buildRow(p));
  }

  const applyBar = buildApplyBar(t, "basic");

  el.append(
    buildContextBar(t, game),
    buildModeBar(t, "basic", onPanel),
    buildStatsRow(t, "basic"),
    params,
    applyBar.el,
  );
  return { el, cleanup: () => {} };
}
