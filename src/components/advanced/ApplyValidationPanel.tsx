import { AlertTriangle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Alert } from "@/components/ds/Feedback";
import {
  formatValidationIssue,
  hasBlockingErrors,
  type ValidationIssue,
} from "@/lib/editor/validation";

interface Props {
  issues: ValidationIssue[];
  warningsAcknowledged: boolean;
  onWarningsAcknowledgedChange: (value: boolean) => void;
}

export function ApplyValidationPanel({
  issues,
  warningsAcknowledged,
  onWarningsAcknowledgedChange,
}: Props) {
  const { t } = useTranslation("advanced");
  if (issues.length === 0) return null;

  const errors = issues.filter((i) => i.severity === "error");
  const warnings = issues.filter((i) => i.severity === "warning");

  return (
    <div className="mb-3 space-y-2">
      {errors.length > 0 && (
        <Alert tone="danger" icon={AlertTriangle} title={t("validation.errorsTitle")}>
          <ul className="mt-1 list-inside list-disc space-y-0.5 text-sm">
            {errors.map((issue) => (
              <li key={`${issue.code}-${issue.key ?? ""}`}>
                {formatValidationIssue(t, issue)}
              </li>
            ))}
          </ul>
        </Alert>
      )}

      {warnings.length > 0 && (
        <Alert tone="warning" icon={AlertTriangle} title={t("validation.warningsTitle")}>
          <ul className="mt-1 list-inside list-disc space-y-0.5 text-sm">
            {warnings.map((issue) => (
              <li key={`${issue.code}-${issue.key ?? ""}`}>
                {formatValidationIssue(t, issue)}
              </li>
            ))}
          </ul>
          {!hasBlockingErrors(issues) && (
            <label className="mt-2 flex cursor-pointer items-start gap-2 text-sm">
              <input
                type="checkbox"
                className="mt-0.5"
                checked={warningsAcknowledged}
                onChange={(event) => onWarningsAcknowledgedChange(event.target.checked)}
              />
              <span>{t("validation.acknowledgeWarnings")}</span>
            </label>
          )}
        </Alert>
      )}
    </div>
  );
}
