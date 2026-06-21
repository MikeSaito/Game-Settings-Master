let revealObserver: IntersectionObserver | null = null;

export function initStaggeredReveal(): void {
  const items = Array.from(document.querySelectorAll<HTMLElement>(".reveal-stagger"));
  items.forEach((item, index) => {
    item.style.setProperty("--reveal-index", String(index % 5));
  });

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    items.forEach((item) => item.classList.add("is-visible"));
    return;
  }

  revealObserver?.disconnect();
  revealObserver = new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        entry.target.classList.add("is-visible");
        revealObserver?.unobserve(entry.target);
      }
    },
    { threshold: 0.12, rootMargin: "0px 0px -8% 0px" },
  );

  items.forEach((item) => revealObserver?.observe(item));
}
