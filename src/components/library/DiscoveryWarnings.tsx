import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { usePollingEnabled } from "@/hooks/app/useBackgroundSafeEnabled";
export function DiscoveryWarnings() {
  const { t } = useTranslation("improvements");
  const polling = usePollingEnabled();
  const { data: warnings = [] } = useQuery({
    queryKey: ["discovery-warnings"],
    queryFn: () => invoke<string[]>("get_discovery_warnings"),
    enabled: polling,
    refetchInterval: polling ? 30_000 : false,
  });
  if (!warnings?.length) return null;
  return (
    <div
      role="status"
      className="mb-3 rounded border border-[var(--color-warning)] p-3 text-sm"
    >
      <p>{t("discovery.warnings")}</p>
      {warnings.map((warning) => (
        <p key={warning}>{warning}</p>
      ))}
    </div>
  );
}
