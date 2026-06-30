import { assetPath } from "../lib/site";

function reduced(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function runBoot(): () => void {
  document.body.classList.add("is-booting");

  if (reduced()) {
    document.body.classList.remove("is-booting");
    document.body.classList.add("is-live");
    return () => {};
  }

  const el = document.createElement("div");
  el.className = "boot";
  el.innerHTML = `
    <div class="boot__ring"></div>
    <img class="boot__logo" src="${assetPath("logo.svg")}" width="64" height="64" alt="" />
    <p class="boot__name">Game Settings Master</p>
    <div class="boot__bar"><i></i></div>
  `;
  document.body.append(el);

  const timer = window.setTimeout(() => {
    el.classList.add("is-out");
    document.body.classList.remove("is-booting");
    document.body.classList.add("is-live");
    window.setTimeout(() => el.remove(), 720);
  }, 1080);

  return () => {
    clearTimeout(timer);
    el.remove();
    document.body.classList.remove("is-booting", "is-live");
  };
}
