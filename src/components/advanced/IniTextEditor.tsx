import { useEffect, useRef, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ds/Button";
import { Alert } from "@/components/ds/Feedback";
import { formatInvokeError, type GameProfile } from "@/lib/core";
import type { IniTextDocument } from "@/lib/api/bindings";
import { previewAndApply } from "@/lib/api/preparedChanges";
import { invalidateGameWorkspace } from "@/lib/game/invalidateGameWorkspace";
import { updateTextDraft } from "@/lib/editor/textDraft";

const FILES = [
  "Engine.ini",
  "GameUserSettings.ini",
  "Scalability.ini",
  "Game.ini",
  "Input.ini",
];
type Draft = { document: IniTextDocument; text: string };
const control =
  "rounded border border-[var(--color-border)] bg-[var(--color-bg-soft)] px-3 py-2 text-sm";

export function IniTextEditor({
  game,
  running,
  active,
}: {
  game: GameProfile;
  running: boolean;
  active: boolean;
}) {
  const { t } = useTranslation("improvements");
  const client = useQueryClient();
  const [file, setFile] = useState(FILES[0]);
  const [drafts, setDrafts] = useState<Record<string, Draft>>({});
  const [busy, setBusy] = useState(false);
  const [confirmReload, setConfirmReload] = useState(false);
  const [message, setMessage] = useState("");
  const [error, setError] = useState("");
  const [search, setSearch] = useState("");
  const textArea = useRef<HTMLTextAreaElement>(null);
  const configDir = game.config_dir ?? "";
  const queryKey = ["ini-text", game.id, configDir, file];
  const read = (name: string) =>
    invoke<IniTextDocument>("get_ini_text_document", {
      gameId: game.id,
      configDir,
      file: name,
    });
  const query = useQuery({
    queryKey,
    queryFn: () => read(file),
    enabled: active,
  });
  useEffect(() => {
    if (!query.data) return;
    const document = query.data;
    setDrafts((previous) => {
      const existing = previous[document.file];
      if (existing && existing.text !== existing.document.text) return previous;
      if (existing?.document === document) return previous;
      return {
        ...previous,
        [document.file]: { document, text: document.text },
      };
    });
  }, [query.data]);
  const draft = drafts[file];
  const modified = Object.values(drafts).filter(
    (item) => item.text !== item.document.text,
  );
  const stale =
    !!draft &&
    !!query.data &&
    (draft.document.revision !== query.data.revision ||
      draft.document.config_dir !== query.data.config_dir);

  async function save() {
    if (busy || running || !modified.length) return;
    setBusy(true);
    setMessage("");
    setError("");
    try {
      const result = await previewAndApply({
        game_id: game.id,
        config_dir: configDir,
        raw_files: modified.map((item) => ({
          file: item.document.file,
          config_dir: item.document.config_dir,
          revision: item.document.revision,
          text: item.text,
        })),
      });
      const reloadErrors: string[] = [];
      for (const name of result.changed_files) {
        try {
          const document = await read(name);
          client.setQueryData(["ini-text", game.id, configDir, name], document);
          setDrafts((previous) => ({
            ...previous,
            [name]: { document, text: document.text },
          }));
        } catch (cause) {
          reloadErrors.push(`${name}: ${formatInvokeError(cause)}`);
        }
      }
      invalidateGameWorkspace(client, configDir, game.id);
      if (result.changed_files.length)
        setMessage(t("files.saved", { count: result.changed_files.length }));
      setError(
        [result.post_apply_warning, ...reloadErrors].filter(Boolean).join("; "),
      );
    } catch (cause) {
      setError(formatInvokeError(cause));
    } finally {
      setBusy(false);
    }
  }
  async function reload() {
    setBusy(true);
    setError("");
    setMessage("");
    try {
      const document = await read(file);
      client.setQueryData(queryKey, document);
      setDrafts((previous) => ({
        ...previous,
        [file]: { document, text: document.text },
      }));
      setConfirmReload(false);
    } catch (cause) {
      setError(formatInvokeError(cause));
    } finally {
      setBusy(false);
    }
  }
  function find() {
    const area = textArea.current;
    if (!area || !search) return;
    const haystack = area.value.toLocaleLowerCase();
    const needle = search.toLocaleLowerCase();
    const position = haystack.indexOf(needle, area.selectionEnd);
    const match = position >= 0 ? position : haystack.indexOf(needle);
    if (match < 0) {
      setMessage(t("files.notFound"));
      return;
    }
    setMessage("");
    area.focus();
    area.setSelectionRange(match, match + search.length);
    const line = area.value.slice(0, match).split("\n").length - 1;
    area.scrollTop = Math.max(0, line * 24 - area.clientHeight / 2);
  }
  return (
    <section
      className="flex min-h-0 flex-1 flex-col gap-2"
      aria-label={t("files.title")}
      onKeyDown={(event) => {
        if (
          (event.ctrlKey || event.metaKey) &&
          event.key.toLowerCase() === "s"
        ) {
          event.preventDefault();
          void save();
        }
      }}
    >
      <div className="flex shrink-0 flex-wrap items-center gap-2">
        <label className="sr-only" htmlFor="ini-file">
          {t("files.select")}
        </label>
        <select
          id="ini-file"
          className={control}
          value={file}
          disabled={busy}
          onChange={(event) => {
            setFile(event.target.value);
            setConfirmReload(false);
            setMessage("");
            setError("");
          }}
        >
          {FILES.map((name) => (
            <option key={name} value={name}>
              {name}
              {drafts[name] && drafts[name].text !== drafts[name].document.text
                ? " *"
                : ""}
            </option>
          ))}
        </select>
        <Button
          size="sm"
          disabled={busy}
          onClick={() =>
            draft && draft.text !== draft.document.text
              ? setConfirmReload(true)
              : void reload()
          }
        >
          {t("files.reload")}
        </Button>
        <form
          className="ml-auto flex min-w-0 gap-2"
          onSubmit={(event) => {
            event.preventDefault();
            find();
          }}
        >
          <input
            className={`${control} w-40 min-w-0`}
            aria-label={t("files.find")}
            placeholder={t("files.find")}
            value={search}
            onChange={(event) => setSearch(event.target.value)}
          />
          <Button size="sm" type="submit" disabled={!search || !draft}>
            {t("files.next")}
          </Button>
        </form>
      </div>
      {confirmReload && (
        <div
          className="flex shrink-0 flex-wrap items-center gap-2 text-sm"
          role="alert"
        >
          <span>{t("files.discardHint")}</span>
          <Button
            size="sm"
            variant="danger"
            disabled={busy}
            onClick={() => void reload()}
          >
            {t("files.discard")}
          </Button>
          <Button size="sm" onClick={() => setConfirmReload(false)}>
            {t("cancel")}
          </Button>
        </div>
      )}
      <div
        className="max-h-24 shrink-0 overflow-y-auto text-sm"
        aria-live="polite"
      >
        {(error || query.error) && (
          <Alert tone="danger">{error || formatInvokeError(query.error)}</Alert>
        )}
        {message && <Alert tone="info">{message}</Alert>}
        {running && <Alert tone="warning">{t("files.running")}</Alert>}
        {stale && <Alert tone="warning">{t("files.stale")}</Alert>}
        {draft?.document.revision === null && <p>{t("files.missing")}</p>}
      </div>
      <label className="sr-only" htmlFor="ini-text">
        {t("files.title")}
      </label>
      <textarea
        id="ini-text"
        ref={textArea}
        className="min-h-0 w-full flex-1 resize-none rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-soft)] p-3 font-mono text-sm leading-6 text-[var(--color-text)] outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-accent)]"
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        wrap="off"
        readOnly={!draft || busy}
        placeholder={query.isLoading ? t("files.readOnly") : ""}
        value={draft?.text.replace(/\r\n/g, "\n") ?? ""}
        onChange={(event) => {
          const value = event.target.value;
          setDrafts((previous) => ({
            ...previous,
            [file]: {
              ...previous[file],
              text: updateTextDraft(previous[file].text, value),
            },
          }));
          setMessage("");
        }}
      />
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 text-xs text-[var(--color-text-muted)]">
        <span className="min-w-0 break-all">{draft?.document.config_dir}</span>
        {draft && (
          <span>
            {draft.document.encoding} ·{" "}
            {draft.text.includes("\r\n") ? "CRLF" : "LF"} ·{" "}
            {t("files.lines", { count: draft.text.split("\n").length })}
          </span>
        )}
      </div>
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-2">
        <span className="text-sm">
          {modified.length
            ? t("files.dirty", { count: modified.length })
            : t("files.clean")}
        </span>
        <Button
          variant="primary"
          size="sm"
          loading={busy}
          disabled={running || !modified.length}
          onClick={() => void save()}
        >
          {t("files.save")}
        </Button>
      </div>
      <p className="shrink-0 text-xs text-[var(--color-text-muted)]">
        {t("files.hint")}
      </p>
    </section>
  );
}
