const SVG_NS = "http://www.w3.org/2000/svg";

type IconName =
  | "library"
  | "sliders"
  | "settings"
  | "search"
  | "plus"
  | "play"
  | "cpu"
  | "zap"
  | "trash"
  | "restore"
  | "heart"
  | "grid"
  | "list"
  | "refresh"
  | "info"
  | "warn";

const ICON_PATHS: Record<IconName, string[]> = {
  library: ["M16 6l4 14", "M12 6v14", "M8 8v12", "M4 4v16"],
  sliders: [
    "M21 4h-7",
    "M10 4H3",
    "M21 12h-9",
    "M8 12H3",
    "M21 20h-5",
    "M12 20H3",
    "M14 2v4",
    "M8 10v4",
    "M16 18v4",
  ],
  settings: [
    "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z",
    "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z",
  ],
  search: ["M11 19a8 8 0 1 0 0-16 8 8 0 0 0 0 16z", "M21 21l-4.3-4.3"],
  plus: ["M5 12h14", "M12 5v14"],
  play: ["M6 3l14 9-14 9V3z"],
  cpu: [
    "M4 4h16v16H4z",
    "M9 9h6v6H9z",
    "M15 2v2",
    "M15 20v2",
    "M2 15h2",
    "M2 9h2",
    "M20 15h2",
    "M20 9h2",
    "M9 2v2",
    "M9 20v2",
  ],
  zap: ["M13 2L3 14h9l-1 8 10-12h-9l1-8z"],
  trash: [
    "M3 6h18",
    "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6",
    "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2",
    "M10 11v6",
    "M14 11v6",
  ],
  restore: ["M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8", "M3 3v5h5"],
  heart: [
    "M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z",
  ],
  grid: [
    "M3 3h7v7H3z",
    "M14 3h7v7h-7z",
    "M14 14h7v7h-7z",
    "M3 14h7v7H3z",
  ],
  list: ["M8 6h13", "M8 12h13", "M8 18h13", "M3 6h.01", "M3 12h.01", "M3 18h.01"],
  refresh: [
    "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8",
    "M21 3v5h-5",
    "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16",
    "M8 16H3v5",
  ],
  info: ["M12 22a10 10 0 1 0 0-20 10 10 0 0 0 0 20z", "M12 16v-4", "M12 8h.01"],
  warn: [
    "M21.73 18l-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3z",
    "M12 9v4",
    "M12 17h.01",
  ],
};

const FILLED: ReadonlySet<IconName> = new Set(["play", "heart", "zap"]);

export function icon(name: IconName, size = 13): SVGSVGElement {
  const svg = document.createElementNS(SVG_NS, "svg");
  svg.setAttribute("class", "mock__icon");
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("aria-hidden", "true");
  svg.setAttribute("fill", FILLED.has(name) ? "currentColor" : "none");
  svg.setAttribute("stroke", FILLED.has(name) ? "none" : "currentColor");
  svg.setAttribute("stroke-width", "2");
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
  for (const d of ICON_PATHS[name]) {
    const path = document.createElementNS(SVG_NS, "path");
    path.setAttribute("d", d);
    svg.append(path);
  }
  return svg;
}

export type BadgeTone = "neutral" | "accent" | "success" | "warning" | "info";

export function badge(tone: BadgeTone, text: string): HTMLSpanElement {
  const el = document.createElement("span");
  el.className = `mock__badge mock__badge--${tone}`;
  el.textContent = text;
  return el;
}

export type ButtonVariant = "primary" | "secondary" | "ghost";

export function mockButton(
  variant: ButtonVariant,
  label: string,
  iconName?: IconName,
): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = `mock__btn mock__btn--${variant}`;
  btn.tabIndex = -1;
  if (iconName) btn.append(icon(iconName, 12));
  btn.append(document.createTextNode(label));
  return btn;
}

export interface DemoTarget {
  demoTo?: string;
  demoAt?: number;
}

function applyDemoAttrs(el: HTMLElement, value: string, demo: DemoTarget): void {
  if (demo.demoTo === undefined) return;
  el.dataset.demoFrom = value;
  el.dataset.demoTo = demo.demoTo;
  el.dataset.demoAt = String(demo.demoAt ?? 0.5);
}

export function mockSwitch(on: boolean, label: string, demo: DemoTarget = {}): HTMLButtonElement {
  const btn = document.createElement("button");
  btn.type = "button";
  btn.className = "mock__toggle";
  btn.setAttribute("role", "switch");
  btn.setAttribute("aria-checked", String(on));
  btn.setAttribute("aria-label", label);
  btn.tabIndex = -1;
  const knob = document.createElement("span");
  knob.className = "mock__toggle-knob";
  btn.append(knob);
  applyDemoAttrs(btn, on ? "True" : "False", demo);
  return btn;
}

export function mockSelect(
  options: string[],
  value: string,
  label: string,
  demo: DemoTarget = {},
): HTMLSelectElement {
  const select = document.createElement("select");
  select.className = "mock__select";
  select.setAttribute("aria-label", label);
  select.tabIndex = -1;
  for (const option of options) {
    const opt = document.createElement("option");
    opt.value = option;
    opt.textContent = option;
    select.append(opt);
  }
  select.value = value;
  applyDemoAttrs(select, value, demo);
  return select;
}

export function mockSlider(
  min: number,
  max: number,
  value: string,
  label: string,
  demo: DemoTarget = {},
): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "mock__range";

  const input = document.createElement("input");
  input.type = "range";
  input.className = "mock__slider";
  input.min = String(min);
  input.max = String(max);
  input.value = value;
  input.setAttribute("aria-label", label);
  input.tabIndex = -1;
  applyDemoAttrs(input, value, demo);
  const pct = max > min ? ((Number(value) - min) / (max - min)) * 100 : 0;
  input.style.background = `linear-gradient(90deg, var(--accent) ${pct}%, var(--border) ${pct}%)`;

  const num = document.createElement("input");
  num.type = "number";
  num.className = "mock__range-num";
  num.min = String(min);
  num.max = String(max);
  num.value = value;
  num.readOnly = true;
  num.setAttribute("aria-label", label);
  num.tabIndex = -1;

  wrap.append(input, num);
  return wrap;
}
