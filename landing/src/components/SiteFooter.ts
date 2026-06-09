import { APP_VERSION } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";

export function createSiteFooter(t: LocaleStrings): HTMLElement {
  const footer = document.createElement("footer");
  footer.className = "site-footer";
  footer.innerHTML = `<p>${t.footer.version(APP_VERSION)}</p>`;
  return footer;
}
