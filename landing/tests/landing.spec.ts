import { test, expect } from "@playwright/test";
import { readFileSync } from "node:fs";
import { en, ru } from "../src/content";

const { version } = JSON.parse(
  readFileSync(new URL("../../package.json", import.meta.url), "utf8"),
);

// Analytics are independent of the product demo and must not delay local tests.
const analytics = /^https:\/\/(?:mc\.yandex\.ru|yastatic\.net)\//;
test.beforeEach(async ({ page }) => {
  await page.route(analytics, (route) => route.abort());
});

for (const t of [ru, en]) {
  const path = t.lang === "ru" ? "/" : "/en/";
  test.describe(t.lang, () => {
    test("preview exclusions and selected result", async ({ page }) => {
      await page.goto(path);
      const demo = page.locator("#interactive-demo");
      await expect(
        demo.getByRole("button", { name: t.demo.preview }),
      ).toBeDisabled();
      await demo
        .getByLabel(t.demo.names[0], { exact: true })
        .selectOption("False");
      await demo.getByLabel(t.demo.names[1], { exact: true }).selectOption("2");
      await demo.getByRole("button", { name: t.demo.preview }).click();
      await expect(demo.locator(".diff-row")).toHaveCount(2);
      await expect(demo.locator(".diff-before")).toHaveText(["True", "3"]);
      await expect(demo.locator(".diff-after")).toHaveText(["False", "2"]);
      await expect(demo).toContainText("/Script/Engine.GameUserSettings");
      await expect(demo).toContainText("ScalabilityGroups");
      await demo
        .getByRole("checkbox", {
          name: `${t.demo.include}: sg.ShadowQuality`,
          exact: true,
        })
        .uncheck();
      await demo
        .getByRole("checkbox", {
          name: `${t.demo.include}: bUseVSync`,
          exact: true,
        })
        .uncheck();
      await expect(
        demo.getByRole("button", { name: t.demo.confirm }),
      ).toBeDisabled();
      await expect(demo.getByRole("status")).toHaveText(t.demo.empty);
      await demo
        .getByRole("checkbox", {
          name: `${t.demo.include}: bUseVSync`,
          exact: true,
        })
        .check();
      await expect(demo.getByRole("status")).toHaveText(
        `${t.demo.selected}: 1`,
      );
      await demo.getByRole("button", { name: t.demo.confirm }).click();
      await expect(
        demo.getByRole("heading", { name: t.demo.done }),
      ).toBeFocused();
      await expect(demo.locator(".result-value")).toHaveCount(1);
      await expect(demo.locator(".result-value")).toContainText("bUseVSync");
      await expect(demo.locator(".result-value")).toContainText("False");
      await expect(demo).toContainText(t.demo.disclaimer);
    });

    test("drafts survive mode switches and editing invalidates preview", async ({
      page,
    }) => {
      await page.goto(path);
      const demo = page.locator("#interactive-demo");
      await demo
        .getByLabel(t.demo.names[0], { exact: true })
        .selectOption("False");
      await demo.getByRole("tab", { name: t.demo.modes[1] }).click();
      await demo
        .getByLabel(t.demo.names[2], { exact: true })
        .selectOption("1024");
      await demo.getByRole("tab", { name: t.demo.modes[0] }).click();
      await expect(
        demo.getByLabel(t.demo.names[0], { exact: true }),
      ).toHaveValue("False");
      await demo.getByRole("tab", { name: t.demo.modes[1] }).click();
      await expect(
        demo.getByLabel(t.demo.names[2], { exact: true }),
      ).toHaveValue("1024");
      await demo.getByRole("button", { name: t.demo.preview }).click();
      await expect(demo.locator(".diff-after")).toHaveText("1024");
      await demo.getByRole("button", { name: t.demo.edit }).click();
      await demo
        .getByLabel(t.demo.names[2], { exact: true })
        .selectOption("2048");
      await expect(
        demo.getByRole("button", { name: t.demo.preview }),
      ).toBeDisabled();
      await demo.getByLabel(t.demo.names[3], { exact: true }).selectOption("0");
      await demo.getByRole("button", { name: t.demo.preview }).click();
      await expect(demo.locator(".diff-row")).toHaveCount(1);
      await expect(demo.locator(".diff-row")).toContainText(
        "r.Lumen.Reflections.Allow",
      );
      await demo.getByRole("button", { name: t.demo.confirm }).click();
      await demo.getByRole("button", { name: t.demo.reset }).click();
      await expect(
        demo.getByLabel(t.demo.names[0], { exact: true }),
      ).toHaveValue("True");
      await demo.getByRole("tab", { name: t.demo.modes[1] }).click();
      await expect(
        demo.getByLabel(t.demo.names[3], { exact: true }),
      ).toHaveValue("1");
      await expect(
        demo.getByRole("button", { name: t.demo.preview }),
      ).toBeDisabled();
    });

    test("keyboard tabs, visible focus and language navigation", async ({
      page,
    }) => {
      await page.goto(path);
      await page.keyboard.press("Tab");
      await expect(page.locator(".skip-link")).toBeFocused();
      await expect(page.locator(".skip-link")).toBeInViewport();
      const basic = page.getByRole("tab", { name: t.demo.modes[0] });
      await basic.focus();
      await page.keyboard.press("ArrowRight");
      await expect(
        page.getByRole("tab", { name: t.demo.modes[1] }),
      ).toBeFocused();
      await page.keyboard.press("Tab");
      await expect(
        page.getByLabel(t.demo.names[2], { exact: true }),
      ).toBeFocused();
      await expect(page.locator(":focus")).toHaveCSS("outline-style", "solid");
      await page.locator(".language").click();
      await expect(page.locator("html")).toHaveAttribute(
        "lang",
        t.lang === "ru" ? "en" : "ru",
      );
    });

    test("static content and direct downloads work without JavaScript", async ({
      browser,
    }) => {
      const context = await browser.newContext({
        javaScriptEnabled: false,
        locale: "en-US",
      });
      await context.route(analytics, (route) => route.abort());
      const page = await context.newPage();
      await page.goto(`http://127.0.0.1:4181${path}`);
      await expect(page.getByRole("heading", { level: 1 })).toHaveText(t.hero);
      expect(await page.evaluate(() => document.characterSet)).toBe("UTF-8");
      await expect(page.locator("html")).toHaveAttribute("lang", t.lang);
      const links = page.locator("[data-download]");
      await expect(links).toHaveCount(3);
      for (const link of await links.all())
        await expect(link).toHaveAttribute(
          "href",
          `https://github.com/MikeSaito/Game-Settings-Master/releases/download/v${version}/Game-Settings-Master_${version}_x64-setup.exe`,
        );
      await page.locator("#smartscreen summary").click();
      await expect(page.locator("#smartscreen p")).toBeVisible();
      await expect(page.locator("#smartscreen p")).toContainText(
        "Authenticode",
      );
      await expect(
        page.locator('a[href*="t.me"], a[href*="telegram"]'),
      ).toHaveCount(0);
      await context.close();
    });

    test("reduced motion, metadata and installation help", async ({ page }) => {
      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.goto(path);
      await expect(page.locator(".hero__visual")).toHaveCSS(
        "animation-name",
        "none",
      );
      await expect(page.locator('meta[name="description"]')).toHaveAttribute(
        "content",
        t.description,
      );
      await expect(page.locator('meta[property="og:title"]')).toHaveAttribute(
        "content",
        t.title,
      );
      const socialImage = `/og-image${t.lang === "en" ? "-en" : ""}.jpg`;
      await expect(page.locator('meta[property="og:image"]')).toHaveAttribute(
        "content",
        `https://gsm-tool.com${socialImage}`,
      );
      const response = await page.request.get(socialImage);
      expect(response.ok()).toBe(true);
      expect(response.headers()["content-type"]).toContain("image/jpeg");
      await expect(page.locator('link[rel="canonical"]')).toHaveAttribute(
        "href",
        `https://gsm-tool.com${path}`,
      );
      await page.getByRole("link", { name: t.help }).click();
      await expect(page.locator("#smartscreen")).toHaveAttribute("open", "");
      await expect(page.locator("#smartscreen p")).toBeVisible();
    });

    for (const [width, height] of [
      [1440, 900],
      [1280, 720],
      [768, 1024],
      [390, 844],
    ]) {
      for (const zoom of [100, 125]) {
        test(`layout ${width}x${height} at ${zoom}%`, async ({
          page,
        }, testInfo) => {
          const errors: string[] = [];
          page.on("pageerror", (error) => errors.push(error.message));
          page.on("response", (response) => {
            if (
              response.status() >= 400 &&
              response.url().startsWith("http://127.0.0.1:4181")
            )
              errors.push(`${response.status()} ${response.url()}`);
          });
          await page.setViewportSize({ width, height });
          await page.goto(path);
          if (zoom === 125)
            await page.addStyleTag({ content: "html { zoom: 1.25; }" });
          await page.evaluate(() => document.fonts.ready);
          const demo = page.locator("#interactive-demo");
          await demo
            .getByLabel(t.demo.names[0], { exact: true })
            .selectOption("False");
          await demo.getByRole("button", { name: t.demo.preview }).click();
          await expect(
            demo.getByRole("button", { name: t.demo.confirm }),
          ).toBeVisible();
          await expect
            .poll(() =>
              page.evaluate(
                () => document.documentElement.scrollWidth <= innerWidth + 1,
              ),
            )
            .toBe(true);
          for (const image of await page
            .locator(".feature__visual img")
            .all()) {
            await image.scrollIntoViewIfNeeded();
            await expect
              .poll(() =>
                image.evaluate(
                  (element: HTMLImageElement) =>
                    element.complete && element.naturalWidth > 0,
                ),
              )
              .toBe(true);
          }
          await expect(page.locator(".feature__visual img")).toHaveCount(3);
          await demo
            .getByRole("button", { name: t.demo.confirm })
            .scrollIntoViewIfNeeded();
          await expect(
            demo.getByRole("button", { name: t.demo.confirm }),
          ).toBeInViewport();
          await page.screenshot({
            path: testInfo.outputPath("preview.png"),
            fullPage: true,
          });
          await expect(errors).toEqual([]);
        });
      }
    }
  });
}
