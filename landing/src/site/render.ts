import type { Copy } from "../content.ts";
import { screenshots } from "./screenshots.ts";

export interface SiteConfig {
  version: string;
  base: string;
  repo: string;
}
export const escapeHtml = (value: string) =>
  value.replace(
    /[&<>"']/g,
    (char) =>
      ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[
        char
      ]!,
  );

export function renderSite(t: Copy, config: SiteConfig): string {
  const e = escapeHtml;
  const base = config.base.endsWith("/") ? config.base : `${config.base}/`;
  const github = `https://github.com/${config.repo}`;
  const release = `${github}/releases/tag/v${config.version}`;
  const download = `${github}/releases/download/v${config.version}/Game-Settings-Master_${config.version}_x64-setup.exe`;
  const button = `<a class="button button--primary" data-download href="${e(download)}"><span>${e(t.download)}</span><span class="button__version">v${e(config.version)} ↗</span></a>`;
  const shots = ["preview", "restore", "diagnostics"] as const;
  const sizes = shots.map((name) => screenshots[t.lang][name]);
  const mobileSizes = shots.map(
    (name) => screenshots[t.lang][`${name}-mobile`],
  );
  return `
  <a class="skip-link" href="#main">${e(t.skip)}</a>
  <header class="header"><div class="header__inner">
    <a class="brand" href="${base}" aria-label="Game Settings Master"><img src="${base}logo.svg" width="30" height="30" alt=""><span>Game Settings Master</span></a>
    <nav class="header__nav" aria-label="${t.lang === "ru" ? "Навигация" : "Navigation"}"><a href="#features">${e(t.nav[0])}</a><a href="#compatibility">${e(t.nav[1])}</a></nav>
    <div class="header__tools"><a class="language" href="${base}${t.lang === "ru" ? "en/" : ""}" lang="${t.lang === "ru" ? "en" : "ru"}" aria-label="${t.lang === "ru" ? "Switch to English" : "Переключить на русский"}">${t.lang === "ru" ? "EN" : "RU"}</a><a class="header__download" data-download href="${e(download)}">${t.lang === "ru" ? "Скачать" : "Download"}<span aria-hidden="true"> ↗</span></a></div>
  </div></header>
  <main id="main">
    <section class="hero container" aria-labelledby="hero-title">
      <div class="hero__copy"><p class="eyebrow"><span class="status-dot"></span> GAME SETTINGS MASTER / WINDOWS</p><h1 id="hero-title">${e(t.hero)}</h1><p class="lead">${e(t.intro)}</p>
        <div class="hero__actions">${button}<a class="source-link" href="${github}">${e(t.source)} <span aria-hidden="true">↗</span></a></div>
        <ul class="trust">${t.trust.map((item) => `<li>${e(item)}</li>`).join("")}</ul>
        <p class="install-note">${e(t.warning)} <a href="#smartscreen">${e(t.help)}</a></p>
      </div>
      <div class="hero__visual"><div class="visual-orbit" aria-hidden="true"></div><div class="demo" id="interactive-demo" aria-label="${e(t.demo.title)}"><div class="demo__top"><span class="eyebrow">${e(t.demo.label)}</span><span class="demo__file">.ini</span></div><h2>${e(t.demo.title)}</h2><p class="demo__disclaimer">${e(t.demo.disclaimer)}</p><div id="demo-content"><div class="static-diff"><span>GameUserSettings.ini</span><code>bUseVSync</code><div><span>${e(t.demo.before)} <b>True</b></span><span aria-hidden="true">→</span><span>${e(t.demo.after)} <b>False</b></span></div></div><noscript><p>${t.lang === "ru" ? "Для управления примером включите JavaScript." : "Enable JavaScript to interact with this example."}</p></noscript></div></div></div>
    </section>
    <section class="workflow container" aria-labelledby="workflow-title"><h2 id="workflow-title">${e(t.workflow)}</h2><ol>${t.steps.map((step, i) => `<li><span class="step-number">0${i + 1}</span><span>${e(step)}</span></li>`).join("")}</ol></section>
    <section class="features container" id="features" aria-labelledby="features-title"><div class="section-heading"><p class="eyebrow">${e(t.featuresKicker)}</p><h2 id="features-title">${e(t.featuresTitle)}</h2></div>
      ${t.features.map((feature, i) => `<article class="feature feature--${i + 1}"><div class="feature__copy"><span class="feature__number" aria-hidden="true">0${i + 1}</span><h3>${e(feature.title)}</h3><p>${e(feature.text)}</p><p class="feature__note">${e(feature.note)}</p></div><figure class="feature__visual"><picture><source media="(max-width: 600px)" srcset="${base}screenshots/${t.lang}/${shots[i]}-mobile.webp" width="${mobileSizes[i][0]}" height="${mobileSizes[i][1]}"><img src="${base}screenshots/${t.lang}/${shots[i]}.webp" width="${sizes[i][0]}" height="${sizes[i][1]}" loading="lazy" decoding="async" alt="${e(feature.title)}"></picture><figcaption>${e(t.caption)}</figcaption></figure></article>`).join("")}
    </section>
    <section class="extras container" aria-labelledby="extras-title"><h2 id="extras-title">${e(t.extrasTitle)}</h2><div class="extras__grid">${t.extras.map((item, i) => `<article><span class="extra-symbol" aria-hidden="true">${["{ }", "▤", "⌘"][i]}</span><h3>${e(item.title)}</h3><p>${e(item.text)}</p></article>`).join("")}</div></section>
    <section class="compat container" id="compatibility" aria-labelledby="compat-title"><div class="section-heading"><p class="eyebrow">UE4 / UE5</p><h2 id="compat-title">${e(t.compatTitle)}</h2><p class="lead">${e(t.compatText)}</p></div><dl class="compat__strip">${["Unreal Engine 4 · 5", t.lang === "ru" ? "Steam · Epic · GOG · Xbox · Вручную" : "Steam · Epic · GOG · Xbox · Manual", "NVIDIA · AMD · Intel"].map((value, i) => `<div><dt>${e(t.compatLabels[i])}</dt><dd>${e(value)}</dd></div>`).join("")}</dl><div class="limits">${t.limits.map((item) => `<article><h3>${e(item.title)}</h3><p>${e(item.text)}</p></article>`).join("")}</div></section>
    <section class="closing container"><div><p class="eyebrow">GAME SETTINGS MASTER / v${e(config.version)}</p><h2>${e(t.final)}</h2><p class="lead">${e(t.finalText)}</p></div><div class="closing__actions">${button}<a href="${release}">${e(t.release)} ↗</a></div></section>
    <section class="faq container" aria-labelledby="faq-title"><h2 id="faq-title">${e(t.faqTitle)}</h2><div>${t.faq.map((item) => `<details id="${item.id}"><summary>${e(item.question)}</summary><p>${e(item.text)}</p></details>`).join("")}</div></section>
  </main>
  <footer class="footer container"><div class="footer__brand"><img src="${base}logo.svg" alt="" width="28" height="28"><span>Game Settings Master <span class="footer__version">v${e(config.version)}</span></span></div><div class="footer__links"><a href="${github}">GitHub ↗</a><a href="https://www.donationalerts.com/r/mike_saito">${e(t.donate)} ↗</a></div><p>${e(t.donateText)}</p></footer>`;
}
