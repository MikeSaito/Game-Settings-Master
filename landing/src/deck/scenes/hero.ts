import type { LocaleStrings } from "../../i18n/types";
import { buildAppWindow } from "../../mock/appWindow";
import { makeActions } from "../../ui/cta";
import type { Scene } from "../scroll";
import { sceneShell } from "./shell";

/** Стартовый экран: заголовок, кнопки и полный вид приложения без скролл-анимации. */
export function buildHeroScene(t: LocaleStrings): Scene {
  const shell = sceneShell("hero", t.story.hero, { titleAs: "h1" });

  const cta = document.createElement("div");
  cta.className = "hero__cta";
  cta.append(makeActions(t));
  shell.copy.append(cta);

  const app = buildAppWindow(t, { initial: "library" });
  shell.stage.append(app.el);

  const hint = document.createElement("p");
  hint.className = "scene__hint";
  hint.textContent = t.story.scrollHint;
  hint.setAttribute("aria-hidden", "true");
  shell.pin.append(hint);

  return {
    el: shell.section,
    init() {},
    cleanup() {
      app.cleanup();
    },
  };
}
