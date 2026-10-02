import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
export function DiscoveryWarnings() {
  const { t } = useTranslation("improvements");
  const { data: warnings = [] } = useQuery({
    queryKey: ["discovery-warnings"],
    queryFn: () => invoke<string[]>("get_discovery_warnings"),
    refetchInterval: 30_000,
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
