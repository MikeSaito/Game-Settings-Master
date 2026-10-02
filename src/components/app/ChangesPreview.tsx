import { useEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { formatInvokeError } from "@/lib/core";
import type {
  ApplyResult,
  ChangeIssue,
  PreparedChanges,
} from "@/lib/api/bindings";
import {
  discardPrepared,
  prepareChanges,
  subscribePreview,
  type PreviewRequest,
} from "@/lib/api/preparedChanges";

export function ChangesPreview() {
  const { t } = useTranslation("improvements");
  const translate = useRef(t);
  translate.current = t;
  const [pending, setPending] = useState<PreviewRequest>();
  const active = useRef<PreviewRequest | undefined>(undefined);
  const [plan, setPlan] = useState<PreparedChanges>();
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [issues, setIssues] = useState<ChangeIssue[]>([]);
  const [ack, setAck] = useState(false);
  const [busy, setBusy] = useState(false);
  const [validating, setValidating] = useState(false);
  const [error, setError] = useState<string>();
  const [completedWarning, setCompletedWarning] = useState<string>();
  const close = () => {
    const request = active.current;
    if (!request || busy) return;
    void discardPrepared(request.plan.id);
    request.reject(new Error(t("preview.cancelled")));
    active.current = undefined;
    setPending(undefined);
    setPlan(undefined);
  };
  useEffect(() => {
    const unsubscribe = subscribePreview((request) => {
      if (active.current) {
        void discardPrepared(request.plan.id);
        request.reject(new Error(translate.current("preview.alreadyOpen")));
        return;
      }
      active.current = request;
      setPending(request);
      setPlan(request.plan);
      setSelected(new Set(request.plan.operations.map((op) => op.id)));
      setIssues(request.plan.issues);
      setAck(false);
      setError(undefined);
    });
    return () => {
      unsubscribe();
      if (active.current) {
        void discardPrepared(active.current.plan.id);
        active.current.reject(
          new Error(translate.current("preview.cancelled")),
        );
        active.current = undefined;
      }
    };
  }, []);
  useEffect(() => {
    if (!plan) return;
    let cancelled = false;
    setValidating(true);
    setAck(false);
    void invoke<ChangeIssue[]>("validate_prepared_changes", {
      planId: plan.id,
      selectedIds: [...selected],
    })
      .then((issues) => {
        if (!cancelled) {
          setIssues(issues);
          setError(undefined);
        }
      })
      .catch((error) => {
        if (!cancelled) setError(formatInvokeError(error));
      })
      .finally(() => {
        if (!cancelled) setValidating(false);
      });
    return () => {
      cancelled = true;
    };
  }, [plan, selected]);
  useEffect(() => {
    if (!pending) return;
    const previous = document.activeElement as HTMLElement | null;
    const dialog = document.getElementById("changes-preview");
    dialog?.focus();
    const keyboard = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      }
      if (event.key === "Tab" && dialog) {
        const controls = [
          ...dialog.querySelectorAll<HTMLElement>(
            'button:not(:disabled), input:not(:disabled), [tabindex="0"]',
          ),
        ];
        if (controls.length === 0) return;
        const first = controls[0],
          last = controls[controls.length - 1];
        if (
          event.shiftKey &&
          (document.activeElement === first ||
            document.activeElement === dialog)
        ) {
          event.preventDefault();
          last.focus();
        } else if (
          !event.shiftKey &&
          (document.activeElement === last || document.activeElement === dialog)
        ) {
          event.preventDefault();
          first.focus();
        }
      }
    };
    document.addEventListener("keydown", keyboard);
    return () => {
      document.removeEventListener("keydown", keyboard);
      previous?.focus();
    };
  }, [pending, busy]);
  if (!pending || !plan)
    return completedWarning
      ? createPortal(
          <div
            role="status"
            className="fixed bottom-4 right-4 z-[100] max-w-xl rounded-xl border border-[var(--color-warning)] bg-[var(--color-surface)] p-4 text-[var(--color-text)]"
          >
            <p>{completedWarning}</p>
            <Button
              variant="ghost"
              onClick={() => setCompletedWarning(undefined)}
            >
              {t("cancel")}
            </Button>
          </div>,
          document.body,
        )
      : null;
  const toggle = (group: string, checked: boolean) =>
    setSelected((old) => {
      const next = new Set(old);
      plan.operations
        .filter((op) => op.group === group)
        .forEach((op) => (checked ? next.add(op.id) : next.delete(op.id)));
      return next;
    });
  const apply = async () => {
    setBusy(true);
    setError(undefined);
    try {
      const result = await invoke<ApplyResult>("apply_prepared_changes", {
        planId: plan.id,
        selectedIds: [...selected],
        warningsAcknowledged: ack,
      });
      setCompletedWarning(result.post_apply_warning ?? undefined);
      active.current = undefined;
      setPending(undefined);
      setPlan(undefined);
      pending.resolve(result);
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  const refresh = async () => {
    setBusy(true);
    setError(undefined);
    try {
      const next = await (pending.reprepare
        ? pending.reprepare()
        : prepareChanges(pending.request));
      await discardPrepared(plan.id);
      pending.plan = next;
      setPlan(next);
      setSelected(new Set(next.operations.map((op) => op.id)));
      setIssues(next.issues);
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };
  return createPortal(
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/70 p-4">
      <section
        id="changes-preview"
        role="dialog"
        aria-modal="true"
        aria-labelledby="preview-title"
        tabIndex={-1}
        className="flex max-h-[90vh] w-full max-w-5xl flex-col rounded-xl border border-[var(--color-border)] bg-[var(--color-surface)] p-5 text-[var(--color-text)] shadow-xl"
      >
        <h2 id="preview-title" className="text-lg font-semibold">
          {t("preview.title")}
        </h2>
        <p className="mb-3 break-all text-xs text-[var(--color-text-muted)]">
          {plan.config_dir}
        </p>
        <div className="min-h-0 flex-1 overflow-auto">
          <table className="w-full text-sm">
            <thead>
              <tr>
                <th>{t("preview.select")}</th>
                <th>{t("preview.parameter")}</th>
                <th>{t("preview.before")}</th>
                <th>{t("preview.after")}</th>
              </tr>
            </thead>
            <tbody>
              {plan.operations.map((op) => (
                <tr
                  key={op.id}
                  className="border-t border-[var(--color-border)] align-top"
                >
                  <td className="p-2">
                    <input
                      type="checkbox"
                      aria-label={`${op.file}: ${op.key}`}
                      checked={selected.has(op.id)}
                      disabled={busy}
                      onChange={(event) =>
                        toggle(op.group, event.target.checked)
                      }
                    />
                  </td>
                  <td className="p-2">
                    <span className="text-xs text-[var(--color-text-muted)]">
                      {op.file} · {op.section}
                    </span>
                    <div>
                      {op.key === "*" ? t("preview.fileBytes") : op.key}
                    </div>
                    {op.kind === "file" && (
                      <small>{t("preview.wholeFile")}</small>
                    )}
                    {op.kind !== "file" &&
                      plan.operations.filter(
                        (other) => other.group === op.group,
                      ).length > 1 && (
                        <small className="block text-[var(--color-text-muted)]">
                          {t("preview.linked")}
                        </small>
                      )}
                    {op.kind.startsWith("scalar:") && (
                      <small className="block text-[var(--color-text-muted)]">
                        {t("preview.occurrence", {
                          number: Number(op.kind.slice(7)) + 1,
                        })}
                      </small>
                    )}
                  </td>
                  <td className="max-w-xs whitespace-pre-wrap break-all p-2 font-mono">
                    {op.before === null
                      ? t("preview.absent")
                      : op.before === ""
                        ? t("preview.empty")
                        : op.before.slice(0, 2000)}
                  </td>
                  <td className="max-w-xs whitespace-pre-wrap break-all p-2 font-mono">
                    {op.after === null
                      ? t("preview.delete")
                      : op.after === ""
                        ? t("preview.empty")
                        : op.after.slice(0, 2000)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        {issues.map((issue, index) => (
          <p
            key={`${issue.code}:${index}`}
            className={
              issue.severity === "error"
                ? "text-[var(--color-danger)]"
                : "text-[var(--color-warning)]"
            }
          >
            {issue.message}
          </p>
        ))}
        {issues.some((issue) => issue.severity === "warning") && (
          <label className="my-2 flex gap-2">
            <input
              type="checkbox"
              checked={ack}
              onChange={(event) => setAck(event.target.checked)}
            />
            {t("preview.ack")}
          </label>
        )}
        {error && (
          <p role="alert" className="my-2 text-[var(--color-danger)]">
            {error}
          </p>
        )}
        <div className="mt-4 flex justify-end gap-2">
          <Button variant="ghost" onClick={close} disabled={busy}>
            {t("cancel")}
          </Button>
          <Button
            variant="secondary"
            onClick={() => void refresh()}
            disabled={busy}
          >
            {t("preview.refresh")}
          </Button>
          <Button
            variant="primary"
            onClick={() => void apply()}
            loading={busy}
            disabled={
              validating ||
              !!error ||
              selected.size === 0 ||
              issues.some((issue) => issue.severity === "error") ||
              (issues.some((issue) => issue.severity === "warning") && !ack)
            }
          >
            {t("preview.confirm")}
          </Button>
        </div>
      </section>
    </div>,
    document.body,
  );
}
