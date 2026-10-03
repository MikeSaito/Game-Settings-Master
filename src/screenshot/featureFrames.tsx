import { useEffect } from "react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ChangesPreview } from "@/components/app/ChangesPreview";
import { DiagnosticsPanel } from "@/components/advanced/DiagnosticsPanel";
import { SnapshotTools } from "@/components/backups/SnapshotTools";
import { showPrepared } from "@/lib/api/preparedChanges";
import {
  featureConfigDir,
  featurePlan,
  featureReport,
  featureRequest,
} from "./featureFixtures";
import { getScreenshotGame } from "./fixtures";
import type { AppLanguage } from "@/i18n";

function PreviewExample() {
  useEffect(() => {
    void showPrepared(featurePlan, featureRequest).catch(() => {});
  }, []);
  return null;
}

export function FeatureFrames({
  lang,
  feature,
}: {
  lang: AppLanguage;
  feature: string;
}) {
  const game = { ...getScreenshotGame(lang), config_dir: featureConfigDir };
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: Infinity } },
  });
  client.setQueryData(["diagnostic", game.id], featureReport);
  return (
    <QueryClientProvider client={client}>
      <div id="shot-feature" className="shot-feature">
        {feature === "preview" && (
          <>
            <ChangesPreview />
            <PreviewExample />
          </>
        )}
        {feature === "restore" && (
          <SnapshotTools
            game={game}
            disabled={false}
            backups={[
              {
                id: "snapshot-demo",
                created_at: "2026-10-03T09:00:00",
                files: ["Engine.ini"],
                name:
                  lang === "ru"
                    ? "До изменения теней"
                    : "Before changing shadows",
              },
            ]}
          />
        )}
        {feature === "diagnostics" && <DiagnosticsPanel game={game} />}
      </div>
    </QueryClientProvider>
  );
}
