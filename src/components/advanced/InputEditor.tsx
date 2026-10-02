import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { formatInvokeError, type GameProfile } from "@/lib/core";
import type { InputDocument } from "@/lib/api/bindings";
import { previewAndApply } from "@/lib/api/preparedChanges";
import { saveGameOverride } from "@/lib/api";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";

const serialize = (fields: Record<string, string>) =>
  `(${Object.entries(fields)
    .map(([key, value]) => `${key}=${value}`)
    .join(",")})`;
const controlClass =
  "ml-2 rounded border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-2";

export function InputEditor({
  game,
  running,
}: {
  game: GameProfile;
  running: boolean;
}) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const configDir = game.config_dir ?? "";
  const {
    data,
    error: queryError,
    isLoading,
  } = useQuery({
    queryKey: ["input-document", game.id, configDir],
    queryFn: () =>
      invoke<InputDocument>("get_input_document", {
        gameId: game.id,
        configDir,
      }),
  });
  const [values, setValues] = useState<Record<number, Record<string, string>>>(
    {},
  );
  const [axisValues, setAxisValues] = useState<
    Record<number, Record<string, string>>
  >({});
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [name, setName] = useState("");
  const [mode, setMode] = useState("changes");
  useEffect(() => {
    setValues({});
    setAxisValues({});
    setError(undefined);
    setName("");
    setMode("changes");
  }, [game.id, configDir]);
  const updates = (data?.entries ?? [])
    .filter((entry) => values[entry.line] && entry.editable)
    .map((entry) => ({
      id: entry.id,
      line: entry.line,
      expected: entry.value,
      value: serialize(values[entry.line]),
    }))
    .filter((update) => update.value !== update.expected);

  const run = async (save: boolean) => {
    setBusy(true);
    setError(undefined);
    try {
      if (save) {
        await saveGameOverride({
          game_id: game.id,
          name,
          files: {},
          removals: {},
          input_updates:
            mode === "profile"
              ? (data?.entries ?? [])
                  .filter((entry) => entry.editable)
                  .map((entry) => ({
                    id: entry.id,
                    line: entry.line,
                    expected: entry.value,
                    value: values[entry.line]
                      ? serialize(values[entry.line])
                      : entry.value,
                  }))
              : updates,
          metadata: {
            format_version: 2,
            mode,
            description: "Input.ini",
            source_game_id: game.id,
            source_game_name: game.name,
            game_build: game.build_id ?? null,
            engine_family: game.engine_family ?? null,
            engine_version: game.engine_version ?? null,
            gpu_vendor: null,
            min_dedicated_memory_mb: null,
            requires_ray_tracing: false,
          },
        });
        void client.invalidateQueries({ queryKey: ["overrides", game.id] });
      } else {
        const result = await previewAndApply({
          game_id: game.id,
          config_dir: configDir,
          input_updates: updates,
          input_revision: data?.revision,
        });
        const applied = new Set(result.applied_input_lines ?? []);
        const preserve = (draft: Record<number, Record<string, string>>) =>
          Object.fromEntries(
            Object.entries(draft).filter(
              ([line]) => !applied.has(Number(line)),
            ),
          );
        setValues(preserve);
        setAxisValues(preserve);
        void client.invalidateQueries({
          queryKey: ["input-document", game.id],
        });
        invalidateGameWorkspace(client, configDir, game.id);
      }
    } catch (error) {
      setError(formatInvokeError(error));
    } finally {
      setBusy(false);
    }
  };

  if (isLoading) return <p>{t("loading")}</p>;
  return (
    <section className="space-y-3 overflow-auto p-2">
      <h2 className="font-semibold">{t("input.title")}</h2>
      <p className="text-sm text-[var(--color-text-muted)]">
        {t("input.scope")}
      </p>
      {(error || queryError) && (
        <p role="alert">{error ?? formatInvokeError(queryError)}</p>
      )}
      {!data?.entries.length && <p>{t("input.empty")}</p>}
      {data?.entries.map((entry) => {
        const fields = values[entry.line] ?? entry.fields;
        const title =
          fields.ActionName ??
          fields.AxisName ??
          fields.AxisKeyName ??
          entry.key;
        const axis = axisValues[entry.line] ?? entry.axis_properties;
        const allowed = entry.key.endsWith("ActionMappings")
          ? ["Key", "bShift", "bCtrl", "bAlt", "bCmd"]
          : entry.key.endsWith("AxisMappings")
            ? ["Key", "Scale"]
            : [];
        const disabled = !entry.editable || running || busy;
        const fieldControl = (
          key: string,
          value: string,
          update: (value: string) => void,
        ) => {
          const boolean = key.startsWith("b");
          const label = t(`input.fields.${key}`, { defaultValue: key });
          return (
            <label key={key} className="text-sm">
              {label}
              <input
                aria-label={`${title}: ${label}`}
                disabled={disabled}
                type={boolean ? "checkbox" : key === "Key" ? "text" : "number"}
                step={boolean || key === "Key" ? undefined : "any"}
                min={
                  ["Sensitivity", "DeadZone", "Exponent"].includes(key)
                    ? 0
                    : undefined
                }
                max={key === "DeadZone" ? 1 : undefined}
                checked={boolean ? value.toLowerCase() === "true" : undefined}
                value={boolean ? undefined : value}
                onChange={(event) =>
                  update(
                    boolean ? String(event.target.checked) : event.target.value,
                  )
                }
                className={controlClass}
              />
            </label>
          );
        };
        return (
          <div
            key={entry.line}
            className="space-y-2 rounded border border-[var(--color-border)] p-3"
          >
            <h3>{title}</h3>
            {!entry.editable && (
              <>
                <p className="text-sm text-[var(--color-warning)]">
                  {entry.reason}
                </p>
                <pre className="whitespace-pre-wrap break-all text-xs">
                  {entry.value}
                </pre>
              </>
            )}
            <div className="flex flex-wrap gap-3">
              {allowed
                .filter((key) => key in fields)
                .map((key) =>
                  fieldControl(key, fields[key], (value) =>
                    setValues((draft) => ({
                      ...draft,
                      [entry.line]: { ...fields, [key]: value },
                    })),
                  ),
                )}
              {axis &&
                ["Sensitivity", "DeadZone", "Exponent", "bInvert"]
                  .filter((key) => key in axis)
                  .map((key) =>
                    fieldControl(key, axis[key], (value) => {
                      const next = { ...axis, [key]: value };
                      setAxisValues((draft) => ({
                        ...draft,
                        [entry.line]: next,
                      }));
                      setValues((draft) => ({
                        ...draft,
                        [entry.line]: {
                          ...fields,
                          AxisProperties: serialize(next),
                        },
                      }));
                    }),
                  )}
            </div>
          </div>
        );
      })}
      <div className="flex flex-wrap gap-2">
        <Button
          loading={busy}
          disabled={running || busy || updates.length === 0}
          onClick={() => void run(false)}
        >
          {t("preview.apply")}
        </Button>
        <input
          aria-label={t("snapshots.name")}
          maxLength={120}
          placeholder={t("snapshots.name")}
          value={name}
          onChange={(event) => setName(event.target.value)}
          className="rounded bg-[var(--color-bg-soft)] p-2"
        />
        <label>
          {t("presets.mode")}{" "}
          <select
            value={mode}
            onChange={(event) => setMode(event.target.value)}
            className={controlClass}
          >
            <option value="changes">{t("presets.changes")}</option>
            <option value="profile">{t("presets.profile")}</option>
          </select>
        </label>
        <Button
          variant="secondary"
          disabled={
            busy ||
            (mode === "changes"
              ? updates.length === 0
              : !data?.entries.some((entry) => entry.editable)) ||
            !name.trim()
          }
          onClick={() => void run(true)}
        >
          {t("input.savePreset")}
        </Button>
      </div>
    </section>
  );
}
