import { AlertTriangle, ChevronDown } from "lucide-react";
import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ReShadeDisclaimerModal } from "../ReShadeDisclaimerModal";
import { ReShadePresetCard } from "./ReShadePresetCard";
import { ReShadeSlider } from "./ReShadeSlider";
import { ReShadeSuccessBar } from "./ReShadeSuccessBar";
import { ReShadeWizardFooter } from "./ReShadeWizardFooter";
import { useReShadePage } from "./useReShadePage";
import { Alert } from "../ui/Alert";
import { Badge } from "../ui/Badge";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { SectionHeader } from "../ui/SectionHeader";
import { Toggle } from "../ui/Toggle";
import { GameRunningAlert } from "../../hooks/useGameRunning";
import {
  apiLabel,
  apiProxyFile,
  engineApiHint,
  formatReShadeStatusMeta,
  ReShadeEffectDefaultParams,
  ReShadeFineTuneEffects,
  ReShadeSliderParams,
  reshadeEffectHint,
  reshadeEffectLabel,
  reshadeSliderLabel,
} from "../../lib/reshade";
import { cn } from "../../lib/cn";
import type { GameProfile } from "../../lib/types";
import { PageHeader } from "../ui/PageHeader";

function engineBadge(family: string | undefined) {
  if (family === "forza") return <Badge tone="accent">Forza Horizon 6</Badge>;
  if (family === "unity") return <Badge tone="accent">Unity</Badge>;
  if (family === "ue4") return <Badge tone="accent">Unreal Engine 4</Badge>;
  if (family === "ue5") return <Badge tone="accent">Unreal Engine 5</Badge>;
  return null;
}

export function ReShadeWizard({ game }: { game: GameProfile }) {
  const { t } = useTranslation("reshade");
  const p = useReShadePage(game);
  const effectiveApi = p.effectiveApi;
  const [fineTuneOpen, setFineTuneOpen] = useState(false);

  useEffect(() => {
    setFineTuneOpen(false);
  }, [game.id]);

  const paramValue = (effect: string, key: string, fallback: string) =>
    p.effectiveOverrides.parameters?.[effect]?.[key] ??
    p.presetDetails?.parameters.find((x) => x.effect === effect && x.key === key)?.value ??
    ReShadeEffectDefaultParams[effect]?.[key] ??
    fallback;

  const presetTechniques = p.presetDetails?.techniques ?? [];

  const enabledEffects = p.effectiveOverrides.techniques ?? presetTechniques;

  return (
    <div className="space-y-8">
      <PageHeader
        title="ReShade"
        meta={
          <>
            {engineBadge(game.engine_family)}
            <span className="text-sm text-muted">
              {formatReShadeStatusMeta({
                globallyOn: p.globallyOn,
                activeForGame: p.activeForGame,
                installed: !!p.status?.installed,
                brokenInstall: !!p.status?.broken_install,
                selectedApi: p.selectedApi,
                requestedPresetName: p.requestedPresetName,
                installedPresetName: p.installedPresetName,
                gpuAdapted: p.gpuAdapted,
              })}
            </span>
          </>
        }
      />

      {p.pageAlert && (
        <Alert tone={p.pageAlert.tone} icon={AlertTriangle} title={p.pageAlert.title}>
          <p>{p.pageAlert.message}</p>
          {p.pageAlert.kind === "broken" && p.pageAlert.actionLabel && (
            <Button
              variant="primary"
              className="mt-3"
              loading={p.removeMutation.isPending}
              onClick={p.removeCurrentGameReShade}
            >
              {p.pageAlert.actionLabel}
            </Button>
          )}
        </Alert>
      )}

      <GameRunningAlert
        exeName={p.runningExeName ?? game.exe_name}
        gameName={game.name}
        context="reshade"
      />
      {p.message && <ReShadeSuccessBar message={p.message} />}

      <section>
        <SectionHeader step={1} title={t("section.enable.title")} hint={engineApiHint(game)} />
        <Card className={cn(p.gameBlockDisabled && "opacity-50")}>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div className="min-w-0">
              <p className="text-sm font-medium text-[var(--color-text)]">
                {t("enable.forGame", { name: game.name })}
              </p>
              <p className="text-xs text-muted">{t("enable.postProcess")}</p>
            </div>
            <div className="flex flex-wrap items-center gap-3">
              <label className="flex items-center gap-2 text-sm text-muted">
                <span className="hidden sm:inline">API</span>
                <select
                  className="input-field min-w-[10.5rem] rounded-lg border px-3 py-2 text-sm text-[var(--color-text)]"
                  value={effectiveApi}
                  disabled={
                    p.gameBlockDisabled || !p.activeForGame || p.saveSettingsMutation.isPending
                  }
                  onChange={(e) => p.selectApi(e.target.value)}
                >
                  {p.apis.length === 0 && effectiveApi ? (
                    <option value={effectiveApi} disabled>
                      {t("enable.loadingOption")}
                    </option>
                  ) : null}
                  {p.apis.map((api) => (
                    <option key={api.id} value={api.id}>
                      {api.name}
                      {p.apiHint === api.id ? t("enable.recommendedSuffix") : ""}
                    </option>
                  ))}
                </select>
              </label>
              <Toggle
                checked={p.settings?.per_game[game.id]?.enabled ?? true}
                onChange={p.handlePerGameToggle}
                disabled={p.gameBlockDisabled || p.saveSettingsMutation.isPending}
              />
            </div>
          </div>
          <p className="mt-3 text-xs text-muted">
            {apiProxyFile(effectiveApi)} · {apiLabel(effectiveApi)}
          </p>
          {p.gpuAdaptReason && p.gpuName && (
            <p className="mt-1 text-xs text-muted">
              {t("enable.gpuAdapted", { gpu: p.gpuName, reason: p.gpuAdaptReason })}
            </p>
          )}
          {(!p.globallyOn || !p.activeForGame) && (
            <p className="mt-2 text-xs text-muted">{t("enable.proxyRemovedNote")}</p>
          )}
          <details className="mt-4 text-sm">
            <summary className="cursor-pointer text-muted hover:text-[var(--color-text)]">
              {t("enable.globalSettingsSummary")}
            </summary>
            <div className="mt-3 space-y-3 border-t border-[var(--color-border)] pt-3">
              <div className="flex items-center justify-between gap-4">
                <span className="text-sm">{t("enable.globalToggleLabel")}</span>
                <Toggle
                  checked={p.globallyOn}
                  onChange={p.handleGlobalToggle}
                  disabled={p.settingsQuery.isLoading}
                />
              </div>
              <p className="text-xs text-muted">
                {t("enable.defaultPreset", {
                  preset: p.settings?.default_preset ?? "clarity",
                })}
              </p>
            </div>
          </details>
        </Card>
      </section>

      <section className={cn(p.gameBlockDisabled && "opacity-50 pointer-events-none")}>
        <SectionHeader
          step={2}
          title={t("section.preset.title")}
          hint={t("section.preset.hint")}
        />
        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
          {p.presets.map((preset) => (
            <ReShadePresetCard
              key={preset.id}
              preset={preset}
              selected={p.requestedPreset === preset.id}
              installed={p.installedPresetId === preset.id}
              appliesWhenAdapted={
                p.gpuAdapted &&
                p.effectivePresetId === preset.id &&
                p.requestedPreset !== preset.id
              }
              disabled={!p.activeForGame}
              onSelect={() => p.handlePresetSelect(preset.id)}
            />
          ))}
        </div>
      </section>

      {p.activeForGame && p.globallyOn && (
        <section>
          <SectionHeader
            step={3}
            title={t("section.fineTune.title")}
            hint={t("section.fineTune.hint")}
          />
          <Card padding="sm" className="overflow-hidden !p-0">
            <button
              type="button"
              aria-expanded={fineTuneOpen}
              className="flex w-full items-center justify-between gap-3 px-4 py-3.5 text-left transition hover:bg-[var(--color-bg-hover)]"
              onClick={() => setFineTuneOpen((open) => !open)}
            >
              <span className="text-sm text-[var(--color-text)]">
                {fineTuneOpen ? t("fineTune.hide") : t("fineTune.show")}
              </span>
              <ChevronDown
                size={18}
                className={cn(
                  "shrink-0 text-muted transition-transform duration-200",
                  fineTuneOpen && "rotate-180",
                )}
                aria-hidden
              />
            </button>
            {fineTuneOpen && (
              <div className="space-y-5 border-t border-[var(--color-border)] px-4 py-4">
                <div>
                  <p className="mb-2 text-xs font-medium uppercase tracking-wide text-muted">
                    {t("fineTune.effects")}
                  </p>
                  {p.presetDetails ? (
                    <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                      {ReShadeFineTuneEffects.map((effect) => {
                        const on = enabledEffects.includes(effect);
                        const label = reshadeEffectLabel(effect);
                        const hint = reshadeEffectHint(effect);
                        const inPreset = presetTechniques.includes(effect);
                        return (
                          <button
                            key={effect}
                            type="button"
                            title={hint}
                            onClick={() => {
                              const next = on
                                ? enabledEffects.filter((x) => x !== effect)
                                : [...enabledEffects, effect];
                              const techniques = next.length ? next : presetTechniques;
                              const patch: {
                                techniques: string[];
                                parameters?: Record<string, Record<string, string>>;
                              } = { techniques };

                              if (
                                !on &&
                                ReShadeEffectDefaultParams[effect] &&
                                !p.effectiveOverrides.parameters?.[effect]
                              ) {
                                patch.parameters = {
                                  ...(p.effectiveOverrides.parameters ?? {}),
                                  [effect]: ReShadeEffectDefaultParams[effect],
                                };
                              }

                              p.patchOverrides(patch);
                            }}
                            className={cn(
                              "rounded-lg border px-3 py-2 text-left transition",
                              on
                                ? "border-[var(--color-accent)] bg-[var(--color-accent-soft)]"
                                : "border-[var(--color-border)] hover:border-[var(--color-border-strong)]",
                            )}
                          >
                            <span
                              className={cn(
                                "block text-sm font-medium",
                                on ? "text-[var(--color-text)]" : "text-body",
                              )}
                            >
                              {label}
                            </span>
                            {hint ? (
                              <span className="mt-0.5 block text-xs text-muted">
                                {hint}
                                {!inPreset && !on ? t("fineTune.notInPresetSuffix") : ""}
                              </span>
                            ) : null}
                          </button>
                        );
                      })}
                    </div>
                  ) : (
                    <p className="text-sm text-muted">{t("fineTune.loadingEffects")}</p>
                  )}
                </div>
                <div className="grid gap-4 sm:grid-cols-2">
                  {ReShadeSliderParams.filter((spec) => enabledEffects.includes(spec.effect)).map(
                    (spec) => {
                    const num = Number.parseFloat(
                      paramValue(spec.effect, spec.key, String(spec.min)),
                    );
                    if (Number.isNaN(num)) return null;
                    return (
                      <ReShadeSlider
                        key={`${spec.effect}-${spec.key}`}
                        label={reshadeSliderLabel(spec.effect, spec.key)}
                        value={num}
                        min={spec.min}
                        max={spec.max}
                        step={spec.step}
                        disabled={
                          p.gameRunning ||
                          p.updateOverridesMutation.isPending ||
                          p.saveSettingsMutation.isPending
                        }
                        onChange={(value) => {
                          const params = {
                            ...(p.effectiveOverrides.parameters ?? {}),
                            [spec.effect]: {
                              ...(p.effectiveOverrides.parameters?.[spec.effect] ?? {}),
                              [spec.key]: String(value),
                            },
                          };
                          p.patchOverrides({ parameters: params });
                        }}
                      />
                    );
                  })}
                </div>
                <div className="flex items-center justify-between gap-4">
                  <div>
                    <p className="text-sm font-medium">{t("fineTune.gpuSaving.title")}</p>
                    <p className="text-xs text-muted">{t("fineTune.gpuSaving.desc")}</p>
                  </div>
                  <Toggle
                    checked={p.effectiveOverrides.behavior?.performance_mode ?? false}
                    disabled={
                      p.gameRunning ||
                      p.updateOverridesMutation.isPending ||
                      p.saveSettingsMutation.isPending
                    }
                    onChange={(checked) =>
                      p.patchOverrides({
                        behavior: {
                          ...(p.effectiveOverrides.behavior ?? {}),
                          performance_mode: checked,
                        },
                      })
                    }
                  />
                </div>
              </div>
            )}
          </Card>
        </section>
      )}

      <ReShadeWizardFooter page={p} />

      <details className="text-sm">
        <summary className="cursor-pointer text-muted hover:text-[var(--color-text)]">
          {t("licenses.summary")}
        </summary>
        <div className="mt-3 space-y-3 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4 text-sm text-body">
          <p>
            <strong>ReShade addon</strong> — © 2014 Patrick Mours (crosire),{" "}
            <a
              href="https://reshade.me"
              className="text-accent underline-offset-2 hover:underline"
              target="_blank"
              rel="noreferrer"
            >
              reshade.me
            </a>
            {t("licenses.addonSuffix")}
          </p>
          <p className="text-xs text-muted">
            {t("licenses.fullTextPath")}{" "}
            <span className="font-mono">presets/reshade/LICENSE-ReShade.txt</span>
            {t("licenses.shadersLabel")}{" "}
            <span className="font-mono">presets/reshade/shaders/THIRD-PARTY-NOTICES.txt</span>
            .
          </p>
          <p className="text-xs text-muted">
            {t("licenses.presetEffects")}{" "}
            <span className="font-mono">presets/reshade/shaders/Shaders/*.fx</span>.
          </p>
        </div>
      </details>

      <details className="text-sm">
        <summary className="cursor-pointer text-muted hover:text-[var(--color-text)]">
          {t("tech.summary")}
        </summary>
        <div className="mt-3 space-y-2 rounded-xl border border-[var(--color-border)] bg-[var(--color-bg)] p-4 font-mono text-xs text-body">
          <p>target_dir: {p.status?.target_dir ?? game.install_dir}</p>
          {p.status?.exe_path && <p>exe: {p.status.exe_path}</p>}
          {p.status?.installed_files?.length ? (
            <p>files: {p.status.installed_files.join(", ")}</p>
          ) : null}
          <p>
            {t("tech.dllInApp")} {p.bundleBinValid ? t("tech.yes") : t("tech.no")}
          </p>
          <p>
            {t("tech.shadersInApp")}{" "}
            {p.status?.shaders_in_bundle ? t("tech.yes") : t("tech.no")}
          </p>
          <p>
            {t("tech.shadersInGame")}{" "}
            {p.status?.shaders_present ? t("tech.yes") : t("tech.no")}
          </p>
          {!p.bundleBinValid && (
            <p className="font-sans text-muted">{t("tech.devHint")}</p>
          )}
        </div>
      </details>

      <ReShadeDisclaimerModal
        kind={p.disclaimer ?? "enable"}
        open={p.disclaimer === "enable" || p.disclaimer === "install"}
        loading={p.saveSettingsMutation.isPending || p.installMutation.isPending}
        onConfirm={p.confirmDisclaimer}
        onCancel={() => p.setDisclaimer(null)}
      />
    </div>
  );
}
