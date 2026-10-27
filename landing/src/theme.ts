export function initTheme(): () => void {
  const button = document.getElementById(
    "theme-toggle",
  ) as HTMLButtonElement | null;
  if (!button) return () => {};
  const media = matchMedia("(prefers-color-scheme: light)");
  let explicit = ["dark", "light"].includes(
    document.documentElement.dataset.themeMode ?? "",
  );
  const apply = (theme: "dark" | "light") => {
    document.documentElement.dataset.theme = theme;
    const label =
      theme === "dark" ? button.dataset.labelLight! : button.dataset.labelDark!;
    button.setAttribute("aria-label", label);
    button.title = label;
    document
      .querySelector('meta[name="theme-color"]')
      ?.setAttribute("content", theme === "light" ? "#fff9ef" : "#141310");
    for (const image of document.querySelectorAll<HTMLImageElement>(
      "[data-shot]",
    )) {
      const picture = image.closest("picture");
      const path = `${image.dataset.shotBase}/${theme}/${image.dataset.shot}`;
      for (const source of picture?.querySelectorAll<HTMLSourceElement>(
        "[data-shot-theme]",
      ) ?? []) {
        source.media =
          source.dataset.shotTheme === theme ? "(max-width: 600px)" : "not all";
      }
      image.src = `${path}${image.dataset.shotVariant ?? ""}.webp`;
    }
  };
  const change = () => {
    const theme =
      document.documentElement.dataset.theme === "light" ? "dark" : "light";
    explicit = true;
    document.documentElement.dataset.themeMode = theme;
    try {
      localStorage.setItem("gsm-site-theme", theme);
    } catch {
      /* Optional storage. */
    }
    apply(theme);
  };
  const system = () => {
    if (!explicit) apply(media.matches ? "light" : "dark");
  };
  apply(document.documentElement.dataset.theme === "light" ? "light" : "dark");
  button.hidden = false;
  button.addEventListener("click", change);
  media.addEventListener("change", system);
  return () => {
    button.removeEventListener("click", change);
    media.removeEventListener("change", system);
  };
}
