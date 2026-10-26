import assert from "node:assert/strict";
import { mkdtemp, readFile, mkdir, writeFile } from "node:fs/promises";
import { resolve, join } from "node:path";
import { createServer } from "node:http";
import { launch, waitFor } from "./driver.mjs";

const application = resolve(process.env.GSM_TEST_BINARY ?? "src-tauri/target/debug/ue-settings-master.exe");
await mkdir(resolve(".verify"), { recursive: true });
const root = await mkdtemp(resolve(".verify/gsm-native-"));
const config = join(root, "fixture/Saved/Config/Windows");
const artifacts = resolve("desktop-test-results/configs");
console.log(`Native fixtures: ${root}`);
await mkdir(artifacts, { recursive: true });
// A real updater endpoint with no update; config tests never contact production.
const server = createServer((_, response) => { response.writeHead(204); response.end(); });
await new Promise((resolve) => server.listen(17891, "127.0.0.1", resolve));
let app;
const env = { GSM_NATIVE_TEST_ROOT: root };
async function start(crash = "") {
  app = await launch(application, { ...env, GSM_TEST_CRASH_AT: crash }, join(artifacts, crash || "normal"));
  await waitFor(() => app.execute("return document.body.innerText.includes('Native Test Game')"));
  await app.execute("localStorage.setItem('gsm-app-settings', JSON.stringify({language:'en'}));localStorage.setItem('uesm:language','en');location.reload();return true");
  await waitFor(() => app.execute("return document.body.innerText.includes('Native Test Game')"));
}
async function openFiles() {
  await app.button("Select");
  await app.click("//*[@role='tab' and normalize-space(.)='Files']");
}
async function preview(text, engine) {
  await app.select("GameUserSettings.ini");
  await app.fill("#ini-text", text);
  if (engine) { await app.select("Engine.ini"); await app.fill("#ini-text", engine); }
  await app.button("Preview and save");
  await waitFor(() => app.execute("return !!document.querySelector('dialog[open], [role=dialog]')"));
  await app.click("//input[@type='checkbox' and following-sibling::span[contains(.,'reviewed')]] | //label[contains(.,'I have reviewed the warnings')]//input");
}
const original = "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n";
async function verifyMaterialWorkspace() {
  await app.button("Select");
  for (const [button, theme] of [["Light", "light"], ["Dark", "dark"]]) {
    await app.click(`//button[@aria-label='${button}']`);
    await waitFor(async () => (await app.execute("return document.documentElement.dataset.theme")) === theme);
    await writeFile(join(artifacts, `material-${theme}.png`), Buffer.from(await app.command("/screenshot", undefined, "GET"), "base64"));
  }
  await app.click("//button[@aria-label='Game details']");
  await waitFor(() => app.execute("return !!document.querySelector('dialog[open]') && document.querySelector('dialog[open]').innerText.includes('Native Test Game')"));
  await app.click("//dialog[@open]//button[@aria-label='Close']");
  await app.click("//button[@data-parameter-details]");
  await waitFor(() => app.execute("return !!document.querySelector('dialog[open] [data-testid=parameter-list-details]')"));
  await app.click("//dialog[@open]//button[@aria-label='Close']");
  await app.button("Save preset");
  await waitFor(() => app.execute("return !!document.querySelector('dialog[open]') && document.querySelector('dialog[open]').innerText.includes('Preset save options')"));
  await app.click("//dialog[@open]//button[@aria-label='Close']");
  await app.click("//*[@role='tab' and normalize-space(.)='Files']");
  console.log("Native light/dark themes, parameter details, game details and preset dialog passed");
}
try {
  await start(); await verifyMaterialWorkspace();
  await preview("[ScalabilityGroups]\r\nsg.ShadowQuality=3\r\n");
  await app.button("Confirm and apply");
  await waitFor(async () => (await readFile(join(config,"GameUserSettings.ini"),"utf8")).includes("sg.ShadowQuality=3"));
  await app.click("//*[@role='tab' and normalize-space(.)='Backups']");
  await app.button("Refresh");
  await app.button("Restore"); await app.button("Confirm and apply");
  await waitFor(async () => (await readFile(join(config,"GameUserSettings.ini"),"utf8")) === original);
  await app.close(); app = null;
  // Exit from inside the real transaction after its first physical write.
  await start("write-0"); await openFiles();
  await preview("[ScalabilityGroups]\r\nsg.ShadowQuality=4\r\n", "[SystemSettings]\r\nr.ViewDistanceScale=2\r\n");
  await app.button("Confirm and apply").catch(() => {});
  await waitFor(async () => (await readFile(join(config,"Engine.ini"),"utf8")).includes("r.ViewDistanceScale=2"));
  await app.terminate(); app = null;
  await start();
  await waitFor(() => app.execute("return !!document.querySelector('[data-testid=recovery-panel]')"));
  await app.button("Later");
  // The backend must reject another write, even after deferring the panel.
  await openFiles(); await preview("[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n");
  await app.button("Confirm and apply");
  await waitFor(() => app.execute("return document.body.innerText.includes('Unfinished write:')"));
  await app.button("Cancel"); await app.button("Show recovery");
  await app.close(); app = null;
  await start("recovery-0");
  await app.click("//*[@data-testid='recovery-panel']//button[normalize-space(.)='Restore']").catch(() => {});
  await waitFor(async () => { try { await readFile(join(config,"Engine.ini")); return false; } catch (error) { return error.code === "ENOENT"; } });
  await app.terminate(); app = null;
  await start();
  await app.click("//*[@data-testid='recovery-panel']//button[normalize-space(.)='Restore']");
  await waitFor(() => app.execute("return !document.querySelector('[data-testid=recovery-panel]')"));
  assert.equal(await readFile(join(config,"GameUserSettings.ini"),"utf8"),original);
  await assert.rejects(readFile(join(config,"Engine.ini")), { code:"ENOENT" });
  console.log("Native save, restore, crash recovery and deferred-write protection passed");
} finally { if (app) await app.close(); server.close(); }
