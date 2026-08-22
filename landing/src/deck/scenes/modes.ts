import type { LocaleStrings } from "../../i18n/types";
import { buildAppWindow, type MockScreen } from "../../mock/appWindow";
import { gsap, DESKTOP_QUERY, type Scene } from "../scroll";
import { sceneShell } from "./shell";

const FPS_FROM = 74;
const FPS_TO = 112;

/** Длительность движения слайдера в долях basic-фазы после порога demoAt. */
const SLIDER_SPAN = 0.45;
const SWITCH_AT = 0.5;
const ADV_LIT_START = 0.15;
const ADV_LIT_STEP = 0.18;
const TOOLTIP_AT = 0.55;

function clamp01(v: number): number {
  return v < 0 ? 0 : v > 1 ? 1 : v;
}

function lerp(a: number, b: number, k: number): number {
  return a + (b - a) * k;
}

export function buildModesScene(t: LocaleStrings): Scene {
  const shell = sceneShell("modes", t.story.modes);

  const wrap = document.createElement("div");
  wrap.className = "modes";

  const winWrap = document.createElement("div");
  winWrap.className = "modes__win";
  const app = buildAppWindow(t, { initial: "basic" });
  winWrap.append(app.el);

  const fps = document.createElement("p");
  fps.className = "modes__fps";
  fps.setAttribute("aria-hidden", "true");
  const fpsValue = document.createElement("b");
  fpsValue.textContent = String(FPS_FROM);
  fps.append(document.createTextNode(`${t.story.modes.fps} `), fpsValue);

  const ue = document.createElement("p");
  ue.className = "modes__ue";
  ue.setAttribute("aria-hidden", "true");
  const firstGame = t.mock.games[0];
  ue.textContent = firstGame ? `${firstGame.engine} ${firstGame.version}` : "UE 5";

  const tip = document.createElement("p");
  tip.className = "modes__tip";
  tip.setAttribute("aria-hidden", "true");
  tip.textContent = t.story.modes.tooltip;

  wrap.append(winWrap, fps, ue, tip);
  shell.stage.append(wrap);

  const paintSlider = (input: HTMLInputElement): void => {
    const min = Number(input.min || 0);
    const max = Number(input.max || 100);
    const pct = max > min ? ((Number(input.value) - min) / (max - min)) * 100 : 0;
    input.style.background = `linear-gradient(90deg, var(--accent) ${pct}%, var(--border) ${pct}%)`;
  };

  // Счетчик изменений basic-фазы живет и в advanced: экран пересоздается с нулем.
  let lastChanged = 0;

  const apply = (p: number): void => {
    const main = app.el.querySelector<HTMLElement>(".mock__main");
    if (!main) return;

    const wantAdvanced = p >= SWITCH_AT;
    const wantScreen: MockScreen = wantAdvanced ? "advanced" : "basic";
    // Источник истины: сам мок. Локальное состояние не храним, чтобы
    // ресайз через границу 900px и повторные init не рассинхронизировали сцену.
    if (wantScreen !== app.getScreen()) {
      app.setScreen(wantScreen);
      const content = main.firstElementChild;
      if (content) {
        content.classList.remove("modes__screen--in-r", "modes__screen--in-l");
        content.classList.add(wantAdvanced ? "modes__screen--in-r" : "modes__screen--in-l");
      }
    }

    if (!wantAdvanced) {
      const bp = clamp01(p / SWITCH_AT);
      // Контролы с data-demo-to приводятся к целевому значению от прогресса:
      // слайдер движется плавно, селект и свитч переключаются на пороге.
      const demoEls = Array.from(main.querySelectorAll<HTMLElement>("[data-demo-to]"));
      let changed = 0;
      for (const el of demoEls) {
        const at = Number(el.dataset.demoAt ?? 0.5);
        const from = el.dataset.demoFrom ?? "";
        const to = el.dataset.demoTo ?? "";
        let triggered = false;
        if (el instanceof HTMLInputElement && el.type === "range") {
          const k = clamp01((bp - at) / SLIDER_SPAN);
          const value = Math.round(lerp(Number(from), Number(to), k));
          el.value = String(value);
          paintSlider(el);
          const num = el.parentElement?.querySelector<HTMLInputElement>(".mock__range-num");
          if (num) num.value = String(value);
          triggered = k > 0;
        } else if (el instanceof HTMLSelectElement) {
          triggered = bp >= at;
          el.value = triggered ? to : from;
        } else {
          triggered = bp >= at;
          const on = (triggered ? to : from).toLowerCase() === "true";
          el.setAttribute("aria-checked", String(on));
        }
        if (triggered) changed += 1;
        el.closest(".mock__row")?.classList.toggle("is-changed", triggered);
      }
      const changesEl = main.querySelector<HTMLElement>(".mock__changes");
      if (changesEl) changesEl.textContent = t.mock.changes(changed);
      lastChanged = changed;

      fpsValue.textContent = String(Math.round(lerp(FPS_FROM, FPS_TO, bp)));
      fps.classList.add("is-on");
      ue.classList.remove("is-on");
      tip.classList.remove("is-on");
      return;
    }

    const ap = clamp01((p - SWITCH_AT - 0.05) / (1 - SWITCH_AT - 0.05));
    const rows = Array.from(main.querySelectorAll<HTMLElement>(".mock__row"));
    rows.forEach((row, i) => {
      row.classList.toggle("is-lit", ap >= ADV_LIT_START + i * ADV_LIT_STEP);
    });
    const changesEl = main.querySelector<HTMLElement>(".mock__changes");
    if (changesEl) changesEl.textContent = t.mock.changes(lastChanged);
    // Быстрый проскок мог не пройти basic-фазу: фиксируем финальное значение.
    fpsValue.textContent = String(FPS_TO);
    fps.classList.add("is-on");
    ue.classList.add("is-on");
    tip.classList.toggle("is-on", ap >= TOOLTIP_AT);
  };

  return {
    el: shell.section,
    init(mm) {
      mm.add(DESKTOP_QUERY, () => {
        const tl = gsap.timeline({
          defaults: { ease: "none" },
          scrollTrigger: {
            trigger: shell.section,
            start: "top top",
            end: "+=260%",
            pin: true,
            scrub: 0.6,
            anticipatePin: 1,
            onUpdate: (self) => apply(self.progress),
            onRefresh: (self) => apply(self.progress),
          },
        });
        tl.fromTo(winWrap, { y: 32, opacity: 0 }, { y: 0, opacity: 1, duration: 0.1, ease: "power1.out" }, 0);
        tl.to({}, { duration: 0.9 });

        // mm.revert() не откатывает прямые DOM-мутации apply(): сбрасываем вручную.
        return () => {
          apply(0);
          fps.classList.remove("is-on");
          ue.classList.remove("is-on");
          tip.classList.remove("is-on");
        };
      });
    },
    cleanup() {
      app.cleanup();
    },
  };
}
