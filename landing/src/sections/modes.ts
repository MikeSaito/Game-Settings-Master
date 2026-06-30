import type { LocaleStrings } from "../i18n/types";
import { localeShot } from "../lib/shots";
import { makeShot } from "../ui/shot";

function modeCard(
  label: string,
  title: string,
  text: string,
  bullets: string[],
  pro: boolean,
): string {
  return `
    <article class="mode${pro ? " mode--pro" : ""}">
      <div class="mode__tag">${label}</div>
      <h3>${title}</h3>
      <p>${text}</p>
      <ul>${bullets.map((b) => `<li>${b}</li>`).join("")}</ul>
    </article>
  `;
}

export function buildModes(t: LocaleStrings): HTMLElement {
  const el = document.createElement("section");
  el.className = "block";
  el.id = "modes";
  const ba = t.basicAdvanced;

  el.innerHTML = `
    <div class="wrap">
      <header class="block__head block__head--wide" data-reveal>
        <p class="block__eyebrow">${t.lang === "ru" ? "Режимы" : "Modes"}</p>
        <h2 class="block__title">${ba.title}</h2>
        <p class="block__lead">${ba.text}</p>
      </header>
      <div class="workbench" data-reveal>
        <div class="workbench__frame">
          <div class="workbench__switch" role="tablist">
            <button type="button" role="tab" aria-selected="true" data-mode="basic">${ba.basic.label}</button>
            <button type="button" role="tab" aria-selected="false" data-mode="advanced">${ba.advanced.label}</button>
          </div>
          <div class="workbench__viewport" data-viewport></div>
        </div>
        <div class="workbench__cols">
          ${modeCard(ba.basic.label, ba.basic.title, ba.basic.text, ba.basic.bullets, false)}
          ${modeCard(ba.advanced.label, ba.advanced.title, ba.advanced.text, ba.advanced.bullets, true)}
        </div>
      </div>
    </div>
  `;

  const viewport = el.querySelector<HTMLElement>("[data-viewport]")!;
  const tabs = el.querySelectorAll<HTMLButtonElement>("[data-mode]");
  const shots = {
    basic: makeShot(localeShot(t.lang, "editor-basic.png"), ba.basic.title),
    advanced: makeShot(localeShot(t.lang, "editor-advanced.png"), ba.advanced.title),
  };

  const pick = (mode: "basic" | "advanced") => {
    viewport.classList.add("is-swap");
    window.setTimeout(() => {
      viewport.replaceChildren(shots[mode]);
      viewport.classList.remove("is-swap");
    }, 160);
    tabs.forEach((tab) => {
      const on = tab.dataset.mode === mode;
      tab.setAttribute("aria-selected", String(on));
    });
  };

  pick("basic");
  tabs.forEach((tab) => {
    tab.addEventListener("click", () => {
      const m = tab.dataset.mode as "basic" | "advanced";
      if (m) pick(m);
    });
  });

  return el;
}
