import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);
ScrollTrigger.config({ ignoreMobileResize: true });

export const DESKTOP_QUERY = "(min-width: 900px) and (prefers-reduced-motion: no-preference)";
export const FALLBACK_QUERY = "(max-width: 899.98px), (prefers-reduced-motion: reduce)";

export { gsap, ScrollTrigger };

export interface Scene {
  el: HTMLElement;
  init: (mm: gsap.MatchMedia) => void;
  cleanup: () => void;
}

export function createMatchMedia(): gsap.MatchMedia {
  return gsap.matchMedia();
}

/**
 * Бесконечные CSS-анимации (пульс CTA, подсказка) запускаются только
 * пока элемент во вьюпорте: класс className ставится и снимается наблюдателем.
 */
export function animateWhenVisible(root: HTMLElement, selector: string, className: string): () => void {
  const targets = Array.from(root.querySelectorAll<HTMLElement>(selector));
  if (targets.length === 0) return () => {};
  if (!("IntersectionObserver" in window)) {
    for (const t of targets) t.classList.add(className);
    return () => {};
  }
  const io = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        entry.target.classList.toggle(className, entry.isIntersecting);
      }
    },
    { threshold: 0.1 },
  );
  for (const t of targets) io.observe(t);
  return () => io.disconnect();
}

/**
 * Фолбэк-режим: элементы с data-reveal проявляются при входе во вьюпорт.
 * При reduced-motion базовый CSS обнуляет transition, появление мгновенное.
 */
export function observeReveals(root: HTMLElement): () => void {
  const targets = Array.from(root.querySelectorAll<HTMLElement>("[data-reveal]"));
  if (targets.length === 0) return () => {};
  if (!("IntersectionObserver" in window)) {
    for (const t of targets) t.classList.add("is-visible");
    return () => {};
  }
  const io = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (entry.isIntersecting) {
          entry.target.classList.add("is-visible");
          io.unobserve(entry.target);
        }
      }
    },
    { threshold: 0.2, rootMargin: "0px 0px -8% 0px" },
  );
  for (const t of targets) io.observe(t);
  return () => io.disconnect();
}
