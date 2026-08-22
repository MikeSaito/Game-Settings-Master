import type { LocaleStrings, MockGame } from "../i18n/types";
import type { MockScreen } from "./appWindow";
import { badge, icon, mockButton } from "./ds";

export type EditorPanel = "basic" | "advanced" | "backups";

/** Шапка выбранной игры: обложка, имя, бейджи, кнопка запуска и путь к конфигу. */
export function buildContextBar(t: LocaleStrings, game: MockGame): HTMLElement {
  const bar = document.createElement("div");
  bar.className = "mock__ctx";

  const cover = document.createElement("div");
  cover.className = "mock__ctx-cover";
  cover.style.setProperty("--hue", String(game.hue));
  cover.setAttribute("aria-hidden", "true");
  cover.textContent = game.name.trim().charAt(0).toUpperCase();

  const info = document.createElement("div");
  info.className = "mock__ctx-info";
  const nameRow = document.createElement("div");
  nameRow.className = "mock__ctx-name-row";
  const name = document.createElement("span");
  name.className = "mock__ctx-name";
  name.textContent = game.name;
  nameRow.append(
    name,
    badge("accent", game.engine),
    badge("neutral", game.version),
    badge(game.ok ? "success" : "warning", game.ok ? t.mock.configOk : t.mock.configMissing),
  );
  const pathRow = document.createElement("div");
  pathRow.className = "mock__ctx-path";
  const pathLabel = document.createElement("span");
  pathLabel.className = "mock__ctx-path-label";
  pathLabel.textContent = t.mock.ctxConfig;
  const pathValue = document.createElement("code");
  pathValue.textContent = game.config || game.path;
  pathRow.append(pathLabel, pathValue);
  info.append(nameRow, pathRow);

  const actions = document.createElement("div");
  actions.className = "mock__ctx-actions";
  const gpuBadge = badge("info", t.mock.gpu);
  gpuBadge.prepend(icon("cpu", 11));
  actions.append(gpuBadge, mockButton("primary", t.mock.ctxPlay, "play"));

  bar.append(cover, info, actions);
  return bar;
}

/** Панель переключения режимов редактора: сегмент-контрол + подсказка. */
export function buildModeBar(
  t: LocaleStrings,
  active: EditorPanel,
  onPanel: (screen: MockScreen) => void,
): HTMLElement {
  const bar = document.createElement("div");
  bar.className = "mock__modebar";

  const segment = document.createElement("div");
  segment.className = "mock__segment";
  const tabs: Array<{ id: string; label: string; target: MockScreen | null }> = [
    { id: "basic", label: t.mock.tabs.basic, target: "basic" },
    { id: "advanced", label: t.mock.tabs.advanced, target: "advanced" },
    { id: "presets", label: t.mock.tabs.presets, target: null },
    { id: "backups", label: t.mock.tabs.backups, target: "backups" },
  ];
  for (const tab of tabs) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "mock__segment-btn";
    if (tab.id === active) btn.classList.add("is-active");
    btn.textContent = tab.label;
    btn.tabIndex = -1;
    if (tab.target === null) {
      btn.disabled = true;
    } else {
      const target = tab.target;
      btn.addEventListener("click", () => onPanel(target));
    }
    segment.append(btn);
  }

  const hintRow = document.createElement("div");
  hintRow.className = "mock__modebar-hint";
  const hint = document.createElement("div");
  hint.className = "mock__modebar-hint-text";
  hint.textContent =
    active === "basic"
      ? t.mock.basicHint
      : active === "advanced"
        ? t.mock.advancedHint
        : t.mock.backupsHint;
  hintRow.append(hint);

  if (active !== "backups") {
    const note = document.createElement("div");
    note.className = `mock__modebar-note mock__modebar-note--${active === "basic" ? "safe" : "warn"}`;
    if (active === "advanced") note.append(icon("warn", 11));
    note.append(document.createTextNode(active === "basic" ? t.mock.basicSafe : t.mock.advancedWarn));
    hintRow.append(note);
  }

  bar.append(segment, hintRow);
  return bar;
}

/** Строка бейджей со статистикой каталога над списком параметров. */
export function buildStatsRow(t: LocaleStrings, panel: "basic" | "advanced"): HTMLElement {
  const row = document.createElement("div");
  row.className = "mock__stats";
  row.append(badge("info", t.mock.paramsForEngine), badge("success", t.mock.known));
  row.append(
    panel === "basic" ? badge("accent", t.mock.sgLimits) : badge("warning", t.mock.engineShort),
  );
  return row;
}

export interface ApplyBarRefs {
  el: HTMLElement;
  changes: HTMLElement;
}

/** Нижний бар редактора: счетчик изменений и кнопки действий. */
export function buildApplyBar(t: LocaleStrings, panel: "basic" | "advanced"): ApplyBarRefs {
  const bar = document.createElement("div");
  bar.className = "mock__applybar";

  const changes = document.createElement("div");
  changes.className = "mock__changes";
  changes.textContent = t.mock.changes(0);

  const actions = document.createElement("div");
  actions.className = "mock__applybar-actions";
  actions.append(
    mockButton("ghost", t.mock.discard, "trash"),
    mockButton("primary", panel === "basic" ? t.mock.applyBasic : t.mock.applyAdvanced, "zap"),
  );

  bar.append(changes, actions);
  return { el: bar, changes };
}
