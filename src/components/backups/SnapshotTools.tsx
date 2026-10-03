import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import type {
  BackupInfo,
  ChangeOperation,
  GameProfile,
  PreparedChanges,
  PrepareRequest,
} from "@/lib/api/bindings";
import { previewAndApply, showPrepared } from "@/lib/api/preparedChanges";
import { formatInvokeError } from "@/lib/core";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";

export function SnapshotTools({
  game,
  backups,
  disabled,
}: {
  game: GameProfile;
  backups: BackupInfo[];
  disabled: boolean;
}) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const [name, setName] = useState("");
  const [first, setFirst] = useState("");
  const [second, setSecond] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  useEffect(() => {
    setName("");
    setFirst("");
    setSecond("");
    setError(undefined);
  }, [game.id]);
  const configDir = game.config_dir ?? "";
  const { data: diff = [], error: comparisonError } = useQuery({
    queryKey: ["snapshot-diff", game.id, configDir, first, second],
    enabled: !!first,
    queryFn: () =>
      invoke<ChangeOperation[]>("compare_snapshots", {
        gameId: game.id,
        configDir,
        backupId: first,
        otherBackupId: second || null,
      }),
  });
  const run = async (operation: () => Promise<unknown>) => {
    setBusy(true);
    setError(undefined);
    try {
      await operation();
      invalidateGameWorkspace(client, configDir, game.id);
      void client.invalidateQueries({ queryKey: ["snapshot-diff"] });
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  const restoreParameter = async (op: ChangeOperation) => {
    const key = op.key.replace(/ \[#\d+\]$/, "");
    const reprepare = () =>
      invoke<PreparedChanges>("prepare_parameter_restore", {
        gameId: game.id,
        configDir,
        backupId: first,
        file: op.file,
        section: op.section,
        key,
      });
    const plan = await reprepare();
    const request: PrepareRequest = {
      game_id: game.id,
      config_dir: configDir,
      changes: { files: {}, removals: {} },
      backup_id: null,
      restore_origin_backup_id: first,
      preset_metadata: null,
      input_revision: null,
      restore_files: [],
      input_updates: [],
    };
    await showPrepared(plan, request, reprepare);
  };
  return (
    <section className="space-y-3 rounded-xl border border-[var(--color-border)] p-4">
      <h3 className="font-semibold">{t("snapshots.title")}</h3>
      <div className="flex flex-wrap gap-2">
        <input
          aria-label={t("snapshots.name")}
          placeholder={t("snapshots.name")}
          maxLength={120}
          value={name}
          onChange={(event) => setName(event.target.value)}
          className="rounded border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-2"
        />
        <Button
          onClick={() =>
            void run(() =>
              invoke("create_snapshot", { gameId: game.id, configDir, name }),
            )
          }
          disabled={disabled || busy || !name.trim()}
        >
          {t("snapshots.create")}
        </Button>
        {first && (
          <Button
            variant="ghost"
            disabled={busy || !name.trim()}
            onClick={() =>
              void run(() =>
                invoke("rename_snapshot", {
                  gameId: game.id,
                  configDir,
                  backupId: first,
                  name,
                }),
              )
            }
          >
            {t("snapshots.rename")}
          </Button>
        )}
      </div>
      <div className="flex flex-wrap gap-2">
        <label>
          {t("snapshots.from")}{" "}
          <select
            aria-label={t("snapshots.from")}
            value={first}
            onChange={(event) => setFirst(event.target.value)}
            className="rounded bg-[var(--color-bg-soft)] p-2"
          >
            <option value="">—</option>
            {backups.map((backup) => (
              <option key={backup.id} value={backup.id}>
                {backup.name ?? backup.created_at}
              </option>
            ))}
          </select>
        </label>
        <label>
          {t("snapshots.to")}{" "}
          <select
            aria-label={t("snapshots.to")}
            value={second}
            onChange={(event) => setSecond(event.target.value)}
            className="rounded bg-[var(--color-bg-soft)] p-2"
          >
            <option value="">{t("snapshots.current")}</option>
            {backups.map((backup) => (
              <option key={backup.id} value={backup.id}>
                {backup.name ?? backup.created_at}
              </option>
            ))}
          </select>
        </label>
      </div>
      {(error || comparisonError) && (
        <p role="alert" className="text-[var(--color-danger)]">
          {error ?? formatInvokeError(comparisonError)}
        </p>
      )}
      {first && (
        <div className="max-h-96 overflow-auto">
          {diff.length === 0 && <p>{t("snapshots.identical")}</p>}
          {diff.map((op) => (
            <div
              key={op.id}
              className="space-y-1 border-t border-[var(--color-border)] py-2 text-sm"
            >
              <div>
                {op.file} · {op.section} ·{" "}
                {op.key === "*" ? t("preview.fileBytes") : op.key}
              </div>
              <div className="break-all font-mono">
                {op.before === null
                  ? t("preview.absent")
                  : op.before === ""
                    ? t("preview.empty")
                    : op.before}{" "}
                →{" "}
                {op.after === null
                  ? t("preview.absent")
                  : op.after === ""
                    ? t("preview.empty")
                    : op.after}
              </div>
              {!second && (
                <div className="flex gap-2">
                  <Button
                    size="sm"
                    variant="ghost"
                    disabled={
                      disabled ||
                      busy ||
                      op.key === "*" ||
                      / \[#\d+\]$/.test(op.key) ||
                      /^[+!.-]/.test(op.key)
                    }
                    onClick={() => void run(() => restoreParameter(op))}
                  >
                    {t("snapshots.restoreParameter")}
                  </Button>
                  <Button
                    size="sm"
                    variant="ghost"
                    disabled={disabled || busy}
                    onClick={() =>
                      void run(() =>
                        previewAndApply({
                          game_id: game.id,
                          config_dir: configDir,
                          backup_id: first,
                          restore_files: [op.file],
                        }),
                      )
                    }
                  >
                    {t("snapshots.restoreFile")}
                  </Button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}
