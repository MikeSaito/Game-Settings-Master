import type { Copy } from "./content";
import { escapeHtml as e } from "./site/render";

type Mode = "basic" | "advanced";
type Stage = "edit" | "preview" | "result";
interface Parameter {
  key: string;
  file: string;
  section: string;
  original: string;
  options: string[];
  name: number;
}
const parameters: Record<Mode, Parameter[]> = {
  basic: [
    {
      key: "bUseVSync",
      file: "GameUserSettings.ini",
      section: "/Script/Engine.GameUserSettings",
      original: "True",
      options: ["True", "False"],
      name: 0,
    },
    {
      key: "sg.ShadowQuality",
      file: "GameUserSettings.ini",
      section: "ScalabilityGroups",
      original: "3",
      options: ["3", "2"],
      name: 1,
    },
  ],
  advanced: [
    {
      key: "r.Shadow.MaxResolution",
      file: "Engine.ini",
      section: "SystemSettings",
      original: "2048",
      options: ["2048", "1024"],
      name: 2,
    },
    {
      key: "r.Lumen.Reflections.Allow",
      file: "Engine.ini",
      section: "SystemSettings",
      original: "1",
      options: ["1", "0"],
      name: 3,
    },
  ],
};

export function initDemo(root: HTMLElement, t: Copy): () => void {
  let mode: Mode = "basic";
  let stage: Stage = "edit";
  let drafts = initial();
  let selected = new Set<string>();
  let result: { key: string; value: string }[] = [];
  const d = t.demo;
  function initial() {
    return Object.fromEntries(
      Object.values(parameters)
        .flat()
        .map((p) => [p.key, p.original]),
    );
  }
  const changes = () =>
    parameters[mode].filter((p) => drafts[p.key] !== p.original);
  const valueLabel = (p: Parameter, value: string) =>
    p.name === 1
      ? value === "3"
        ? d.epic
        : d.high
      : p.name === 2
        ? value
        : ["True", "1"].includes(value)
          ? d.on
          : d.off;
  const actions = (body: string) => `<div class="demo__actions">${body}</div>`;
  function render(focus?: string) {
    root.dataset.stage = stage;
    const tabs = `<div class="demo__tabs" role="tablist" aria-label="${e(d.title)}">${(["basic", "advanced"] as const).map((m, i) => `<button type="button" role="tab" id="mode-${m}" aria-selected="${mode === m}" aria-controls="demo-panel" tabindex="${mode === m ? 0 : -1}" data-mode="${m}">${e(d.modes[i])}</button>`).join("")}</div>`;
    let body = "";
    if (stage === "edit") {
      body =
        parameters[mode]
          .map(
            (p) =>
              `<div class="parameter"><div class="parameter__label"><label for="${p.key}">${e(d.names[p.name])}</label><code>${e(p.key)}</code><small>${e(p.file)}</small></div><select id="${p.key}" data-key="${p.key}">${p.options.map((v) => `<option value="${v}" ${drafts[p.key] === v ? "selected" : ""}>${e(valueLabel(p, v))}</option>`).join("")}</select><p class="parameter__original">${e(d.original)}: ${e(valueLabel(p, p.original))}</p></div>`,
          )
          .join("") +
        actions(
          `<p class="demo__count" role="status">${changes().length ? `${e(d.changed)}: <b>${changes().length}</b>` : e(d.idle)}</p><button type="button" class="button button--primary" data-action="preview" ${changes().length ? "" : "disabled"}>${e(d.preview)} <span aria-hidden="true">→</span></button>`,
        );
    } else if (stage === "preview") {
      body =
        `<div class="diff-heading"><span>${e(d.before)}</span><span aria-hidden="true">→</span><span>${e(d.after)}</span></div>` +
        changes()
          .map(
            (p) =>
              `<label class="diff-row ${selected.has(p.key) ? "" : "diff-row--excluded"}"><input type="checkbox" data-select="${p.key}" aria-label="${e(d.include)}: ${e(p.key)}" ${selected.has(p.key) ? "checked" : ""}><span class="diff-row__body"><small>${e(p.file)}<br><span class="diff-section">[${e(p.section)}]</span></small><code>${e(p.key)}</code><span class="diff-values"><span class="diff-before">${e(p.original)}</span><span aria-hidden="true">→</span><span class="diff-after">${e(drafts[p.key])}</span></span></span></label>`,
          )
          .join("") +
        actions(
          `<p class="demo__count" role="status">${selected.size ? `${e(d.selected)}: <b>${selected.size}</b>` : e(d.empty)}</p><button class="button button--primary" data-action="confirm" ${selected.size ? "" : "disabled"}>${e(d.confirm)}</button><button class="demo__back" data-action="edit">← ${e(d.edit)}</button>`,
        );
    } else {
      body =
        `<div class="demo-result"><span class="result-check" aria-hidden="true">✓</span><h3 tabindex="-1" id="demo-done">${e(d.done)}</h3><p>${e(d.result)}</p>${result.map((p) => `<div class="result-value"><code>${e(p.key)}</code><b>${e(p.value)}</b></div>`).join("")}</div>` +
        actions(
          `<button class="button button--primary" data-action="reset">${e(d.reset)} ↺</button>`,
        );
    }
    root.innerHTML = `${tabs}<div id="demo-panel" role="tabpanel" aria-labelledby="mode-${mode}">${body}</div>`;
    if (focus) document.getElementById(focus)?.focus();
  }
  const click = (event: Event) => {
    const button = (event.target as HTMLElement).closest<HTMLButtonElement>(
      "button",
    );
    if (!button || button.disabled) return;
    if (button.dataset.mode) {
      mode = button.dataset.mode as Mode;
      stage = "edit";
      selected.clear();
      render(`mode-${mode}`);
      return;
    }
    switch (button.dataset.action) {
      case "preview":
        selected = new Set(changes().map((p) => p.key));
        stage = "preview";
        render();
        root.querySelector<HTMLInputElement>("input")?.focus();
        break;
      case "edit":
        stage = "edit";
        selected.clear();
        render();
        root.querySelector<HTMLSelectElement>("select")?.focus();
        break;
      case "confirm":
        result = changes()
          .filter((p) => selected.has(p.key))
          .map((p) => ({ key: p.key, value: drafts[p.key] }));
        stage = "result";
        render("demo-done");
        break;
      case "reset":
        mode = "basic";
        stage = "edit";
        drafts = initial();
        result = [];
        selected.clear();
        render("mode-basic");
        break;
    }
  };
  const change = (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (input.dataset.key) {
      drafts[input.dataset.key] = input.value;
      selected.clear();
      stage = "edit";
      render(input.id);
    }
    if (input.dataset.select) {
      const key = input.dataset.select;
      input.checked ? selected.add(key) : selected.delete(key);
      render();
      root.querySelector<HTMLInputElement>(`[data-select="${key}"]`)?.focus();
    }
  };
  const keydown = (event: KeyboardEvent) => {
    const tab = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-mode]",
    );
    if (!tab || !["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key))
      return;
    event.preventDefault();
    mode =
      event.key === "Home"
        ? "basic"
        : event.key === "End"
          ? "advanced"
          : mode === "basic"
            ? "advanced"
            : "basic";
    stage = "edit";
    selected.clear();
    render(`mode-${mode}`);
  };
  root.addEventListener("click", click);
  root.addEventListener("change", change);
  root.addEventListener("keydown", keydown);
  render();
  return () => {
    root.removeEventListener("click", click);
    root.removeEventListener("change", change);
    root.removeEventListener("keydown", keydown);
  };
}
