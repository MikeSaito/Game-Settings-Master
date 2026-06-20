import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { engineWarningAckKey } from "../../lib/advancedEditorPanels";
import { Alert } from "../ui/Alert";
import { Toggle } from "../ui/Toggle";

interface Props {
  gameId: string;
}

export function EngineExpertWarning({ gameId }: Props) {
  const { t } = useTranslation("advanced");
  const [dismissed, setDismissed] = useState(false);

  const [ack, setAck] = useState(false);

  useEffect(() => {
    try {
      if (sessionStorage.getItem(engineWarningAckKey(gameId)) === "1") {
        setDismissed(true);
      }
    } catch {
      /* ignore */
    }
  }, [gameId]);

  if (dismissed) return null;

  return (
    <Alert tone="warning" title={t("engineWarning.title")}>
      <p className="mb-3 text-sm">{t("engineWarning.body")}</p>
      <p className="mb-3 text-xs text-muted">{t("engineWarning.backupHint")}</p>
      <label className="flex cursor-pointer items-center gap-2 text-sm">
        <Toggle
          checked={ack}
          onChange={(next) => {
            setAck(next);
            if (!next) return;
            try {
              sessionStorage.setItem(engineWarningAckKey(gameId), "1");
            } catch {
              /* ignore */
            }
            setDismissed(true);
          }}
        />
        {t("engineWarning.acknowledge")}
      </label>
    </Alert>
  );
}
