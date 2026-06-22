import { AlertTriangle, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { EditorApplyBar } from "../components/advanced/EditorApplyBar";
import { EditorModeBar } from "../components/advanced/EditorModeBar";
import { EditorSidebar } from "../components/advanced/EditorSidebar";
import { ParameterList } from "../components/advanced/ParameterList";
import { SavedPresetsPanel } from "../components/advanced/SavedPresetsPanel";
import { Alert, EmptyState } from "../components/ds/Feedback";
import { Badge } from "../components/ds/Badge";
import { useAdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { gpuFilterHint } from "@/lib/gpu";
import type { GameProfile } from "@/lib/core";
import { Backups } from "./Backups";

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
        {t("noConfig.default")}
      </Alert>
    );
  }

  const gpuHint = state.gpu ? gpuFilterHint(state.gpu) : null;
  const pageTitle =
    state.panel === "basic"
      ? t("mode.basicTitle")
      : state.panel === "advanced"
        ? t("mode.advancedTitle")
        : t("mode.backupsTitle");

  return (
    <div>
      <EditorModeBar
        gameId={game.id}
        panel={state.panel}
        onPanelChange={state.setPanel}
        engineStats={state.engineStats}
      />

      {state.panel === "backups" ? (
        <Backups game={game} embedded />
      ) : (
        <div className="flex gap-4">
          <EditorSidebar
            search={state.search}
            onSearchChange={state.setSearch}
            categories={state.categories}
            activeCategory={state.activeCategory}
            onCategoryChange={state.setActiveCategory}
            filterMode={state.filterMode}
            onFilterModeChange={state.setFilterMode}
          />

          <section className="min-w-0 flex-1">
            <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
              <div>
                <h2 className="text-lg font-semibold text-[var(--color-text)]">{pageTitle}</h2>
                <div className="mt-1 flex flex-wrap gap-1.5">
                  <Badge tone="info">
                    {game.engine_version
                      ? t("paramsForEngine", {
                          count: state.catalogStats.total,
                          version: game.engine_version,
                        })
                      : t("paramsCount", { count: state.catalogStats.total })}
                  </Badge>
                  <Badge tone="success">{t("knownCount", { count: state.catalogStats.known })}</Badge>
                  {state.catalogStats.unknown > 0 && (
                    <Badge tone="warning">{t("unknownCount", { count: state.catalogStats.unknown })}</Badge>
                  )}
                  {state.limits && state.panel === "basic" && (
                    <Badge tone="accent">
                      {t("scalabilityLimits", { max: state.limits.global_max })}
                    </Badge>
                  )}
                  {state.panel === "advanced" && state.engineStats.total > 0 && (
                    <Badge tone="warning">
                      {t("engineIni.short", {
                        on: state.engineStats.on,
                        total: state.engineStats.total,
                      })}
                    </Badge>
                  )}
                </div>
              </div>
              {gpuHint && (
                <Badge tone="info" className="max-w-xl" title={gpuHint}>
                  {t("gpuHintTitle")}: {gpuHint}
                </Badge>
              )}
            </div>

            {state.conflictCount > 0 && (
              <Alert tone="warning" icon={AlertTriangle} className="mb-3" title={t("conflict.bannerTitle")}>
                {t("conflict.bannerBody", { count: state.conflictCount })}
              </Alert>
            )}

            {state.gameRunning && (
              <Alert tone="warning" icon={AlertTriangle} className="mb-3" title={t("gameRunningTitle")}>
                {t("gameRunningInline")}
              </Alert>
            )}

            {state.message && (
              <Alert tone="success" className="mb-3">
                {state.message}
              </Alert>
            )}
            {state.applyError && (
              <Alert tone="danger" className="mb-3" title={t("errorTitle")}>
                {state.applyError}
              </Alert>
            )}

            <ParameterList
              filteredParams={state.filteredParams}
              search={state.search}
              parametersLoading={state.parametersLoading}
              gpu={state.gpu}
              engineEnabled={state.engineEnabled}
              showEngineToggle={state.panel === "advanced"}
              pendingConflictKeys={state.pendingConflictKeys}
              onUpdateParam={state.updateParam}
              onToggleEngineParam={state.toggleEngineParam}
            />

            <EditorApplyBar state={state} />
            <SavedPresetsPanel state={state} />
          </section>
        </div>
      )}
    </div>
  );
}
