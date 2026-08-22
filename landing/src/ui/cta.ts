import { githubUrl } from "../lib/site";
import type { LocaleStrings } from "../i18n/types";
import { showDownloadModal } from "./downloadModal";

export function makeActions(t: LocaleStrings): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className = "actions";

  const dl = document.createElement("button");
  dl.type = "button";
  dl.className = "btn btn--fill";
  dl.textContent = t.download.button;
  dl.addEventListener("click", () => showDownloadModal(t));

  const gh = document.createElement("a");
  gh.className = "btn btn--line";
  gh.href = githubUrl;
  gh.target = "_blank";
  gh.rel = "noopener noreferrer";
  gh.textContent = t.download.githubButton;

  wrap.append(dl, gh);
  return wrap;
}
