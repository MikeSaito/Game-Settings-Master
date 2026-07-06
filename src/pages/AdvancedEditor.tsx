import { AlertTriangle, SlidersHorizontal } from "lucide-react";
import { useTranslation } from "react-i18next";
import { EditorApplyBar } from "@/components/advanced/EditorApplyBar";
import { ApplyValidationPanel } from "@/components/advanced/ApplyValidationPanel";
import { EditorModeBar } from "@/components/advanced/EditorModeBar";
import { EditorSidebar } from "@/components/advanced/EditorSidebar";
import { ExtraIniPanel } from "@/components/advanced/ExtraIniPanel";
import { ParameterList } from "@/components/advanced/ParameterList";
import { SavedPresetsPanel } from "@/components/advanced/SavedPresetsPanel";
import { SgEngineConflictPanel } from "@/components/advanced/SgEngineConflictPanel";
import { Alert, EmptyState } from "@/components/ds/Feedback";
import { Badge } from "@/components/ds/Badge";
import { useAdvancedEditorState } from "@/hooks/editor/useAdvancedEditorState";
import { gpuFilterHint } from "@/lib/gpu";
import type { GameProfile } from "@/lib/core";
import { BackupsPanel } from "@/components/backups";
import { ConfigPathHelp } from "@/components/library/ConfigPathHelp";

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
      <div className="space-y-3">
        <Alert tone="warning" title={t("noConfig.title")}>
          {t("noConfig.default")}
        </Alert>
        <ConfigPathHelp />
      </div>
    );
  }

  const gpuHint = state.gpu ? gpuFilterHint(state.gpu) : null;

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="shrink-0">
        <EditorModeBar
          gameId={game.id}
          panel={state.panel}
          onPanelChange={state.setPanel}
          engineStats={state.engineStats}
          showExtraTab={state.extraIniAvailable}
        />
      </div>

      {state.panel === "backups" ? (
        <div className="min-h-0 flex-1 overflow-y-auto">
          <BackupsPanel game={game} />
        </div>
      ) : state.panel === "extra" ? (
        <div className="min-h-0 flex-1 overflow-y-auto">
          <ExtraIniPanel gameConfig={state.gameConfig} loading={state.gameConfigLoading} />
        </div>
      ) : state.panel === "presets" ? (
        <div className="min-h-0 flex-1 space-y-3 overflow-y-auto">
          {state.gameRunning && (
            <Alert tone="warning" icon={AlertTriangle} title={t("gameRunningTitle")}>
              {t("gameRunningInline")}
            </Alert>
          )}
          {state.message && (
            <Alert tone="success">{state.message}</Alert>
          )}
          {state.applyError && (
            <Alert tone="danger" title={t("errorTitle")}>
              {state.applyError}
            </Alert>
          )}
          <SavedPresetsPanel state={state} variant="page" />
        </div>
      ) : (
        <div className="flex min-h-0 flex-1 gap-4">
          <EditorSidebar
            search={state.search}
            onSearchChange={state.setSearch}
            categories={state.categories}
            activeCategory={state.activeCategory}
            onCategoryChange={state.setActiveCategory}
            filterMode={state.filterMode}
            onFilterModeChange={state.setFilterMode}
          />

          <section className="flex min-h-0 min-w-0 flex-1 flex-col">
            <div className="mb-3 shrink-0 flex flex-wrap items-center justify-between gap-2">
              <div className="flex flex-wrap gap-1.5">
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
                  {state.panel === "basic" && state.gusIniStats.total > 0 && (
                    <Badge tone="warning">
                      {t("gusIni.short", {
                        on: state.gusIniStats.on,
                        total: state.gusIniStats.total,
                      })}
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
              {gpuHint && (
                <Badge tone="info" className="max-w-xl" title={gpuHint}>
                  {t("gpuHintTitle")}: {gpuHint}
                </Badge>
              )}
            </div>

            {state.validationIssues.length > 0 && (
              <ApplyValidationPanel
                issues={state.validationIssues}
                warningsAcknowledged={state.applyWarningsAcknowledged}
                onWarningsAcknowledgedChange={state.setApplyWarningsAcknowledged}
              />
            )}

            {state.conflictGroups.length > 0 && (
              <SgEngineConflictPanel
                groups={state.conflictGroups}
                onResolve={state.resolveSgConflict}
              />
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
              className="min-h-0 flex-1"
              filteredParams={state.filteredParams}
              search={state.search}
              parametersLoading={state.parametersLoading}
              gpu={state.gpu}
              engineEnabled={state.engineEnabled}
              showEngineToggle
              gusIniToggleOnly={state.panel === "basic"}
              shippedIniKeys={state.shippedIniKeys}
              pendingConflictKeys={state.pendingConflictKeys}
              comboWarningsByKey={state.comboWarningsByKey}
              onUpdateParam={state.updateParam}
              onToggleEngineParam={state.toggleEngineParam}
            />

            <div className="mt-3 shrink-0">
              <EditorApplyBar state={state} />
            </div>
          </section>
        </div>
      )}
    </div>
  );
}
