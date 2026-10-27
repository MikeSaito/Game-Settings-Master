import { spawn, spawnSync } from "node:child_process";
import { mkdirSync, unlinkSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { chromium, expect } from "@playwright/test";

// Render the actual application with isolated demo IPC data, not static UI frames.
const root = resolve(import.meta.dirname, "..");
const baseUrl = "http://127.0.0.1:4178/e2e.html";
const vite = resolve(
  dirname(fileURLToPath(import.meta.resolve("vite/package.json"))),
  "bin/vite.js",
);
const server = spawn(
  process.execPath,
  [vite, "--host", "127.0.0.1", "--port", "4178", "--strictPort"],
  {
    cwd: root,
    stdio: "inherit",
    env: { ...process.env, GSM_E2E: "1" },
  },
);
let serverExit = null;
server.on("exit", (code) => {
  serverExit = code ?? -1;
});

async function waitForServer() {
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    if (serverExit !== null)
      throw new Error(`Preview server exited: ${serverExit}`);
    try {
      if ((await fetch(baseUrl, { signal: AbortSignal.timeout(2_000) })).ok)
        return;
    } catch {
      /* starting */
    }
    await new Promise((done) => setTimeout(done, 250));
  }
  throw new Error("Preview server did not become ready");
}

let browser;
try {
  await waitForServer();
  browser = await chromium.launch();
  for (const language of ["en", "ru"]) {
    for (const theme of ["dark", "light"]) {
      const context = await browser.newContext({
        viewport: { width: 1920, height: 1080 },
        deviceScaleFactor: 1.25,
        locale: language === "ru" ? "ru-RU" : "en-US",
        timezoneId: "Europe/Moscow",
        reducedMotion: "reduce",
        colorScheme: theme,
      });
      await context.addInitScript((language) => {
        sessionStorage.setItem("gsm-e2e-language", language);
        sessionStorage.setItem("gsm-e2e-fixtures", "readme");
      }, language);
      const page = await context.newPage();
      page.setDefaultTimeout(15_000);
      await page.clock.setFixedTime(new Date("2026-10-24T09:00:00+03:00"));
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      page.on("console", (message) => {
        if (message.type() === "error") errors.push(message.text());
      });
      const directory = resolve(root, "docs/screenshots", language, theme);
      mkdirSync(directory, { recursive: true });
      const label = (en, ru) => (language === "ru" ? ru : en);
      const tab = (en, ru) =>
        page.getByRole("tab", { name: label(en, ru), exact: true });
      async function capture(name) {
        await page.evaluate(() => document.fonts.ready);
        await page.mouse.move(1910, 1070);
        const png = resolve(directory, `${name}.png`);
        await page.screenshot({
          path: png,
          animations: "disabled",
          caret: "hide",
        });
        const result = spawnSync(
          "python",
          [
            "-c",
            "from PIL import Image; import sys; Image.open(sys.argv[1]).save(sys.argv[2], 'WEBP', lossless=True)",
            png,
            resolve(directory, `${name}.webp`),
          ],
          { encoding: "utf8" },
        );
        if (result.status !== 0)
          throw new Error(`WebP conversion failed: ${result.stderr}`);
        unlinkSync(png);
        console.log(`saved docs/screenshots/${language}/${theme}/${name}.webp`);
      }

      await page.goto(`${baseUrl}?theme=${theme}`);
      await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
      await expect(
        page.getByText(label("Example UE5 Game", "Пример игры UE5")),
      ).toBeVisible();
      await capture("library");
      await page
        .locator("article")
        .filter({ hasText: label("Example UE5 Game", "Пример игры UE5") })
        .getByRole("button", { name: label("Select", "Выбрать"), exact: true })
        .click();
      await expect(page.getByTestId("parameter-row").first()).toBeVisible();
      await capture("editor-basic");

      await tab("Advanced", "Расширенное").click();
      await expect(page.getByTestId("parameter-row").first()).toBeVisible();
      await capture("editor-advanced");

      await tab("Files", "Файлы").click();
      await expect(
        page.getByRole("textbox", {
          name: label("INI file text", "Текст INI-файла"),
        }),
      ).toHaveValue(/r\.Tonemapper\.Sharpen=0\.5/);
      await capture("editor-files");

      await tab("Basic", "Базовое").click();
      await page
        .getByTestId("parameter-row")
        .filter({ hasText: "VSync" })
        .getByRole("switch")
        .click();
      await page
        .getByRole("button", {
          name: label(
            "Apply to GameUserSettings",
            "Применить в пользовательские настройки",
          ),
          exact: true,
        })
        .click();
      const dialog = page.getByRole("dialog");
      await expect(dialog).toBeVisible();
      await expect(dialog.getByRole("checkbox")).toHaveCount(1);
      await capture("preview");
      await dialog
        .getByRole("button", {
          name: label("Confirm and apply", "Подтвердить и применить"),
          exact: true,
        })
        .click();
      await expect(dialog).not.toBeVisible();
      await tab("Backups", "Бэкапы").click();
      await expect(
        page
          .getByRole("button", {
            name: label("Restore", "Восстановить"),
            exact: true,
          })
          .first(),
      ).toBeVisible();
      await capture("backups");
      await context.close();
      if (errors.length)
        throw new Error(
          `${language}/${theme} rendering errors:\n${errors.join("\n")}`,
        );
    }
  }
} finally {
  await browser?.close();
  if (process.platform === "win32")
    spawnSync("taskkill", ["/pid", String(server.pid), "/T", "/F"], {
      stdio: "ignore",
    });
  else server.kill("SIGTERM");
}
