import { APP_VERSION, donateUrl } from "../../lib/site";
import type { LocaleStrings } from "../../i18n/types";

export function createSiteFooter(t: LocaleStrings): HTMLElement {
  const footer = document.createElement("footer");
  footer.className = "site-footer";
  footer.innerHTML = `
    <p>${t.footer.version(APP_VERSION)}</p>
    <p class="site-footer__donate">
      <a href="${donateUrl}" target="_blank" rel="noopener noreferrer">${t.footer.donateLink}</a>
    </p>
  `;
  return footer;
}
