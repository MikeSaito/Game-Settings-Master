import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";

export const waitFor = async (fn, timeout = 30000) => {
  const deadline = Date.now() + timeout;
  let last;
  while (Date.now() < deadline) {
    try { const result = await fn(); if (result) return result; } catch (error) { last = error; }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw last ?? new Error("Timed out waiting for desktop application");
};
export async function launch(application, env, artifactDir) {
  await mkdir(artifactDir, { recursive: true });
  const driver = spawn(process.env.TAURI_DRIVER ?? "tauri-driver", [], { env: { ...process.env, ...env }, windowsHide: true });
  const output = [];
  driver.stdout.on("data", (bytes) => output.push(bytes));
  driver.stderr.on("data", (bytes) => output.push(bytes));
  let startupError;
  driver.on("error", (error) => { startupError = error; });
  driver.on("exit", (code) => { startupError = new Error(`tauri-driver exited: ${code}`); });
  async function request(path, body, method = "POST", timeout = 30000) {
    if (startupError) throw startupError;
    const response = await fetch(`http://127.0.0.1:4444${path}`, { method, headers: { "Content-Type": "application/json" }, body: body === undefined ? undefined : JSON.stringify(body), signal: AbortSignal.timeout(timeout) });
    const json = await response.json();
    if (!response.ok || json.value?.error) throw new Error(JSON.stringify(json.value));
    return json.value;
  }
  await waitFor(() => request("/status", undefined, "GET"));
  let session;
  try {
    // EdgeDriver interprets/reorders arguments as Chromium switches. Keep the
    // path attached to its switch so it survives WebDriver and NSIS relaunch.
    const args = env.GSM_NATIVE_TEST_ROOT ? [`--gsm-native-test-root=${env.GSM_NATIVE_TEST_ROOT}`] : [];
    session = await request("/session", { capabilities: { alwaysMatch: { browserName: "wry", "tauri:options": { application, args } } } }, "POST", 120000);
    // WebView2 can return a session while its initial about:blank is still active.
    // Wait for the application document before consumers access origin storage.
    await waitFor(() => request(`/session/${session.sessionId}/execute/sync`, {
      script: "return location.protocol !== 'about:' && document.readyState === 'complete' && !!document.getElementById('root')",
      args: [],
    }));
  }
  catch (error) {
    driver.kill();
    await writeFile(`${artifactDir}/driver.log`, Buffer.concat(output));
    throw error;
  }
  const base = `/session/${session.sessionId}`;
  const command = (path, body, method) => request(base + path, body, method);
  const execute = (script, args = []) => command("/execute/sync", { script, args });
  const find = async (using, value) => {
    const element = await waitFor(() => command("/element", { using, value }));
    return element["element-6066-11e4-a52e-4f735466cecf"];
  };
  const click = async (xpath) => waitFor(async () => {
    await command(`/element/${await find("xpath", xpath)}/click`, {});
    return true;
  });
  const button = (text) => click(`//button[normalize-space(.)='${text}' and not(@disabled)]`);
  return {
    execute, command, button, click,
    async fill(css, text) {
      await waitFor(() => execute("const el=document.querySelector(arguments[0]);return el && !el.readOnly && !el.disabled", [css]));
      const id = await find("css selector", css);
      await command(`/element/${id}/clear`, {});
      await command(`/element/${id}/value`, { text });
    },
    async select(file) { await click(`//select[@id='ini-file']/option[@value='${file}']`); },
    async close() {
      await mkdir(artifactDir, { recursive: true });
      try { await writeFile(`${artifactDir}/screenshot.png`, Buffer.from(await command("/screenshot", undefined, "GET"), "base64")); } catch {}
      try { await writeFile(`${artifactDir}/page.html`, await command("/source", undefined, "GET")); } catch {}
      try { await command("", undefined, "DELETE"); } catch {}
      driver.kill();
      await writeFile(`${artifactDir}/driver.log`, Buffer.concat(output));
      await new Promise((resolve) => setTimeout(resolve, 500));
    },
    async terminate() {
      driver.kill();
      await writeFile(`${artifactDir}/driver.log`, Buffer.concat(output));
      await new Promise((resolve) => setTimeout(resolve, 500));
    },
  };
}
