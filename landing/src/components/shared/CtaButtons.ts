import { githubUrl } from "../../lib/site";
import type { LocaleStrings } from "../../i18n/types";
import { openDownloadModal } from "./DownloadModal";

export function createCtaButtons(
  t: LocaleStrings,
  layout: "hero" | "download" = "hero",
): HTMLElement {
  const wrap = document.createElement("div");
  wrap.className =
    layout === "hero" ? "hero__cta" : "download__cta";

  const github = document.createElement("a");
  github.className = "btn btn--secondary";
  github.href = githubUrl;
  github.target = "_blank";
  github.rel = "noopener noreferrer";
  github.textContent = t.download.githubButton;

  const download = document.createElement("button");
  download.type = "button";
  download.className = "btn btn--primary";
  download.textContent = t.download.button;
  download.addEventListener("click", () => openDownloadModal(t));

  wrap.append(download, github);
  return wrap;
}
