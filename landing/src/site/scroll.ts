export function bindScrollReveal(): void {
  const nodes = document.querySelectorAll<HTMLElement>("[data-reveal]");
  nodes.forEach((n, i) => n.style.setProperty("--i", String(i % 10)));

  if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
    nodes.forEach((n) => n.classList.add("is-in"));
    return;
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
}

export function bindTopbar(header: HTMLElement): void {
  let solid = false;
  const tick = () => {
    const next = window.scrollY > 72;
    if (next === solid) return;
    solid = next;
    header.classList.toggle("is-solid", solid);
  };
  window.addEventListener("scroll", tick, { passive: true });
  tick();
}
