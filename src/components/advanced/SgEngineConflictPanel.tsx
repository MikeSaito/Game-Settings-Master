import { ChevronDown, ChevronUp, Wrench, AlertTriangle } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Alert } from "@/components/ds/Feedback";
import { Badge } from "@/components/ds/Badge";
import { Button } from "@/components/ds/Button";
import { Panel } from "@/components/ds/Panel";
import type { SgEngineConflictGroup } from "@/lib/editor/sgEngineConflicts";
import { SG_R_PREFIX } from "@/lib/editor/sgEngineConflicts";

interface Props {
  groups: SgEngineConflictGroup[];
  onResolve: (sgKey: string) => void;
}

function resetPrefixForGroup(group: SgEngineConflictGroup): string {
  const prefix = SG_R_PREFIX[group.sgKey];
  if (prefix) return `${prefix}*`;
  const keys = group.conflictingRParams.map((p) => p.key);
  if (keys.length === 1) return keys[0]!;
  const head = keys[0]!.split(".")[0];
  return keys.every((k) => k.startsWith(`${head}.`)) ? `${head}.*` : keys.join(", ");
}

function ConflictGroupCard({
  group,
  onResolve,
}: {
  group: SgEngineConflictGroup;
  onResolve: (sgKey: string) => void;
}) {
  const { t } = useTranslation("advanced");
  const [tierOpen, setTierOpen] = useState(false);
  const rPrefix = resetPrefixForGroup(group);

  return (
    <Panel padding="md" className="border-[var(--color-warning)]/35 bg-[var(--color-warning-soft)]/40">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0 flex-1 space-y-2">
          <div className="flex flex-wrap items-center gap-2">
            <code className="font-mono text-xs text-[var(--color-text)]">{group.sgParam.key}</code>
            <Badge tone="accent">
              {t("conflict.sgValue", { value: group.sgValue })}
            </Badge>
          </div>
          <p className="text-xs text-[var(--color-text-secondary)]">
            {t("conflict.overridesList", {
              keys: group.conflictingRParams.map((p) => p.key).join(", "),
            })}
          </p>
        </div>
        <Button
          variant="secondary"
          size="sm"
          icon={<Wrench size={14} />}
          onClick={() => onResolve(group.sgKey)}
          className="shrink-0"
        >
          {t("conflict.resetRKeepSg", { sg: group.sgParam.key, prefix: rPrefix })}
        </Button>
      </div>

      {group.tierPreview && (
        <div className="mt-3 border-t border-[var(--color-border)] pt-3">
          <button
            type="button"
            onClick={() => setTierOpen((open) => !open)}
            className="flex w-full items-center justify-between gap-2 text-left text-xs font-medium text-[var(--color-text-secondary)] hover:text-[var(--color-text)]"
          >
            <span>
              {t("conflict.tierForSg", {
                sg: group.sgParam.key,
                value: group.sgValue,
                tier: group.tierPreview.tierLabel,
              })}
            </span>
            {tierOpen ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
          </button>
          {tierOpen && (
            <ul className="mt-2 space-y-1 font-mono text-xs text-[var(--color-text-muted)]">
              {group.tierPreview.cvars.map((row) => (
                <li key={row.key}>
                  <span className="text-[var(--color-accent-hover)]">{row.key}</span>
                  <span className="text-[var(--color-text-faint)]"> = </span>
                  <span>{row.value}</span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}
    </Panel>
  );
}

export function SgEngineConflictPanel({ groups, onResolve }: Props) {
  const { t } = useTranslation("advanced");
  if (groups.length === 0) return null;

  const conflictKeyCount = new Set(
    groups.flatMap((g) => [
      g.sgKey,
      ...g.conflictingRParams.map((p) => p.key.toLowerCase()),
    ]),
  ).size;

  return (
    <Alert tone="warning" icon={AlertTriangle} className="mb-3 space-y-3" title={t("conflict.bannerTitle")}>
      <p>{t("conflict.bannerBody", { count: conflictKeyCount })}</p>
      <div className="space-y-2">
        {groups.map((group) => (
          <ConflictGroupCard key={group.sgKey} group={group} onResolve={onResolve} />
        ))}
      </div>
    </Alert>
  );
}
