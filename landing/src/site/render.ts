import type { Copy } from "../content.ts";
import { screenshots } from "./screenshots.ts";
import { icon } from "./icons.ts";

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
  const home = `${base}${t.lang === "en" ? "en/" : ""}`;
  const github = `https://github.com/${config.repo}`;
  const release = `${github}/releases/tag/v${config.version}`;
  const download = `${github}/releases/download/v${config.version}/Game-Settings-Master_${config.version}_x64-setup.exe`;
  const button = `<a class="button button--primary" data-download href="${e(download)}">${icon("download")}<span>${e(t.download)}</span><span class="button__version">v${e(config.version)}</span></a>`;
  const shots = ["preview", "restore", "diagnostics"] as const;
  const shot = (
    name: (typeof shots)[number],
    title: string,
    narrow = false,
  ) => {
    const suffix = narrow ? "-mobile" : name === "preview" ? "-tall" : "";
    const key = `${name}${suffix}` as keyof typeof screenshots.ru.dark;
    const dimensions = screenshots[t.lang].dark[key];
    return `<picture class="theme-picture">
      ${["dark", "light"].map((theme) => `<source data-shot-theme="${theme}" media="${theme === "light" ? "not all" : "(max-width: 600px)"}" srcset="${base}screenshots/${t.lang}/${theme}/${name}-mobile.webp">`).join("")}
      <img src="${base}screenshots/${t.lang}/dark/${name}${suffix}.webp" data-shot="${name}" data-shot-variant="${suffix}" data-shot-base="${base}screenshots/${t.lang}" width="${dimensions[0]}" height="${dimensions[1]}" loading="lazy" decoding="async" alt="${e(title)}">
    </picture>`;
  };
  return `
  <a class="skip-link" href="#main">${e(t.skip)}</a>
  <header class="header"><div class="header__inner">
    <a class="brand" href="${home}" aria-label="Game Settings Master"><img src="${base}logo.png" width="36" height="36" alt=""><span>Game Settings Master</span></a>
    <nav class="header__nav" aria-label="${t.lang === "ru" ? "Навигация" : "Navigation"}"><a href="#features">${e(t.nav[0])}</a><a href="#compatibility">${e(t.nav[1])}</a><a href="#faq">FAQ</a></nav>
    <div class="header__tools">
      <button type="button" id="theme-toggle" class="icon-button" hidden data-label-light="${e(t.theme.light)}" data-label-dark="${e(t.theme.dark)}" aria-label="${e(t.theme.light)}">${icon("sun", "theme-sun")}${icon("moon", "theme-moon")}</button>
      <a class="language" href="${base}${t.lang === "ru" ? "en/" : ""}" lang="${t.lang === "ru" ? "en" : "ru"}" aria-label="${t.lang === "ru" ? "Switch to English" : "Переключить на русский"}">${t.lang === "ru" ? "EN" : "RU"}</a>
      <a class="header__download" data-download href="${e(download)}">${icon("download")}<span>${t.lang === "ru" ? "Скачать" : "Download"}</span></a>
    </div>
  </div></header>
  <main id="main">
    <section class="hero wide-container" aria-labelledby="hero-title">
      <div class="hero__copy">
        <p class="eyebrow">${e(t.kicker)}</p>
        <h1 id="hero-title">${e(t.hero)}</h1><p class="lead">${e(t.intro)}</p>
        <div class="hero__actions">${button}<a class="text-link" href="#features">${e(t.heroLink)}${icon("arrow")}</a></div>
        <ul class="trust">${t.trust.map((item, index) => `<li>${icon("check")}${index === 1 ? `<a href="${github}" title="${e(t.source)}">${e(item)}</a>` : `<span>${e(item)}</span>`}</li>`).join("")}</ul>
        <p class="install-note">${e(t.warning)} <a href="#smartscreen">${e(t.help)}</a></p>
      </div>
      <div class="hero__visual">
        <div class="demo" id="interactive-demo" aria-label="${e(t.demo.title)}">
          <div class="demo__top"><span class="demo__mark">${icon("sliders")}</span><div><p class="eyebrow">${e(t.demo.label)}</p><h2>${e(t.demo.title)}</h2></div><span class="demo__tag">UE5</span></div>
          <div id="demo-content"><div class="static-diff"><span>GameUserSettings.ini</span><code>bUseVSync</code><div><span>${e(t.demo.before)} <b>True</b></span>${icon("arrow")}<span>${e(t.demo.after)} <b>False</b></span></div></div><noscript><p>${t.lang === "ru" ? "Для управления примером включите JavaScript." : "Enable JavaScript to interact with this example."}</p></noscript></div>
          <p class="demo__disclaimer">${e(t.demo.disclaimer)}</p>
        </div>
      </div>
    </section>
    <section class="workflow container" aria-labelledby="workflow-title"><h2 id="workflow-title">${e(t.workflow)}</h2><ol>${t.steps.map((step, i) => `<li><span class="step-number">${i + 1}</span><span>${e(step)}</span>${i < 3 ? icon("arrow", "step-arrow") : ""}</li>`).join("")}</ol></section>
    <section class="features container" id="features" aria-labelledby="features-title">
      <div class="section-heading"><p class="eyebrow">${e(t.featuresKicker)}</p><h2 id="features-title">${e(t.featuresTitle)}</h2></div>
      <div class="feature-grid">${t.features.map((feature, i) => `<article class="feature feature--${i + 1}"><div class="feature__copy"><span class="feature__icon">${icon(shots[i])}</span><h3>${e(feature.title)}</h3><p>${e(feature.text)}</p></div><figure class="feature__visual">${shot(shots[i], feature.title, i === 1)}<figcaption>${e(t.caption)}</figcaption></figure><p class="feature__note">${e(feature.note)}</p></article>`).join("")}</div>
    </section>
    <section class="extras container" aria-labelledby="extras-title"><div class="section-heading"><p class="eyebrow">${e(t.toolsKicker)}</p><h2 id="extras-title">${e(t.extrasTitle)}</h2></div><div class="extras__grid">${t.extras.map((item, i) => `<article><span class="extra-symbol">${icon(["code", "preset", "gpu", "input"][i])}</span><h3>${e(item.title)}</h3><p>${e(item.text)}</p></article>`).join("")}</div></section>
    <section class="compat container" id="compatibility" aria-labelledby="compat-title"><div class="compat__intro"><p class="eyebrow">${e(t.compatKicker)}</p><h2 id="compat-title">${e(t.compatTitle)}</h2><p>${e(t.compatText)}</p><dl class="compat__strip">${["Unreal Engine 4 · 5", t.lang === "ru" ? "Steam · Epic · GOG · Xbox · Вручную" : "Steam · Epic · GOG · Xbox · Manual", "NVIDIA · AMD · Intel"].map((value, i) => `<div><dt>${e(t.compatLabels[i])}</dt><dd>${e(value)}</dd></div>`).join("")}</dl></div><div class="limits">${t.limits.map((item) => `<details><summary>${e(item.title)}${icon("chevron")}</summary><p>${e(item.text)}</p></details>`).join("")}</div></section>
    <section class="faq container" id="faq" aria-labelledby="faq-title"><div><p class="eyebrow">FAQ</p><h2 id="faq-title">${e(t.faqTitle)}</h2><p>${e(t.faqIntro)}</p></div><div class="faq__list">${t.faq.map((item) => `<details id="${item.id}"><summary>${e(item.question)}${icon("chevron")}</summary><p>${e(item.text)}</p></details>`).join("")}</div></section>
    <section class="closing wide-container"><div class="closing__symbol" aria-hidden="true">${icon("game")}</div><div class="closing__copy"><p class="eyebrow">GAME SETTINGS MASTER · v${e(config.version)}</p><h2>${e(t.final)}</h2><p>${e(t.finalText)}</p></div><div class="closing__actions">${button}<a class="text-link" href="${release}">${e(t.release)}${icon("northeast")}</a></div></section>
  </main>
  <footer class="footer container"><div class="footer__brand"><a class="brand" href="${home}"><img src="${base}logo.png" alt="" width="36" height="36"><span>Game Settings Master</span></a><span class="footer__version">v${e(config.version)} · Windows x64</span></div><div class="footer__links"><a href="${github}">${icon("github")}GitHub${icon("northeast")}</a><a href="https://www.donationalerts.com/r/mike_saito">${e(t.donate)}${icon("northeast")}</a></div><p>${e(t.donateText)}</p><p class="footer__license">MIT · ${t.lang === "ru" ? "Бесплатно и с открытым кодом" : "Free and open source"}</p></footer>`;
}
