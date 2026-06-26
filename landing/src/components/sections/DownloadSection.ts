import { APP_VERSION, donateUrl } from "../../lib/site";
import type { LocaleStrings } from "../../i18n/types";
import { createCtaButtons } from "../shared/CtaButtons";

export function createDownloadSection(t: LocaleStrings): HTMLElement {
  const section = document.createElement("section");
  section.className = "download reveal-stagger";
  section.id = "download";
  section.innerHTML = `
    <div class="download__glow" aria-hidden="true"></div>
    <div class="download__inner page-wrap">
      <div class="download__card">
        <p class="download__kicker">${t.siteName}</p>
        <h2 class="download__title">${t.download.title}</h2>
        <p class="download__text">${t.download.subtitle} · v${APP_VERSION}</p>
      </div>
    </div>
  `;
  const card = section.querySelector(".download__card");
  card?.appendChild(createCtaButtons(t, "download"));

  const donate = document.createElement("div");
  donate.className = "download__donate";
  donate.innerHTML = `
    <p class="download__donate-title">${t.donate.title}</p>
    <p class="download__donate-text">${t.donate.text}</p>
    <a class="btn btn--ghost download__donate-btn" href="${donateUrl}" target="_blank" rel="noopener noreferrer">${t.donate.button}</a>
  `;
  card?.appendChild(donate);

  return section;
}
