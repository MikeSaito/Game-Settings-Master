import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import "@/styles/startup.css";

export function StartupScreen({
  title,
  description,
  stage = "library",
  progress,
  busy = true,
  embedded = false,
  children,
}: {
  title: string;
  description?: string;
  stage?: "updates" | "library";
  progress?: number | null;
  busy?: boolean;
  embedded?: boolean;
  children?: ReactNode;
}) {
  const { t } = useTranslation("startup");
  return (
    <div
      className={`startup-screen${embedded ? " startup-screen--embedded" : ""}`}
      data-testid="startup-screen"
      data-stage={stage}
    >
      <div className="startup-card">
        <img
          className="startup-logo"
          src="/logo.svg"
          width="72"
          height="72"
          alt=""
        />
        <h1 className="startup-title">Game Settings Master</h1>
        <div role="status" aria-live="polite">
          <p className="startup-label">{title}</p>
          <p className="startup-description">
            {description ??
              t(stage === "updates" ? "updatesHint" : "libraryHint")}
          </p>
        </div>
        {busy && (
          <div
            className={`startup-progress${progress == null ? " startup-progress--indeterminate" : ""}`}
            role="progressbar"
            aria-label={title}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={progress ?? undefined}
          >
            <span
              style={progress == null ? undefined : { width: `${progress}%` }}
            />
          </div>
        )}
        <ol className="startup-steps" aria-label={t("stages")}>
          <li aria-current={stage === "updates" ? "step" : undefined}>
            {t("updates")}
          </li>
          <li aria-current={stage === "library" ? "step" : undefined}>
            {t("library")}
          </li>
        </ol>
        {children && <div className="startup-actions">{children}</div>}
      </div>
    </div>
  );
}
