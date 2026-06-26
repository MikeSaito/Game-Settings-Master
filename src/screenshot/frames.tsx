import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";
import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { AppLanguage } from "@/i18n";
import { AppShell } from "@/components/layout/AppShell";
import { EditorModeBar } from "@/components/advanced/EditorModeBar";
import { EditorSidebar } from "@/components/advanced/EditorSidebar";
import { ParameterList } from "@/components/advanced/ParameterList";
import { BackupRow } from "@/components/backups/BackupRow";
import { BackupSectionTitle } from "@/components/backups/BackupSectionTitle";
import { GameGridCard } from "@/components/library/GameGridCard";
import { LibraryToolbar } from "@/components/library/LibraryToolbar";
import { Badge } from "@/components/ds/Badge";
import { Alert } from "@/components/ds/Feedback";
import { Button } from "@/components/ds/Button";
import { AppWindowFocusProvider } from "@/context/AppWindowFocusProvider";
import type { EditorPanel } from "@/lib/routing";
import type { GameParameter, GameProfile } from "@/lib/core";
import {
  advancedParameters,
  basicParameters,
  getScreenshotGame,
  getScreenshotGames,
  screenshotBackups,
  screenshotCategoriesAdvanced,
  screenshotCategoriesBasic,
  screenshotGpu,
} from "./fixtures";

const CATALOG_TOTAL = 725;

function ShotProviders({ children, route = "/library" }: { children: ReactNode; route?: string }) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  client.setQueryData(["gpu"], screenshotGpu);
  return (
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={[route]}>
        <AppWindowFocusProvider>{children}</AppWindowFocusProvider>
      </MemoryRouter>
    </QueryClientProvider>
  );
}

function ShotFrame({ id, children }: { id: string; children: ReactNode }) {
  return (
    <div id={id} className="shot-frame">
      <div className="shot-frame__inner">{children}</div>
    </div>
  );
}

function LibraryFrame({ games }: { games: GameProfile[] }) {
  const { t } = useTranslation("library");
  const scanSummary = {
    total: games.length,
    withConfig: games.filter((g) => g.config_dir).length,
    ue: games.filter((g) => g.is_ue).length,
    withCover: games.filter((g) => g.cover_url || g.custom_cover).length,
  };

  return (
    <ShotFrame id="shot-library">
      <ShotProviders route="/library">
        <AppShell selectedGame={null}>
          <div className="space-y-4">
            <div className="flex flex-wrap items-end justify-between gap-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.22em] text-[var(--color-accent)]">
                  Game Settings Master
                </p>
                <h1 className="mt-1 text-2xl font-semibold tracking-tight text-[var(--color-text)]">
                  {t("header.title")}
                </h1>
                <p className="mt-1 text-sm text-[var(--color-text-muted)]">{t("header.subtitle")}</p>
              </div>
              <div className="flex flex-wrap gap-1.5">
                <Badge tone="info">{t("badges.total", { count: scanSummary.total })}</Badge>
                <Badge tone="success">{t("badges.withConfig", { count: scanSummary.withConfig })}</Badge>
                <Badge tone="accent">{t("badges.ue", { count: scanSummary.ue })}</Badge>
                <Badge tone="neutral">{t("badges.withCover", { count: scanSummary.withCover })}</Badge>
              </div>
            </div>
            <LibraryToolbar
              query=""
              onQueryChange={() => {}}
              viewMode="grid"
              onViewModeChange={() => {}}
              scanning={false}
              onScan={() => {}}
              onAdd={() => {}}
              adding={false}
            />
            <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              {games.map((game, index) => (
                <GameGridCard
                  key={game.id}
                  game={game}
                  selected={index === 0}
                  mode="grid"
                  onSelect={() => {}}
                  onPickConfig={() => {}}
                  pickingConfig={false}
                  onImportCover={() => {}}
                  importingCover={false}
                  onRemoveCover={() => {}}
                  removingCover={false}
                  onRemoveGame={() => {}}
                />
              ))}
            </div>
          </div>
        </AppShell>
      </ShotProviders>
    </ShotFrame>
  );
}

function EditorApplyBarStatic({ panel }: { panel: EditorPanel }) {
  const { t } = useTranslation("advanced");
  const applyLabel = panel === "basic" ? t("applyBasic") : t("applyAdvanced");

  return (
    <div className="sticky bottom-3 z-10 mt-3 rounded-[var(--radius-panel)] border border-[var(--color-border)] bg-[var(--color-surface)] p-2 shadow-[var(--shadow-panel)]">
      <div className="flex flex-wrap items-center gap-2">
        <div className="min-w-[160px] flex-1">
          <div className="text-xs font-semibold text-[var(--color-text)]">
            {t("changesCount", { count: 0 })}
          </div>
        </div>
        <Button variant="primary" disabled>
          {applyLabel}
        </Button>
      </div>
    </div>
  );
}

function EditorFrame({
  id,
  panel,
  parameters,
  game,
}: {
  id: string;
  panel: EditorPanel;
  parameters: GameParameter[];
  game: GameProfile;
}) {
  const { t } = useTranslation("advanced");
  const pageTitle =
    panel === "basic"
      ? t("mode.basicTitle")
      : panel === "advanced"
        ? t("mode.advancedTitle")
        : t("mode.backupsTitle");
  const knownCount = parameters.filter((p) => p.known).length;
  const categories = panel === "basic" ? screenshotCategoriesBasic : screenshotCategoriesAdvanced;
  const activeCategory = panel === "basic" ? "Scalability" : "Rendering";

  return (
    <ShotFrame id={id}>
      <ShotProviders route="/game/game-1/advanced">
        <AppShell selectedGame={game}>
          <div>
            <EditorModeBar
              gameId={game.id}
              panel={panel}
              onPanelChange={() => {}}
              engineStats={{ total: 142, on: 38, off: 104 }}
            />
            <div className="flex gap-4">
              <EditorSidebar
                search=""
                onSearchChange={() => {}}
                categories={categories}
                activeCategory={activeCategory}
                onCategoryChange={() => {}}
                filterMode="recommended"
                onFilterModeChange={() => {}}
              />
              <section className="min-w-0 flex-1">
                <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
                  <div>
                    <h2 className="text-lg font-semibold text-[var(--color-text)]">{pageTitle}</h2>
                    <div className="mt-1 flex flex-wrap gap-1.5">
                      <Badge tone="info">
                        {game.engine_version
                          ? t("paramsForEngine", {
                              count: CATALOG_TOTAL,
                              version: game.engine_version,
                            })
                          : t("paramsCount", { count: CATALOG_TOTAL })}
                      </Badge>
                      <Badge tone="success">{t("knownCount", { count: knownCount })}</Badge>
                      {panel === "basic" && (
                        <Badge tone="accent">{t("scalabilityLimits", { max: 3 })}</Badge>
                      )}
                      {panel === "advanced" && (
                        <Badge tone="warning">
                          {t("engineIni.short", { on: 38, total: 142 })}
                        </Badge>
                      )}
                    </div>
                  </div>
                </div>
                <ParameterList
                  filteredParams={parameters}
                  search=""
                  parametersLoading={false}
                  gpu={screenshotGpu}
                  engineEnabled={new Set(parameters.map((p) => p.key))}
                  showEngineToggle={panel === "advanced"}
                  onUpdateParam={() => {}}
                  onToggleEngineParam={() => {}}
                />
                <EditorApplyBarStatic panel={panel} />
              </section>
            </div>
          </div>
        </AppShell>
      </ShotProviders>
    </ShotFrame>
  );
}

function BackupsFrame({ game }: { game: GameProfile }) {
  const { t } = useTranslation("backups");

  return (
    <ShotFrame id="shot-backups">
      <ShotProviders route="/game/game-1/advanced">
        <AppShell selectedGame={game}>
          <div>
            <EditorModeBar
              gameId={game.id}
              panel="backups"
              onPanelChange={() => {}}
              engineStats={{ total: 142, on: 38, off: 104 }}
            />
            <div className="space-y-6">
              <div className="flex flex-wrap gap-2">
                <Badge tone="neutral">{t("header.backupsCount", { count: screenshotBackups.length })}</Badge>
              </div>
              <Alert tone="info" title={t("howItWorks.title")}>
                {t("howItWorks.body")}
              </Alert>
              <section>
                <BackupSectionTitle title={t("list.title")} description={t("list.desc")} />
                <div className="space-y-2">
                  {screenshotBackups.map((backup) => (
                    <BackupRow
                      key={backup.id}
                      backup={backup}
                      restoring={false}
                      disabled={false}
                      onRestore={() => {}}
                    />
                  ))}
                </div>
              </section>
            </div>
          </div>
        </AppShell>
      </ShotProviders>
    </ShotFrame>
  );
}

export function ScreenshotFrames({ lang }: { lang: AppLanguage }) {
  const games = getScreenshotGames(lang);
  const game = getScreenshotGame(lang);

  return (
    <div className="shot-root">
      <LibraryFrame games={games} />
      <EditorFrame id="shot-editor-basic" panel="basic" parameters={basicParameters} game={game} />
      <EditorFrame id="shot-editor-advanced" panel="advanced" parameters={advancedParameters} game={game} />
      <BackupsFrame game={game} />
    </div>
  );
}
