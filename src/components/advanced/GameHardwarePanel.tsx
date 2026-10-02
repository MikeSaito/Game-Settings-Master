import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import type { GameProfile, GpuCapabilities } from "@/lib/core";
import { formatInvokeError } from "@/lib/core";

export function GameHardwarePanel({
  game,
  gpu,
}: {
  game: GameProfile;
  gpu?: GpuCapabilities;
}) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const { data: adapters = [] } = useQuery({
    queryKey: ["gpu-adapters"],
    queryFn: () => invoke<GpuCapabilities[]>("list_gpu_adapters"),
    staleTime: 300_000,
  });
  const [error, setError] = useState<string>();
  const [busy, setBusy] = useState(false);
  const select = async (id: string) => {
    setBusy(true);
    setError(undefined);
    try {
      await invoke("set_game_gpu", { gameId: game.id, adapterId: id || null });
      await client.invalidateQueries({ queryKey: ["gpu", game.id] });
      await client.invalidateQueries({ queryKey: ["games"] });
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  return (
    <details className="mb-3 rounded border border-[var(--color-border)] p-3 text-sm">
      <summary>
        {t("gpu.title")} · {gpu?.name ?? t("gpu.unknown")}
      </summary>
      <label className="mt-2 block">
        {t("gpu.selection")}{" "}
        <select
          aria-label={t("gpu.selection")}
          disabled={busy}
          value={
            game.gpu_adapter_id && !gpu?.selection_warning
              ? (gpu?.adapter_id ?? game.gpu_adapter_id)
              : ""
          }
          onChange={(event) => void select(event.target.value)}
          className="rounded bg-[var(--color-bg-soft)] p-2"
        >
          <option value="">{t("gpu.auto")}</option>
          {adapters?.map((adapter) => (
            <option
              key={adapter.adapter_id ?? adapter.name}
              value={adapter.adapter_id ?? ""}
            >
              {adapter.name}
            </option>
          ))}
        </select>
      </label>
      <p>
        {t("gpu.dedicated")}:{" "}
        {gpu?.dedicated_memory_mb == null
          ? t("gpu.unknown")
          : `${gpu.dedicated_memory_mb} MB`}{" "}
        · {t("gpu.shared")}:{" "}
        {gpu?.shared_memory_mb == null
          ? t("gpu.unknown")
          : `${gpu.shared_memory_mb} MB`}
      </p>
      <p>
        {t("gpu.rt")}: {t(`gpu.${gpu?.ray_tracing_status ?? "unknown"}`)}
      </p>
      <p className="text-[var(--color-text-muted)]">{t("gpu.scope")}</p>
      {gpu?.selection_warning && (
        <p className="text-[var(--color-warning)]">{gpu.selection_warning}</p>
      )}
      <Button
        variant="ghost"
        size="sm"
        onClick={() =>
          void invoke("open_graphics_settings").catch((error) =>
            setError(formatInvokeError(error)),
          )
        }
      >
        {t("gpu.windows")}
      </Button>
      {error && <p role="alert">{error}</p>}
    </details>
  );
}
