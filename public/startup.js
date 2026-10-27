// Keep this bootstrap independent of the application bundle and network fonts.
(() => {
  let settings = {};
  let storedLanguage;
  try {
    storedLanguage = localStorage.getItem("uesm:language");
    settings =
      JSON.parse(localStorage.getItem("gsm-app-settings") || "null") || {};
  } catch {
    /* System preferences work when storage is unavailable. */
  }
  const root = document.documentElement;
  const explicitTheme = settings.theme === "light" || settings.theme === "dark";
  root.dataset.theme = explicitTheme
    ? settings.theme
    : matchMedia("(prefers-color-scheme: light)").matches
      ? "light"
      : "dark";
  root.dataset.reducedMotion =
    settings.reducedMotion === true ? "true" : "false";
  const detected =
    (navigator.languages || [navigator.language]).find((tag) =>
      /^(ru|en)(-|$)/i.test(tag),
    ) || "en";
  const validLanguage = (value) => value === "ru" || value === "en";
  const language = validLanguage(settings.language)
    ? settings.language
    : validLanguage(storedLanguage)
      ? storedLanguage
      : detected.split("-")[0].toLowerCase();
  const ru = language === "ru";
  root.lang = ru ? "ru" : "en";
  document.querySelector("[data-boot-label]").textContent = ru
    ? "Подготовка приложения"
    : "Preparing the app";
  document.querySelector("[data-boot-description]").textContent = ru
    ? "Сейчас появится библиотека и настройки ваших игр."
    : "Your game library and settings will appear shortly.";
  document.querySelector("[data-boot-retry]").textContent = ru
    ? "Перезапустить интерфейс"
    : "Reload the interface";
  const stalled = setTimeout(() => {
    const label = document.querySelector("[data-boot-label]");
    if (!label) return;
    label.textContent = ru
      ? "Запуск занимает больше времени"
      : "Startup is taking longer";
    document.querySelector("[data-boot-retry]").hidden = false;
  }, 8000);
  window.addEventListener("gsm:mounted", () => clearTimeout(stalled), {
    once: true,
  });
})();
