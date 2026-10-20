import { useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { isTauriRuntime } from "@/lib/api";
import { formatInvokeError } from "@/lib/core";
import type { PendingOperation } from "@/lib/api/bindings";

export function RecoveryPanel() {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const [deferred, setDeferred] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const pending = useQuery({
    queryKey: ["pending-operations"],
    queryFn: () => invoke<PendingOperation[]>("list_pending_operations"),
    enabled: isTauriRuntime(),
    refetchInterval: 5000,
  });
  const recover = async (id: string) => {
    setBusy(true); setError(undefined);
    try {
      await invoke("recover_pending_operation", { id });
      await client.invalidateQueries();
    } catch (err) { setError(formatInvokeError(err)); }
    finally { setBusy(false); }
  };
  if (!pending.error && !pending.data?.length) return null;
  return (
    <section aria-label={t("recovery.title")} data-testid="recovery-panel"
      className={`fixed right-4 z-[90] max-h-[80vh] overflow-auto rounded border border-[var(--color-border)] bg-[var(--color-bg)] p-4 shadow-xl ${deferred ? "top-4 w-72" : "bottom-4 w-[32rem]"}`}>
      <h2 className="font-semibold">{t("recovery.title")}</h2>
      <p>{t("recovery.blocked")}</p>
      {pending.error && <p role="alert">{formatInvokeError(pending.error)}</p>}
      {!deferred && pending.data?.map((operation) => (
        <div key={operation.id} className="my-3">
          <p className="break-all">{operation.folders.join("\n") || operation.id}</p>
          {operation.error && <p role="alert">{operation.error}</p>}
          <Button disabled={busy || !!operation.error} onClick={() => void recover(operation.id)}>
            {t("recovery.restore")}
          </Button>
        </div>
      ))}
      {error && <p role="alert">{error}</p>}
      <Button variant="ghost" disabled={busy} onClick={() => setDeferred(!deferred)}>
        {t(deferred ? "recovery.show" : "recovery.defer")}
      </Button>
    </section>
  );
}
