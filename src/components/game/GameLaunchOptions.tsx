import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { openPathDialog } from "@/lib/api";
import { formatInvokeError, type GameProfile } from "@/lib/core";

export function GameLaunchOptions({ game }: { game: GameProfile }) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const [open, setOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const { data: executables = [], error: listError } = useQuery({
    queryKey: ["game-executables", game.id],
    queryFn: () =>
      invoke<string[]>("list_game_executables", { gameId: game.id }),
    enabled: open && game.launch_target?.kind !== "package",
  });
  const select = async (exePath: string) => {
    if (!exePath) return;
    setBusy(true);
    setError(undefined);
    try {
      await invoke("set_game_executable", { gameId: game.id, exePath });
      await client.invalidateQueries({ queryKey: ["games"] });
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  if (game.launch_target?.kind === "package")
    return <p className="mt-2 text-xs">{t("launch.nativePackage")}</p>;
  return (
    <details
      onToggle={(event) => setOpen(event.currentTarget.open)}
      className="mt-2 text-xs"
    >
      <summary>{t("launch.options")}</summary>
      <div className="mt-2 flex flex-wrap items-center gap-2">
        <select
          aria-label={t("launch.select")}
          value={
            game.launch_target?.kind === "exe" ? game.launch_target.value : ""
          }
          disabled={busy}
          onChange={(event) => void select(event.target.value)}
          className="max-w-full rounded bg-[var(--color-surface)] p-2"
        >
          <option value="">{t("launch.select")}</option>
          {executables?.map((path) => (
            <option key={path} value={path}>
              {path}
            </option>
          ))}
        </select>
        <Button
          variant="ghost"
          size="sm"
          disabled={busy}
          onClick={() =>
            void openPathDialog({
              filters: [{ name: "Windows EXE", extensions: ["exe"] }],
              title: t("launch.select"),
            })
              .then(async (path) => {
                if (path) await select(path);
              })
              .catch((error) => setError(formatInvokeError(error)))
          }
        >
          {t("launch.browse")}
        </Button>
        {game.launch_target && (
          <span className="break-all">{game.launch_target.value}</span>
        )}
        {(error || listError) && (
          <p role="alert">{error ?? formatInvokeError(listError)}</p>
        )}
      </div>
    </details>
  );
}
