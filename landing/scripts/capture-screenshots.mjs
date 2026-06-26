import { spawn } from "node:child_process";
import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = resolve(__dirname, "../..");
const outDir = resolve(__dirname, "../public/screenshots");
const frameWidth = 1440;
const frameHeight = 880;
const locales = ["ru", "en"];
const shots = [
  { id: "shot-library", file: "library.png" },
  { id: "shot-editor-basic", file: "editor-basic.png" },
  { id: "shot-editor-advanced", file: "editor-advanced.png" },
  { id: "shot-backups", file: "backups.png" },
];

function waitForUrl(url, timeoutMs = 60_000) {
  const start = Date.now();
  return new Promise((resolvePromise, reject) => {
    const tick = async () => {
      try {
        const res = await fetch(url);
        if (res.ok) {
          resolvePromise();
          return;
        }
      } catch {
        /* retry */
      }
      if (Date.now() - start > timeoutMs) {
        reject(new Error(`Timed out waiting for ${url}`));
        return;
      }
      setTimeout(tick, 400);
    };
    tick();
  });
}

function runVite() {
  const child = spawn("npm", ["run", "dev", "--", "--host", "127.0.0.1", "--port", "4177", "--strictPort"], {
    cwd: root,
    shell: true,
    stdio: "ignore",
    env: { ...process.env, GSM_SCREENSHOT: "1" },
  });
  return child;
}

async function main() {
  mkdirSync(outDir, { recursive: true });
  const vite = runVite();
  const baseUrl = "http://127.0.0.1:4177/screenshot.html";

  try {
    await waitForUrl(`${baseUrl}?lang=ru`);
    const browser = await chromium.launch();
    const page = await browser.newPage({
      viewport: { width: frameWidth, height: frameHeight },
      deviceScaleFactor: 2,
    });

    for (const locale of locales) {
      const localeDir = resolve(outDir, locale);
      mkdirSync(localeDir, { recursive: true });
      await page.goto(`${baseUrl}?lang=${locale}`, { waitUntil: "networkidle" });
      await page.waitForTimeout(1200);

      for (const shot of shots) {
        const frame = page.locator(`#${shot.id} .shot-frame__inner`);
        await frame.waitFor({ state: "visible" });
        await frame.screenshot({
          path: resolve(localeDir, shot.file),
          type: "png",
        });
        console.log(`saved ${locale}/${shot.file}`);
      }
    }

    await browser.close();
  } finally {
    vite.kill("SIGTERM");
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
