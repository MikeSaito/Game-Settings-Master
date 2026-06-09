export function getScrollProgress(): number {
  const doc = document.documentElement;
  const scrollable = doc.scrollHeight - window.innerHeight;
  if (scrollable <= 0) return 1;
  return Math.min(1, Math.max(0, window.scrollY / scrollable));
}

export function prefersReducedMotion(): boolean {
  return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

export function useScrollProgress(onProgress: (progress: number) => void): () => void {
  let ticking = false;

  const update = () => {
    ticking = false;
    const progress = prefersReducedMotion() ? 1 : getScrollProgress();
    onProgress(progress);
  };

  const onScroll = () => {
    if (!ticking) {
      ticking = true;
      requestAnimationFrame(update);
    }
  };

  const motionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");
  const onMotionChange = () => update();

  window.addEventListener("scroll", onScroll, { passive: true });
  window.addEventListener("resize", onScroll, { passive: true });
  motionQuery.addEventListener("change", onMotionChange);
  update();

  return () => {
    window.removeEventListener("scroll", onScroll);
    window.removeEventListener("resize", onScroll);
    motionQuery.removeEventListener("change", onMotionChange);
  };
}
