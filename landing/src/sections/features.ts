import type { LocaleStrings } from "../i18n/types";
import { makeShot } from "../ui/shot";

export function buildFeatures(t: LocaleStrings): HTMLElement {
  const el = document.createElement("section");
  el.className = "block";
  el.id = "features";

  const head = document.createElement("header");
  head.className = "block__head wrap";
  head.setAttribute("data-reveal", "");
  head.innerHTML = `
    <p class="block__eyebrow">${t.lang === "ru" ? "Возможности" : "Capabilities"}</p>
    <h2 class="block__title">${t.lang === "ru" ? "Всё для тонкой настройки" : "Everything for fine control"}</h2>
  `;

  const list = document.createElement("div");
  list.className = "timeline wrap";

  for (const f of t.features) {
    const row = document.createElement("article");
    row.className = "story";
    row.id = f.id;
    row.setAttribute("data-reveal", "");
    row.innerHTML = `
      <div class="story__num">${f.step}</div>
      <div class="story__copy">
        <h3>${f.title}</h3>
        <p>${f.text}</p>
      </div>
      <div class="story__shot"></div>
    `;
    row.querySelector(".story__shot")?.append(makeShot(f.shot, f.title));
    list.append(row);
  }

  el.append(head, list);
  return el;
}
