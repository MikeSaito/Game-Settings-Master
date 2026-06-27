import { useTranslation } from "react-i18next";
import { Alert } from "@/components/ds/Feedback";

interface Props {
  className?: string;
}

export function ConfigPathHelp({ className }: Props) {
  const { t } = useTranslation("library");
  const examples = t("configHelp.examples", { returnObjects: true }) as string[];

  return (
    <Alert tone="warning" title={t("configHelp.title")} className={className}>
      <div className="space-y-2">
        <p>{t("configHelp.body")}</p>
        <ul className="list-disc space-y-1 pl-4">
          <li>{t("configHelp.pickFolder")}</li>
          <li>{t("configHelp.gameLaunchHint")}</li>
        </ul>
        <div>
          <div className="mb-1 font-semibold text-[var(--color-text)]">
            {t("configHelp.examplesTitle")}
          </div>
          <ul className="space-y-1 font-mono text-xs text-[var(--color-text-faint)]">
            {examples.map((example) => (
              <li key={example}>{example}</li>
            ))}
          </ul>
        </div>
      </div>
    </Alert>
  );
}
