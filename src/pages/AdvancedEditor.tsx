import { SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { BackupBanner } from "../components/BackupBanner";
import { AdvancedEditorFooter } from "../components/advanced/AdvancedEditorFooter";
import { AdvancedEditorToolbar } from "../components/advanced/AdvancedEditorToolbar";
import { ParameterList } from "../components/advanced/ParameterList";
import { Alert } from "../components/ui/Alert";
import { Badge } from "../components/ui/Badge";
import { EmptyState } from "../components/ui/EmptyState";
import { PageHeader } from "../components/ui/PageHeader";
import { GameRunningAlert } from "../hooks/useGameRunning";
import { useAdvancedEditorState } from "../hooks/useAdvancedEditorState";
import { gpuFilterHint } from "../lib/gpuCompat";
import type { GameProfile } from "../lib/types";

interface Props {
  game: GameProfile | null;
}

export function AdvancedEditor({ game }: Props) {
  const { t } = useTranslation("advanced");
  const state = useAdvancedEditorState(game);

  if (!game) {
    return (
      <EmptyState
        icon={SlidersHorizontal}
        title={t("noGame.title")}
        description={t("noGame.desc")}
      />
    );
  }

  if (!state.configDir) {
    return (
      <Alert tone="warning" title={t("noConfig.title")}>
        {state.isUnity ? t("noConfig.unity") : t("noConfig.default")}
      </Alert>
    );
  }

  const gpuHint = state.gpu ? gpuFilterHint(state.gpu) : null;

  return (
    <div className="space-y-6">
      <PageHeader
        title={t("title")}
        meta={
          <>
            <Badge tone="default">
              {t("paramsCount", { count: state.catalogStats.total })}
            </Badge>
            <Badge tone="success">
              {t("knownCount", { count: state.catalogStats.known })}
            </Badge>
            {state.catalogStats.unknown > 0 && (
              <Badge tone="warning">
                {t("unknownCount", { count: state.catalogStats.unknown })}
              </Badge>
            )}
            {state.limits && (
              <Badge tone="info">sg max {state.limits.global_max} · scale 25–200%</Badge>
            )}
          </>
        }
      />

      {gpuHint && (
        <Alert tone="info" title={t("gpuHintTitle")}>
          {gpuHint}
        </Alert>
      )}

      <GameRunningAlert exeName={state.runningExeName} gameName={game.name} />

      {state.message && <BackupBanner message={state.message} />}
      {state.applyError && (
        <Alert tone="error" title={t("errorTitle")}>
          {state.applyError}
        </Alert>
      )}

      <AdvancedEditorToolbar
        search={state.search}
        onSearchChange={state.setSearch}
        categories={state.categories}
        activeCategory={state.activeCategory}
        onCategoryChange={state.setActiveCategory}
        showEngineIniHint={state.showEngineIniHint}
        engineStats={state.engineStats}
      />

      <ParameterList
        filteredParams={state.filteredParams}
        search={state.search}
        parametersLoading={state.parametersLoading}
        gpu={state.gpu}
        engineEnabled={state.engineEnabled}
        onUpdateParam={state.updateParam}
        onToggleEngineParam={state.toggleEngineParam}
      />

      <AdvancedEditorFooter state={state} />
    </div>
  );
}
