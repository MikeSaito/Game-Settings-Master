import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { formatInvokeError, type GameProfile } from "@/lib/core";
import type { DiagnosticReport } from "@/lib/api/bindings";
import { isTauriRuntime } from "@/lib/api";
import { previewAndApply } from "@/lib/api/preparedChanges";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";

export function DiagnosticsPanel({ game }: { game: GameProfile }) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const { data: report, error: queryError } = useQuery({
    queryKey: ["diagnostic", game.id],
    queryFn: () =>
      invoke<DiagnosticReport | null>("get_diagnostic_report", {
        gameId: game.id,
        refresh: false,
      }),
  });
  useEffect(() => {
    if (!isTauriRuntime()) return;
    const subscription = listen<DiagnosticReport | null>(
      "diagnostic-updated",
      (event) => {
        if (event.payload?.game_id === game.id)
          client.setQueryData(["diagnostic", game.id], event.payload);
      },
    );
    void subscription.catch((error) => setError(formatInvokeError(error)));
    return () => {
      void subscription.then((unlisten) => unlisten()).catch(() => {});
    };
  }, [game.id, client]);
  const refresh = () =>
    invoke<DiagnosticReport | null>("get_diagnostic_report", {
      gameId: game.id,
      refresh: true,
    }).then((report) => client.setQueryData(["diagnostic", game.id], report));
  const reapply = async () => {
    if (!report || !game.config_dir) return;
    const changes: {
      files: Record<string, Record<string, Record<string, string>>>;
      removals: Record<string, Record<string, string[]>>;
    } = { files: {}, removals: {} };
    const inputUpdates: { line: number; expected: string; value: string }[] =
      [];
    for (const op of report.changes) {
      if (op.kind.startsWith("input:")) {
        if (op.before != null && op.after != null)
          inputUpdates.push({
            line: Number(op.kind.slice(6)),
            expected: op.after,
            value: op.before,
          });
        continue;
      }
      if (op.kind !== "scalar" && !op.kind.startsWith("scalar:")) continue;
      if (op.before === null)
        ((changes.removals[op.file] ??= {})[op.section] ??= []).push(op.key);
      else
        ((changes.files[op.file] ??= {})[op.section] ??= {})[op.key] =
          op.before;
    }
    await previewAndApply({
      game_id: game.id,
      config_dir: game.config_dir,
      changes,
      input_updates: inputUpdates,
    });
    await refresh();
    invalidateGameWorkspace(client, game.config_dir, game.id);
  };
  const run = async (fn: () => Promise<unknown>) => {
    setBusy(true);
    setError(undefined);
    try {
      await fn();
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <details
      open={!!report && report.status !== "unchanged"}
      className="mb-3 rounded border border-[var(--color-border)] p-3 text-sm"
    >
      <summary>
        {t("diagnostics.title")}
        {report && ` · ${t(`diagnostics.${report.status}`)}`}
      </summary>
      <p className="text-[var(--color-text-muted)]">{t("diagnostics.scope")}</p>
      {report && (
        <>
          <p className="break-all">{report.config_dir}</p>
          {report.status === "moved" && (
            <p>
              {t("diagnostics.previous")}: {report.expected_dir}
            </p>
          )}
          {report.message && <p>{report.message}</p>}
          {report.changes.map((op) => (
            <p key={op.id} className="break-all font-mono">
              {op.file} · {op.key}: {op.before ?? t("preview.absent")} →{" "}
              {op.after ?? t("preview.absent")}
            </p>
          ))}
        </>
      )}
      <div className="mt-2 flex gap-2">
        <Button
          variant="ghost"
          size="sm"
          disabled={busy}
          onClick={() => void run(refresh)}
        >
          {t("diagnostics.check")}
        </Button>
        {report?.changes.some(
          (op) =>
            op.kind === "scalar" ||
            op.kind.startsWith("scalar:") ||
            op.kind.startsWith("input:"),
        ) && (
          <Button size="sm" disabled={busy} onClick={() => void run(reapply)}>
            {t("diagnostics.reapply")}
          </Button>
        )}
      </div>
      {(error || queryError) && (
        <p role="alert">{error ?? formatInvokeError(queryError)}</p>
      )}
    </details>
  );
}
