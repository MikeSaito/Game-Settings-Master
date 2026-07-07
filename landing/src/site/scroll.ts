function onScrollFrame(cb: () => void): () => void {
  let raf = 0;
  const tick = () => {
    raf = 0;
    cb();
  };
  const onScroll = () => {
    if (raf) return;
    raf = requestAnimationFrame(tick);
  };
  window.addEventListener("scroll", onScroll, { passive: true });
  return () => {
    cancelAnimationFrame(raf);
    window.removeEventListener("scroll", onScroll);
  };
}

export function bindScrollReveal(): () => void {
  const nodes = document.querySelectorAll<HTMLElement>("[data-reveal]");
  nodes.forEach((n, i) => n.style.setProperty("--i", String(i % 10)));

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    nodes.forEach((n) => n.classList.add("is-in"));
    return () => {};
  }

  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        e.target.classList.add("is-in");
        io.unobserve(e.target);
      }
    },
    { threshold: 0.1, rootMargin: "0px 0px -4% 0px" },
  );

  nodes.forEach((n) => io.observe(n));
  return () => io.disconnect();
}

export function bindTopbar(header: HTMLElement): () => void {
  let solid = window.scrollY > 72;

  header.classList.toggle("is-solid", solid);

  const tick = () => {
    const next = window.scrollY > 72;
    if (next === solid) return;
    solid = next;
    header.classList.toggle("is-solid", solid);
  };

  const stopScroll = onScrollFrame(tick);
  return stopScroll;
}
