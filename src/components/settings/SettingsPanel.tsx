import { Monitor, Moon, Settings, Sun, X } from "lucide-react";
import { useEffect, useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { useAppSettings } from "@/hooks/app/useAppSettings";
import type { FontScale, ThemeMode } from "@/lib/settings";
import { Button } from "../ds/Button";
import { LanguageSelect } from "./LanguageSelect";
import { SegmentControl } from "../ds/SegmentControl";
import { Switch } from "../ds/Switch";
import { Divider } from "../ds/Panel";

interface Props {
  open: boolean;
  onClose: () => void;
  /** When true, overlay covers only the parent pane (not NavRail / game header). */
  scoped?: boolean;
}

const APP_VERSION = "1.0.2-a";
const FONT_SCALES: Array<{ value: FontScale; labelKey: string }> = [
  { value: 0.875, labelKey: "fontScale.small" },
  { value: 1, labelKey: "fontScale.normal" },
  { value: 1.125, labelKey: "fontScale.large" },
  { value: 1.25, labelKey: "fontScale.xlarge" },
];

export function SettingsPanel({ open, onClose, scoped = false }: Props) {
  const { t } = useTranslation("settings");
  const {
    settings,
    setTheme,
    setFontScale,
    setLanguage,
    setReducedMotion,
    setCompactDensity,
    reset,
  } = useAppSettings();
  const panelRef = useRef<HTMLDivElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) return;
    const previous = document.activeElement as HTMLElement | null;
    closeButtonRef.current?.focus();

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
        return;
      }
      if (event.key !== "Tab") return;
      const focusables = panelRef.current?.querySelectorAll<HTMLElement>(
        'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
      );
      if (!focusables?.length) return;
      const first = focusables[0];
      const last = focusables[focusables.length - 1];
      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    };

    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
      previous?.focus?.();
    };
  }, [onClose, open]);

  if (!open) return null;

  const overlayClass = scoped
    ? "absolute inset-0 z-50 flex"
    : "fixed inset-0 z-50 flex";

  return (
    <div className={overlayClass} role="presentation">
      <aside
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby="settings-title"
        className="h-full w-[min(28rem,calc(100vw-1rem))] overflow-y-auto border-r border-[var(--color-border-strong)] bg-[var(--color-surface)] p-4 shadow-[var(--shadow-panel)]"
      >
        <div className="mb-4 flex items-center justify-between gap-3">
          <div className="flex min-w-0 items-center gap-2">
            <span className="grid h-9 w-9 shrink-0 place-items-center rounded-[var(--radius-panel)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] text-[var(--color-accent-hover)]">
              <Settings size={18} />
            </span>
            <div className="min-w-0">
              <h2 id="settings-title" className="text-base font-semibold text-[var(--color-text)]">
                {t("title")}
              </h2>
              <p className="break-words text-xs text-[var(--color-text-muted)]">{t("subtitle")}</p>
            </div>
          </div>
          <button
            ref={closeButtonRef}
            type="button"
            onClick={onClose}
            className="grid h-8 w-8 place-items-center rounded-[var(--radius-control)] border border-[var(--color-border)] text-[var(--color-text-muted)] hover:border-[var(--color-border-strong)] hover:bg-[var(--color-surface-hover)] hover:text-[var(--color-text)]"
            aria-label={t("close")}
          >
            <X size={16} />
          </button>
        </div>

        <div className="space-y-5">
          <section className="space-y-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-[var(--color-text-muted)]">
              {t("language.title")}
            </h3>
            <LanguageSelect value={settings.language} onChange={setLanguage} />
          </section>

          <section className="space-y-2">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-[var(--color-text-muted)]">
              {t("theme.title")}
            </h3>
            <SegmentControl
              value={settings.theme}
              onChange={(value) => setTheme(value as ThemeMode)}
              ariaLabel={t("theme.aria")}
              className="w-full"
              options={[
                { value: "dark", label: <span className="inline-flex items-center gap-1"><Moon size={13} />{t("theme.dark")}</span> },
                { value: "light", label: <span className="inline-flex items-center gap-1"><Sun size={13} />{t("theme.light")}</span> },
                { value: "system", label: <span className="inline-flex items-center gap-1"><Monitor size={13} />{t("theme.system")}</span> },
              ]}
            />
          </section>

          <section className="space-y-2">
            <div>
              <h3 className="text-xs font-semibold uppercase tracking-wide text-[var(--color-text-muted)]">
                {t("fontScale.title")}
              </h3>
              <p className="text-xs text-[var(--color-text-muted)]">
                {t("fontScale.preview", { value: Math.round(settings.fontScale * 100) })}
              </p>
            </div>
            <div className="grid grid-cols-2 gap-2">
              {FONT_SCALES.map((item) => (
                <button
                  key={item.value}
                  type="button"
                  onClick={() => setFontScale(item.value)}
                  className={
                    settings.fontScale === item.value
                      ? "min-h-11 rounded-[var(--radius-control)] border border-[var(--color-accent)] bg-[var(--color-accent-soft)] px-2 py-2 text-center text-sm font-semibold leading-tight text-[var(--color-text)]"
                      : "min-h-11 rounded-[var(--radius-control)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] px-2 py-2 text-center text-sm leading-tight text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]"
                  }
                >
                  {t(item.labelKey)}
                </button>
              ))}
            </div>
          </section>

          <Divider />

          <section className="space-y-3">
            <h3 className="text-xs font-semibold uppercase tracking-wide text-[var(--color-text-muted)]">
              {t("more.title")}
            </h3>
            <SettingSwitch
              label={t("more.reducedMotion")}
              desc={t("more.reducedMotionDesc")}
              checked={settings.reducedMotion}
              onChange={setReducedMotion}
            />
            <SettingSwitch
              label={t("more.compactDensity")}
              desc={t("more.compactDensityDesc")}
              checked={settings.compactDensity}
              onChange={setCompactDensity}
            />
          </section>

          <Divider />

          <div className="flex items-center justify-between gap-3">
            <Button variant="ghost" onClick={reset}>
              {t("reset")}
            </Button>
            <span className="text-xs text-[var(--color-text-faint)]">
              {t("version", { version: APP_VERSION })}
            </span>
          </div>
        </div>
      </aside>
      <button
        type="button"
        className="flex-1 bg-black/45"
        aria-label={t("close")}
        onClick={onClose}
      />
    </div>
  );
}

function SettingSwitch({
  label,
  desc,
  checked,
  onChange,
}: {
  label: string;
  desc: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}) {
  const labelId = useId();
  const descId = useId();

  return (
    <div className="flex items-start justify-between gap-3 rounded-[var(--radius-control)] border border-[var(--color-border-strong)] bg-[var(--color-bg-soft)] p-3">
      <span className="min-w-0 flex-1">
        <span id={labelId} className="block break-words text-sm font-medium leading-snug text-[var(--color-text)]">{label}</span>
        <span id={descId} className="mt-0.5 block break-words text-xs leading-snug text-[var(--color-text-muted)]">{desc}</span>
      </span>
      <Switch checked={checked} onChange={onChange} aria-labelledby={labelId} aria-describedby={descId} />
    </div>
  );
}
