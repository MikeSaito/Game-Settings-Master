import { useQueryClient } from "@tanstack/react-query";
import { useTranslation } from "react-i18next";
import { setBackendLanguage } from "../../lib/api";
import { type AppLanguage, SUPPORTED_LANGUAGES } from "../../i18n";
import { cn } from "../../lib/cn";

export function LanguageToggle() {
  const { i18n } = useTranslation();
  const queryClient = useQueryClient();
  const active: AppLanguage = i18n.resolvedLanguage === "en" ? "en" : "ru";

  const change = (lang: AppLanguage) => {
    if (lang === active) return;
    void i18n.changeLanguage(lang);
    void setBackendLanguage(lang).catch(() => {});
    // Parameter catalog titles/descriptions are localized on the backend.
    void queryClient.invalidateQueries({ queryKey: ["parameters"] });
  };

  return (
    <div className="flex items-center gap-1 rounded-[var(--radius-control)] border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-0.5">
      {SUPPORTED_LANGUAGES.map((lang) => (
        <button
          key={lang}
          type="button"
          onClick={() => change(lang)}
          className={cn(
            "flex-1 rounded-[7px] px-2 py-1 text-xs font-semibold uppercase transition",
            lang === active
              ? "bg-[var(--color-surface-raised)] text-[var(--color-text)]"
              : "text-[var(--color-text-muted)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text-secondary)]",
          )}
        >
          {lang}
        </button>
      ))}
    </div>
  );
}
