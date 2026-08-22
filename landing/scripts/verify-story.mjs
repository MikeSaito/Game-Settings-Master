import { spawn } from "node:child_process";
import { mkdirSync, rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "playwright";

const __dirname = dirname(fileURLToPath(import.meta.url));
const landingRoot = resolve(__dirname, "..");
const outDir = resolve(landingRoot, ".verify");
const port = 4178;
const baseUrl = `http://127.0.0.1:${port}/`;

const sceneFractions = {
  hero: [0.5],
  scan: [0.5, 0.95],
  backup: [0.45, 0.95],
  modes: [0.3, 0.85],
  final: [0.2, 0.85],
};

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
  return spawn(
    "npm",
    ["run", "dev", "--", "--host", "127.0.0.1", "--port", String(port), "--strictPort"],
    { cwd: landingRoot, shell: true, stdio: "ignore" },
  );
}

function killVite(vite) {
  if (process.platform === "win32") {
    // shell:true оборачивает vite в cmd: SIGTERM убил бы только обертку.
    spawn("taskkill", ["/pid", String(vite.pid), "/T", "/F"], { stdio: "ignore" });
    return;
  }
  vite.kill("SIGTERM");
}

async function measureScenes(page) {
  return page.evaluate(() => {
    const vh = window.innerHeight;
    return [...document.querySelectorAll(".scene")].map((s) => {
      const parent = s.parentElement;
      const el = parent && parent.className.includes("pin-spacer") ? parent : s;
      const rect = el.getBoundingClientRect();
      return {
        name: s.getAttribute("data-scene"),
        top: rect.top + window.scrollY,
        scrollable: rect.height - vh,
      };
    });
  });
}

async function main() {
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  const vite = runVite();
  try {
    await waitForUrl(baseUrl);
    const browser = await chromium.launch();
    const page = await browser.newPage({ viewport: { width: 1440, height: 900 } });
    await page.goto(baseUrl, { waitUntil: "networkidle" });
    await page.waitForTimeout(1200);

    const scenes = await measureScenes(page);
    console.log("scenes:", JSON.stringify(scenes));

    // Headless может не гнать rAF между кадрами: прокачиваем тикер GSAP вручную.
    const pump = () =>
      page.evaluate(
        () =>
          new Promise((resolvePump) => {
            let n = 0;
            const tick = () => {
              n += 1;
              if (n > 100) resolvePump();
              else requestAnimationFrame(tick);
            };
            requestAnimationFrame(tick);
          }),
      );

    for (const scene of scenes) {
      const fractions = sceneFractions[scene.name] ?? [0.5];
      for (const f of fractions) {
        const y = Math.round(scene.top + scene.scrollable * f);
        await page.evaluate((scrollY) => window.scrollTo(0, scrollY), y);
        await pump();
        const file = resolve(outDir, `${scene.name}-${Math.round(f * 100)}.png`);
        await page.screenshot({ path: file });
        console.log(`saved ${scene.name}-${Math.round(f * 100)}.png`);
      }
    }

    await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
    await pump();
    await page.screenshot({ path: resolve(outDir, "flow.png") });
    console.log("saved flow.png");

    const mobile = await browser.newPage({ viewport: { width: 390, height: 844 } });
    await mobile.goto(baseUrl, { waitUntil: "networkidle" });
    await mobile.waitForTimeout(900);
    await mobile.evaluate(() => {
      const el = document.querySelector('[data-scene="scan"]');
      el?.scrollIntoView();
    });
    await mobile.waitForTimeout(900);
    await mobile.screenshot({ path: resolve(outDir, "mobile-scan.png") });
    console.log("saved mobile-scan.png");

    await browser.close();
  } finally {
    killVite(vite);
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
