import { APP_VERSION, donateUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";
import { createCtaButtons } from "./CtaButtons";

export function createDownloadSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "download page-wrap";
  section.id = "download";

  const card = document.createElement("div");
  card.className = "download__card";
  card.innerHTML = `
    <h2 class="download__title">${t.download.title}</h2>
    <p class="download__text">${t.download.subtitle} · v${APP_VERSION}</p>
    <div class="download__engines">
      ${t.engineTags.map((tag) => `<span class="engine-tag">${tag}</span>`).join("")}
    </div>
  `;
  card.appendChild(createCtaButtons(t, "download"));

  const donate = document.createElement("div");
  donate.className = "download__donate";
  donate.innerHTML = `
    <p class="download__donate-title">${t.donate.title}</p>
    <p class="download__donate-text">${t.donate.text}</p>
    <a class="btn btn--ghost download__donate-btn" href="${donateUrl}" target="_blank" rel="noopener noreferrer">${t.donate.button}</a>
  `;
  card.appendChild(donate);

  section.appendChild(card);

  return section;
}
