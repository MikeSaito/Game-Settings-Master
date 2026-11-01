import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { createHash, randomUUID } from "node:crypto";
import { createServer } from "node:http";
import { mkdir, mkdtemp, readFile, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { launch, waitFor } from "./driver.mjs";

if (process.platform !== "win32")
  throw new Error("Windows performance measurement required");
const application = resolve(
  process.env.GSM_TEST_BINARY ??
    "src-tauri/target/release/ue-settings-master.exe",
);
const label = process.env.GSM_PERFORMANCE_LABEL ?? "candidate";
const artifacts = resolve(`test-results/performance/${label}`);
await mkdir(artifacts, { recursive: true });
await mkdir(resolve(".verify"), { recursive: true });
const root = await mkdtemp(resolve(`.verify/performance-${label}-`));
await mkdir(join(root, "app-data/diagnostics"), { recursive: true });
const games = [];
const engine =
  "[SystemSettings]\n" +
  Array.from({ length: 1000 }, (_, i) => `r.PerformanceFixture${i}=1\n`).join(
    "",
  );
for (let i = 0; i < 8; i++) {
  const install = join(root, `fixture-${i}/install`);
  const config = join(root, `fixture-${i}/Saved/Config/Windows`);
  await mkdir(install, { recursive: true });
  await mkdir(config, { recursive: true });
  await writeFile(
    join(config, "GameUserSettings.ini"),
    "[ScalabilityGroups]\nsg.ShadowQuality=2\n",
  );
  await writeFile(join(config, "Engine.ini"), engine);
  const game = {
    id: `manual-performance-${i}`,
    name: `Performance Fixture ${i}`,
    source: "manual",
    install_dir: install,
    config_dir: config,
    exe_name: `gsm-performance-game-${i}.exe`,
    is_ue: true,
    engine_family: "ue5",
    engine_version: "5.4",
  };
  games.push(game);
  const operations = Array.from({ length: 200 }, (_, n) => ({
    id: String(n),
    group: "fixture",
    file: "Engine.ini",
    section: "SystemSettings",
    key: `r.PerformanceFixture${n}`,
    before: null,
    after: "1",
    kind: "scalar",
  }));
  const name = createHash("sha256").update(game.id).digest("hex");
  await writeFile(
    join(root, "app-data/diagnostics", `${name}.json`),
    JSON.stringify({
      profile: game,
      dir: config,
      operations,
      file_hashes: {},
      report: null,
      generation: `fixture-${i}`,
      input_identities: {},
    }),
  );
}
await writeFile(join(root, "app-data/games.json"), JSON.stringify({ games }));
await writeFile(join(root, "initialized"), "1");
for (let i = 0; i < 256; i++) {
  const id = randomUUID();
  const folder = join(root, "app-data/transactions", id);
  await mkdir(folder, { recursive: true });
  await writeFile(
    join(folder, "journal.json"),
    JSON.stringify({
      version: 1,
      id,
      game_id: games[0].id,
      exe: null,
      created_at: new Date().toISOString(),
      state: "complete",
      originals: [
        {
          dir: games[0].config_dir,
          file: "Engine.ini",
          hash: null,
          attributes: 0,
        },
      ],
    }),
  );
}
const server = createServer((_, response) => {
  response.writeHead(204);
  response.end();
});
await new Promise((done) => server.listen(17891, "127.0.0.1", done));
let app;
const result = {
  label,
  fixture_games: 8,
  diagnostic_operations_per_game: 200,
  completed_journals: 256,
  startup: [],
  phases: {},
};
async function sample(phase, minimize = false) {
  const output = join(artifacts, `${phase}.json`);
  const args = [
    "-NoProfile",
    "-ExecutionPolicy",
    "Bypass",
    "-File",
    resolve("scripts/measure-process-tree.ps1"),
    "-Executable",
    application,
    "-Output",
    output,
    "-Seconds",
    process.env.GSM_PERFORMANCE_SECONDS ?? "60",
  ];
  if (minimize) args.push("-Minimize");
  const child = spawn("powershell.exe", args, {
    windowsHide: true,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const errors = [];
  child.stderr.on("data", (bytes) => errors.push(bytes));
  const code = await new Promise((done, reject) => {
    child.on("exit", done);
    child.on("error", reject);
  });
  assert.equal(code, 0, Buffer.concat(errors).toString());
  const data = JSON.parse(
    (await readFile(output, "utf8")).replace(/^\uFEFF/, ""),
  );
  assert.ok(data.samples.length >= 5, "Insufficient performance samples");
  const samples = data.samples.slice(1);
  result.phases[phase] = {
    ...data,
    average_cpu_percent:
      ((samples.reduce((sum, row) => sum + row.cpu_ms, 0) /
        ((samples.at(-1).seconds - data.samples[0].seconds) * 1000)) *
        100) /
      data.logical_processors,
    private_start_mib: samples[0].private_mib,
    private_end_mib: samples.at(-1).private_mib,
    private_peak_mib: Math.max(...samples.map((row) => row.private_mib)),
  };
  await writeFile(
    join(artifacts, "summary.json"),
    JSON.stringify(result, null, 2),
  );
}
try {
  for (let run = 0; run < 3; run++) {
    const started = Date.now();
    app = await launch(
      application,
      { GSM_NATIVE_TEST_ROOT: root },
      join(artifacts, `startup-${run}`),
    );
    await waitFor(() =>
      app.execute(
        "return document.body.innerText.includes('Performance Fixture 0')",
      ),
    );
    result.startup.push({
      driver_to_library_upper_bound_ms: Date.now() - started,
      ...(await app.execute(
        "return {document_to_library_upper_bound_ms:performance.now(),first_contentful_paint_ms:performance.getEntriesByName('first-contentful-paint')[0]?.startTime??null}",
      )),
    });
    if (run < 2) {
      await app.close();
      app = null;
    }
  }
  await app.execute(
    "localStorage.setItem('gsm-app-settings',JSON.stringify({language:'en',theme:'dark'}));localStorage.setItem('uesm:language','en');location.reload();return true",
  );
  await waitFor(() =>
    app.execute(
      "return document.documentElement.dataset.nativeReady==='true' && document.body.innerText.includes('Performance Fixture 0') && !!Array.from(document.querySelectorAll('button')).find(el=>el.textContent.trim()==='Select')",
    ),
  );
  await app.execute(
    "window.__performanceCommands={};const original=window.__TAURI_INTERNALS__.invoke;window.__TAURI_INTERNALS__.invoke=function(command,args,...rest){window.__performanceCommands[command]=(window.__performanceCommands[command]??0)+1;return original.call(this,command,args,...rest)};return true",
  );
  await new Promise((done) => setTimeout(done, 30_000));
  await sample("library_idle");
  await app.execute("window.__performanceCommands={};return true");
  await sample("minimized", true);
  result.background_commands = await app.execute(
    "return window.__performanceCommands",
  );
  await app.button("Select");
  await app.click("//*[@role='tab' and normalize-space(.)='Files']");
  await sample("editor_idle");
  result.before_stress_heap = await app.execute(
    "return performance.memory?.usedJSHeapSize??null",
  );
  await app.execute(
    "window.__performanceLag=0;let last=performance.now();window.__performanceTimer=setInterval(()=>{const now=performance.now();window.__performanceLag=Math.max(window.__performanceLag,now-last-50);last=now},50);return true",
  );
  for (let cycle = 0; cycle < 20; cycle++) {
    await app.select("Engine.ini");
    await app.execute(
      "const el=document.querySelector('#ini-text');Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype,'value').set.call(el,arguments[0]);el.dispatchEvent(new Event('input',{bubbles:true}));return true",
      [engine + `; draft ${cycle}\n`],
    );
    await app.button("Preview and save");
    await waitFor(() =>
      app.execute("return !!document.querySelector('dialog[open]')"),
    );
    await app.button("Cancel");
    await app.click("//*[@role='tab' and normalize-space(.)='Backups']");
    await app.click("//*[@role='tab' and normalize-space(.)='Files']");
  }
  result.stress = await app.execute(
    "clearInterval(window.__performanceTimer);return {max_event_loop_delay_ms:window.__performanceLag,js_heap_bytes:performance.memory?.usedJSHeapSize??null}",
  );
  await sample("after_stress");
  result.active_previews = await app.execute(
    "return document.querySelectorAll('dialog[open]').length",
  );
  assert.equal(result.active_previews, 0);
  if (label === "candidate") {
    assert.ok(
      (result.background_commands.list_pending_operations ?? 0) <= 1,
      "Recovery IPC should stop when minimized",
    );
    assert.ok(
      result.phases.minimized.average_cpu_percent < 1,
      "Sustained minimized CPU usage exceeds 1% of the machine",
    );
  }
  await writeFile(
    join(artifacts, "summary.json"),
    JSON.stringify(result, null, 2),
  );
  console.log(
    JSON.stringify(
      {
        label,
        startup: result.startup,
        phases: Object.fromEntries(
          Object.entries(result.phases).map(([key, value]) => [
            key,
            {
              average_cpu_percent: value.average_cpu_percent,
              private_start_mib: value.private_start_mib,
              private_end_mib: value.private_end_mib,
              private_peak_mib: value.private_peak_mib,
            },
          ]),
        ),
        stress: result.stress,
      },
      null,
      2,
    ),
  );
} finally {
  if (app) await app.close();
  server.close();
}
