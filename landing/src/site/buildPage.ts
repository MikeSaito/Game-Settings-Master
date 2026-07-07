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

export function assemblePage(t: LocaleStrings): { shell: HTMLElement; cleanups: Array<() => void> } {
  const cleanups: Array<() => void> = [];
  const shell = document.createElement("div");
  shell.className = "shell";

  const main = document.createElement("main");
  const ticker = buildTicker(t);
  cleanups.push(ticker.cleanup);
  main.append(
    buildHero(t),
    ticker.el,
    buildModes(t),
    buildFeatures(t),
    buildFaq(t),
    buildDownload(t),
    buildFooter(t),
  );

  shell.append(main);
  return { shell, cleanups };
}

export function initSite(t: LocaleStrings): () => void {
  const cleanups: Array<() => void> = [];
  const app = document.getElementById("app");
  if (!app) return () => {};

  const { shell, cleanups: pageCleanups } = assemblePage(t);
  cleanups.push(...pageCleanups);
  const header = buildHeader(t);
  app.append(header, shell);

  const gate = shell.querySelector<HTMLElement>(".gate");
  if (gate) {
    cleanups.push(mountScene(gate).cleanup);
  }

  cleanups.push(runBoot());
  cleanups.push(bindTopbar(header));
  cleanups.push(bindScrollReveal());

  return () => {
    for (const c of cleanups) c();
  };
}
