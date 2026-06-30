import { APP_VERSION, donateUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";
import { makeActions } from "../ui/cta";

export function buildDownload(t: LocaleStrings): HTMLElement {
  const el = document.createElement("section");
  el.className = "acquire";
  el.id = "download";

  el.innerHTML = `
    <div class="acquire__glow" aria-hidden="true"></div>
    <div class="acquire__inner wrap" data-reveal>
      <p class="acquire__kicker">${t.siteName}</p>
      <h2 class="acquire__title">${t.download.title}</h2>
      <p class="acquire__sub">${t.download.subtitle} · v${APP_VERSION}</p>
    </div>
  `;

  el.querySelector(".acquire__inner")?.append(makeActions(t, "download"));

  const donate = document.createElement("aside");
  donate.className = "acquire__donate";
  donate.innerHTML = `
    <p class="acquire__donate-tag">${t.lang === "ru" ? "по желанию" : "optional"}</p>
    <h3>${t.donate.title}</h3>
    <p>${t.donate.text}</p>
    <a class="btn btn--donate" href="${donateUrl}" target="_blank" rel="noopener noreferrer">${t.donate.button}</a>
  `;
  el.querySelector(".acquire__inner")?.append(donate);

  return el;
}
