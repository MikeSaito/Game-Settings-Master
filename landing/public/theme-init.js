// Apply a saved or explicit palette before the page is painted.
(() => {
  const root = document.documentElement;
  const query = new URLSearchParams(location.search).get("theme");
  let saved;
  try {
    saved = localStorage.getItem("gsm-site-theme");
  } catch {}
  const preference = query === "dark" || query === "light" ? query : saved;
  const explicit = preference === "dark" || preference === "light";
  root.dataset.themeMode = explicit ? preference : "system";
  root.dataset.theme = explicit
    ? preference
    : matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
})();
