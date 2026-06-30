import type { LocaleStrings } from "../i18n/types";
import { runBoot } from "./boot";
import { mountScene } from "./scene";
import { bindScrollReveal, bindTopbar } from "./scroll";
import { buildHeader } from "../sections/header";
import { buildHero } from "../sections/hero";
import { buildTicker } from "../sections/ticker";
import { buildModes } from "../sections/modes";
import { buildFeatures } from "../sections/features";
import { buildFaq } from "../sections/faq";
import { buildDownload } from "../sections/download";
import { buildFooter } from "../sections/footer";

export function assemblePage(t: LocaleStrings): HTMLElement {
  const shell = document.createElement("div");
  shell.className = "shell";

  const main = document.createElement("main");
  main.append(
    buildHero(t),
    buildTicker(t),
    buildModes(t),
    buildFeatures(t),
    buildFaq(t),
    buildDownload(t),
    buildFooter(t),
  );

  shell.append(main);
  return shell;
}

export function initSite(t: LocaleStrings): () => void {
  const cleanups: Array<() => void> = [];
  const app = document.getElementById("app");
  if (!app) return () => {};

  const shell = assemblePage(t);
  const header = buildHeader(t);
  app.append(header, shell);

  cleanups.push(mountScene());
  cleanups.push(runBoot());
  bindTopbar(header);
  bindScrollReveal();

  return () => {
    for (const c of cleanups) c();
  };
}
