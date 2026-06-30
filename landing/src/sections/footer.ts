import { APP_VERSION, donateUrl, githubUrl, telegramUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

export function buildFooter(t: LocaleStrings): HTMLElement {
  const el = document.createElement("footer");
  el.className = "end";
  el.innerHTML = `
    <div class="end__row wrap">
      <span>${t.footer.version(APP_VERSION)}</span>
      <div>
        <a href="${githubUrl}" target="_blank" rel="noopener noreferrer">GitHub</a>
        ·
        <a href="${telegramUrl}" target="_blank" rel="noopener noreferrer">${t.footer.telegramLink}</a>
        ·
        <a href="${donateUrl}" target="_blank" rel="noopener noreferrer">${t.footer.donateLink}</a>
      </div>
    </div>
  `;
  return el;
}
