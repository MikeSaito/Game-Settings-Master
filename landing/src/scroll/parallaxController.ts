export function initParallax(): void {
  const layers = Array.from(document.querySelectorAll<HTMLElement>("[data-parallax]"));
  if (!layers.length) return;

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

  let ticking = false;

  const update = () => {
    ticking = false;
    const scrollY = window.scrollY;
    for (const layer of layers) {
      const speed = Number(layer.dataset.parallax) || 0.25;
      layer.style.transform = `translate3d(0, ${scrollY * speed}px, 0) scale(1.08)`;
    }
  };

  window.addEventListener(
    "scroll",
    () => {
      if (ticking) return;
      ticking = true;
      requestAnimationFrame(update);
    },
    { passive: true },
  );

  update();
}
