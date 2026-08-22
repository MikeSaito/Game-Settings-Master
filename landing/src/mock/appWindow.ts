import type { LocaleStrings } from "../i18n/types";
import { buildLibraryScreen } from "./library";
import { buildBasicScreen } from "./editorBasic";
import { buildAdvancedScreen } from "./editorAdvanced";
import { buildBackupsScreen } from "./backups";
import { icon } from "./ds";

export type MockScreen = "library" | "basic" | "advanced" | "backups";

export interface AppWindowApi {
  el: HTMLElement;
  setScreen: (screen: MockScreen) => void;
  getScreen: () => MockScreen;
  cleanup: () => void;
}

export function sourceLabel(t: LocaleStrings, source: "steam" | "epic" | "manual"): string {
  if (source === "steam") return t.mock.sourceSteam;
  if (source === "epic") return t.mock.sourceEpic;
  return t.mock.sourceManual;
}

export function buildAppWindow(
  t: LocaleStrings,
  opts: {
    initial: MockScreen;
    onNavigate?: (screen: MockScreen) => void;
  },
): AppWindowApi {
  const cleanups: Array<() => void> = [];
  let screenCleanup: (() => void) | null = null;
  let current: MockScreen = opts.initial;

  const el = document.createElement("div");
  el.className = "mock";
  // Декоративная реплика UI: role="img" делает всю начинку presentational для скринридеров.
  el.setAttribute("role", "img");
  el.setAttribute("aria-label", t.mock.windowTitle);

  const rail = document.createElement("div");
  rail.className = "mock__rail";

  const main = document.createElement("div");
  main.className = "mock__main";

  const railBtns: Array<{ id: "library" | "editor" | "settings"; btn: HTMLButtonElement }> = [];

  const mountScreen = (screen: MockScreen) => {
    screenCleanup?.();
    screenCleanup = null;
    current = screen;
    main.replaceChildren();
    const onPanel = (target: MockScreen) => {
      setScreen(target);
      opts.onNavigate?.(target);
    };
    let built: { el: HTMLElement; cleanup: () => void };
    if (screen === "library") built = buildLibraryScreen(t);
    else if (screen === "basic") built = buildBasicScreen(t, onPanel);
    else if (screen === "advanced") built = buildAdvancedScreen(t, onPanel);
    else built = buildBackupsScreen(t, onPanel);
    main.append(built.el);
    screenCleanup = built.cleanup;

    for (const { id, btn } of railBtns) {
      const on =
        (id === "library" && screen === "library") ||
        (id === "editor" && screen !== "library");
      if (on) btn.setAttribute("aria-current", "page");
      else btn.removeAttribute("aria-current");
    }
  };

  const setScreen = (screen: MockScreen) => {
    if (screen === current && main.childElementCount) return;
    mountScreen(screen);
  };

  const railDefs: Array<{
    id: "library" | "editor" | "settings";
    icon: "library" | "sliders" | "settings";
    label: string;
    target: MockScreen | null;
  }> = [
    { id: "library", icon: "library", label: t.mock.navLibrary, target: "library" },
    { id: "editor", icon: "sliders", label: t.mock.navEditor, target: "basic" },
    { id: "settings", icon: "settings", label: t.mock.navSettings, target: null },
  ];

  const railNav = document.createElement("div");
  railNav.className = "mock__rail-nav";
  let settingsBtn: HTMLButtonElement | null = null;

  for (const def of railDefs) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "mock__rail-btn";
    btn.setAttribute("aria-label", def.label);
    btn.title = def.label;
    btn.tabIndex = -1;
    btn.append(icon(def.icon, 15));
    if (def.target === null) {
      btn.disabled = true;
      btn.setAttribute("aria-disabled", "true");
      settingsBtn = btn;
    } else {
      const target = def.target;
      const handler = () => {
        setScreen(target);
        opts.onNavigate?.(target);
      };
      btn.addEventListener("click", handler);
      cleanups.push(() => btn.removeEventListener("click", handler));
      railNav.append(btn);
    }
    railBtns.push({ id: def.id, btn });
  }
  rail.append(railNav);
  if (settingsBtn) rail.append(settingsBtn);

  el.append(rail, main);
  mountScreen(opts.initial);

  return {
    el,
    setScreen,
    getScreen: () => current,
    cleanup: () => {
      screenCleanup?.();
      screenCleanup = null;
      for (const c of cleanups) c();
    },
  };
}
